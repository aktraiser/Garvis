// Sprint 1 Niveau 1: LLM Response Generation
// Wrapper pour synthèse de réponses avec LLM

import { invoke } from "@tauri-apps/api/core";
import { LiteLLMClient, modelConfigStore } from "./litellm";

// Types correspondant aux structs Rust
export interface LlmContextResponse {
  session_id: string;
  formatted_context: string;
  chunks: LlmChunkInfo[];
  query: string;
  search_time_ms: number;
  has_ocr_data: boolean;
}

export interface LlmChunkInfo {
  chunk_id: string;
  source_label: string;
  content: string;
  score: number;
  confidence: number;
  page: number | null;
  figure_id: string | null;
  source_type: string;
}

export interface ChatRequest {
  session_id: string;
  query: string;
  selection: any | null;
  limit: number | null;
}

export interface LlmChatResponse {
  answer: string;
  sources: LlmChunkInfo[];
  confidence: number;
  has_ocr_warning: boolean;
  search_time_ms: number;
  llm_time_ms: number;
}

// Prompt template - Version finale 100% fidélité (best practices 2024-2025)
// Objectif: Zéro hallucination, ignorer bruit, citations systématiques, focus WHY pas HOW
const LLM_ANSWER_PROMPT = `You are a document analysis assistant. Your task: answer the question using ONLY the excerpts provided below.

CRITICAL RULES - FOLLOW EXACTLY:

1. **SOURCE FIDELITY**
   - Use ONLY facts explicitly stated in the excerpts
   - NEVER add information from your general knowledge
   - If unsure → state "The document does not provide enough information"

2. **NOISE FILTERING - ENHANCED**
   - Some excerpts may be irrelevant noise: technical details, architecture, benchmarks, tables
   - IGNORE excerpts that don't directly answer the question
   - Focus on excerpts containing the core answer (Abstract, Introduction, Conclusion)
   - **CRITICAL**: If the question asks about "objective" or "goal" → answer the strategic WHY, NOT the technical HOW
   - Avoid describing implementation details (components, pipelines, architecture) unless explicitly asked

3. **STRATEGIC VS TECHNICAL**
   - Question about "objective/goal/purpose" → Answer: WHY does this exist? What problem does it solve?
   - Question about "method/approach/architecture" → Answer: HOW does it work? What components?
   - Question about "results/performance" → Answer: What metrics? What outcomes?

4. **CITATION DISCIPLINE**
   - Cite source for EVERY claim: "According to [Source Label]..."
   - For numbers/tables: cite exactly (e.g., "Table 3 shows...")
   - Multiple sources → list them (e.g., "Sources 3 and 5 state...")

5. **ANSWER IN FRENCH**
   - 2-4 concise sentences maximum
   - Direct answer first, details after
   - Use bullet points if multiple aspects
   - Prioritize clarity and strategic insight over technical jargon

6. **QUALITY OVER QUANTITY**
   - Better 1 perfect sentence from good excerpt than 3 diluted sentences from mixed excerpts
   - Prefer Abstract/Introduction excerpts over technical sections for objective questions

---

EXCERPTS FROM DOCUMENT:
{context}

QUESTION: {question}

YOUR ANSWER (in French, precise, strategic, with inline citations):`;

/**
 * Synthèse LLM complète (Sprint 1 Niveau 1)
 * 1. Appelle Rust pour obtenir le contexte formaté (RAG)
 * 2. Appelle LLM pour synthèse
 * 3. Retourne réponse structurée
 */
export async function chatWithLlmSynthesis(
  sessionId: string,
  query: string,
  selection: any | null = null,
  limit: number | null = 10
): Promise<LlmChatResponse> {
  const startTime = performance.now();

  // 1. Appel Rust : RAG + contexte formaté
  console.log("🔍 Fetching LLM context from Rust...");
  const contextResponse: LlmContextResponse = await invoke(
    "chat_with_llm_context",
    {
      request: {
        session_id: sessionId,
        query,
        selection,
        limit,
      },
    }
  );

  console.log(
    `✅ Got context: ${contextResponse.chunks.length} chunks, ${contextResponse.formatted_context.length} chars`
  );

  if (contextResponse.chunks.length === 0) {
    return {
      answer:
        "Je n'ai pas trouvé d'informations pertinentes pour répondre à votre question dans ce document.",
      sources: [],
      confidence: 0.0,
      has_ocr_warning: false,
      search_time_ms: contextResponse.search_time_ms,
      llm_time_ms: 0,
    };
  }

  // 2. Construire le prompt avec le contexte
  const prompt = LLM_ANSWER_PROMPT.replace(
    "{context}",
    contextResponse.formatted_context
  ).replace("{question}", query);

  console.log(
    `🤖 Calling LLM with ${prompt.length} chars prompt (${
      contextResponse.chunks.length
    } chunks × ~${Math.round(
      contextResponse.formatted_context.length / contextResponse.chunks.length
    )} chars/chunk)`
  );

  const llmStartTime = performance.now();

  // 3. Appel LLM via LiteLLMClient
  const config = modelConfigStore.getConfig();
  const client = new LiteLLMClient(config);

  let llmAnswer = "";
  try {
    const response = await client.chat([
      {
        role: "user",
        content: prompt,
      },
    ]);

    llmAnswer = response.choices[0].message.content;
    console.log(
      `✅ LLM response: ${llmAnswer.length} chars, ${response.usage?.total_tokens || "?"} tokens`
    );
  } catch (error) {
    console.error("❌ LLM call failed:", error);
    throw new Error(`LLM synthesis failed: ${error}`);
  }

  const llmTime = performance.now() - llmStartTime;

  // 4. Ajouter avertissement OCR si nécessaire
  const hasOcrWarning = contextResponse.has_ocr_data;
  let finalAnswer = llmAnswer;

  if (hasOcrWarning) {
    finalAnswer += `\n\n⚠️ Note: Cette réponse contient des données extraites par OCR. Vérifiez visuellement dans le document pour les valeurs exactes.`;
  }

  // 5. Calculer confidence (simple: score du top-1 chunk)
  const confidence =
    contextResponse.chunks.length > 0 ? contextResponse.chunks[0].score : 0.0;

  const totalTime = performance.now() - startTime;
  console.log(
    `✅ LLM Synthesis complete: ${totalTime.toFixed(0)}ms total (RAG: ${contextResponse.search_time_ms}ms, LLM: ${llmTime.toFixed(0)}ms)`
  );

  return {
    answer: finalAnswer,
    sources: contextResponse.chunks,
    confidence,
    has_ocr_warning: hasOcrWarning,
    search_time_ms: contextResponse.search_time_ms,
    llm_time_ms: Math.round(llmTime),
  };
}

/**
 * Version streaming (pour future implémentation)
 * TODO: Niveau 2 ou 3
 */
export async function chatWithLlmSynthesisStream(
  sessionId: string,
  query: string,
  onChunk: (chunk: string) => void,
  selection: any | null = null,
  limit: number | null = 10
): Promise<LlmChatResponse> {
  // TODO: Implémenter streaming avec chatStream de LiteLLMClient
  throw new Error("Streaming not yet implemented - use chatWithLlmSynthesis");
}
