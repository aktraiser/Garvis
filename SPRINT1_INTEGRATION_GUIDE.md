# Sprint 1 - Niveau 1: LLM Synthesis Integration Guide

> **Date** : 20 novembre 2024
> **Status** : ✅ Backend implémenté, Frontend prêt
> **Next Step** : Intégration dans DirectChatPage

---

## 🎯 Ce qui a été implémenté

### ✅ Backend Rust

**Fichier** : [`gravis-app/src-tauri/src/rag/direct_chat_commands.rs`](gravis-app/src-tauri/src/rag/direct_chat_commands.rs)

1. **Structs ajoutées** (lignes 99-123) :
   ```rust
   pub struct LlmContextResponse
   pub struct LlmChunkInfo
   ```

2. **Nouvelle commande Tauri** (lignes 289-338) :
   ```rust
   #[tauri::command]
   pub async fn chat_with_llm_context(...)
   ```

3. **Helper function** (lignes 1685-1747) :
   ```rust
   fn build_llm_context(...) -> (String, Vec<LlmChunkInfo>, bool)
   ```

**Features** :
- ✅ Troncature à 800 chars par chunk (roadmap recommendation)
- ✅ Détection automatique des données OCR
- ✅ Formatting source labels (Figure OCR, Table, Document Text, etc.)
- ✅ Confidence basée sur score top-1 chunk

### ✅ Frontend TypeScript

**Fichier** : [`gravis-app/src/lib/llm-synthesis.ts`](gravis-app/src/lib/llm-synthesis.ts)

1. **Types TypeScript** (lignes 7-34)
2. **Prompt Template** (ligne 36-50) - conforme au roadmap
3. **Fonction principale** :
   ```typescript
   export async function chatWithLlmSynthesis(
     sessionId: string,
     query: string,
     selection?: any | null,
     limit?: number | null
   ): Promise<LlmChatResponse>
   ```

**Flow complet** :
```
User Query
   ↓
chatWithLlmSynthesis()
   ↓
1. invoke("chat_with_llm_context") → Rust RAG
   ↓
2. LiteLLMClient.chat() → LLM synthesis
   ↓
3. Ajoute OCR warning si nécessaire
   ↓
LlmChatResponse (answer + sources + confidence)
```

---

## 📘 Comment l'utiliser dans DirectChatPage

### Option A : Remplacement simple (recommandé pour test)

**Fichier** : `gravis-app/src/hooks/useDirectChat.ts` ou directement dans `DirectChatPage.tsx`

```typescript
import { chatWithLlmSynthesis } from "@/lib/llm-synthesis";

// Dans votre fonction de chat existante, remplacer:
// const response = await invoke("chat_with_dropped_document", { request });

// Par:
const llmResponse = await chatWithLlmSynthesis(
  sessionId,
  userQuery,
  null,  // selection
  10     // limit
);

// Utiliser llmResponse.answer au lieu de response.response
console.log("LLM Answer:", llmResponse.answer);
console.log("Sources:", llmResponse.sources);
console.log("Confidence:", llmResponse.confidence);
```

### Option B : Toggle LLM ON/OFF (production)

```typescript
const [useLlmSynthesis, setUseLlmSynthesis] = useState(true);

const handleChat = async (query: string) => {
  if (useLlmSynthesis) {
    // Niveau 1: LLM Synthesis
    const llmResponse = await chatWithLlmSynthesis(sessionId, query);
    setMessages([...messages, {
      role: "assistant",
      content: llmResponse.answer,
      sources: llmResponse.sources,
      confidence: llmResponse.confidence,
    }]);
  } else {
    // Legacy: Chunks bruts
    const response = await invoke("chat_with_dropped_document", { request });
    setMessages([...messages, {
      role: "assistant",
      content: response.response,
    }]);
  }
};
```

### Option C : Comparaison A/B côte-à-côte

```typescript
const [showComparison, setShowComparison] = useState(false);

const handleChatWithComparison = async (query: string) => {
  // Appeler les deux en parallèle
  const [llmResponse, legacyResponse] = await Promise.all([
    chatWithLlmSynthesis(sessionId, query),
    invoke("chat_with_dropped_document", { request }),
  ]);

  setMessages([...messages, {
    role: "assistant",
    llm_answer: llmResponse.answer,
    legacy_answer: legacyResponse.response,
    show_comparison: showComparison,
  }]);
};
```

---

## 🧪 Tests Recommandés

### Test 1 : Query textuelle simple
```typescript
const query = "What is DeepSeek-OCR?";
const response = await chatWithLlmSynthesis(sessionId, query);

// Vérifier:
// - response.answer contient "DeepSeek"
// - response.sources.length > 0
// - response.confidence > 0.5
```

### Test 2 : Query avec données numériques
```typescript
const query = "Quelle est la précision à compression < 10x ?";
const response = await chatWithLlmSynthesis(sessionId, query);

// Vérifier:
// - response.answer contient des chiffres (ex: "96.5%", "6.7×")
// - response.sources contient des chunks FigureRegionText ou Table
// - response.has_ocr_warning === true
```

### Test 3 : Query sans réponse
```typescript
const query = "What is the meaning of life?";
const response = await chatWithLlmSynthesis(sessionId, query);

// Vérifier:
// - response.answer contient "ne contient pas" ou "not found"
// - response.confidence est bas
```

---

## 📊 Logs à surveiller

### Backend Rust
```
🤖 LLM Context Chat - session: xxx, query: 'your query'
🔍 Hybrid search in N chunks for session xxx
✅ Built LLM context from 10 chunks in 150ms (OCR: true)
```

### Frontend TypeScript
```
🔍 Fetching LLM context from Rust...
✅ Got context: 10 chunks, 8000 chars
🤖 Calling LLM with 9500 chars prompt (10 chunks × ~800 chars/chunk)
✅ LLM response: 450 chars, 1200 tokens
✅ LLM Synthesis complete: 2500ms total (RAG: 150ms, LLM: 2350ms)
```

---

## ⚙️ Configuration LLM

Assurez-vous que l'utilisateur a configuré un modèle dans le Model Selector :

1. Ouvrir le Model Selector (`cmd+shift+M` ou menu)
2. Sélectionner un modèle (ex: GPT-4o-mini, Claude 3.5 Haiku, Ollama local)
3. Configurer la connexion (LiteLLM, Modal, Ollama, etc.)

Le système utilise automatiquement `modelConfigStore.getConfig()` pour obtenir la config actuelle.

---

## 🐛 Troubleshooting

### Erreur : "LLM synthesis failed"
**Cause** : LLM non configuré ou API key invalide
**Solution** : Vérifier Model Selector et connexion active

### Erreur : "Search failed"
**Cause** : Session ID invalide ou expirée (TTL 2h)
**Solution** : Recharger le document ou vérifier que la session existe

### Latence trop élevée (> 5s)
**Cause** : Modèle trop gros ou trop de chunks
**Solution** :
- Utiliser modèle plus petit (Haiku, GPT-4o-mini)
- Réduire `limit` de 10 à 5 chunks
- Activer streaming (TODO Niveau 2)

### Réponses incohérentes
**Cause** : Contexte trop fragmenté ou chunks non pertinents
**Solution** :
- Vérifier logs RAG (scores des chunks)
- Considérer Niveau 2 (Query Rewriting) si queries en FR
- Ajuster prompt template si nécessaire

---

## 🚀 Next Steps

### Sprint 2 : Niveau 2 - Query Rewriting
- [ ] Détecter queries françaises
- [ ] Call LLM pour rewriting FR → EN
- [ ] Cache des rewrites
- [ ] Mesurer amélioration recall

### Sprint 3 : Niveau 3 - LLM Reranking (optionnel)
- [ ] Décider GO/NO-GO basé sur métriques N1+N2
- [ ] Implémenter si gap > 5%

---

## 📝 Checklist Intégration

- [ ] Importer `chatWithLlmSynthesis` dans DirectChatPage
- [ ] Tester avec 3 queries variées
- [ ] Vérifier affichage sources
- [ ] Vérifier avertissement OCR
- [ ] Mesurer latence (target < 3s P95)
- [ ] Collecter feedback utilisateur
- [ ] Logger métriques (search_time_ms, llm_time_ms, confidence)

---

**Auteur** : Claude (Assistant IA Anthropic)
**Version** : 1.0 - Sprint 1 Integration Guide
**Status** : ✅ Ready for Integration
