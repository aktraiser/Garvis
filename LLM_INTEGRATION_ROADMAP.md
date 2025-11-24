# LLM Integration Roadmap - GRAVIS RAG System

> **Date de création** : 20 novembre 2024
> **Dernière mise à jour** : 21 novembre 2024 (Post-Audit)
> **Objectif** : Intégrer le LLM existant pour améliorer la qualité des réponses
> **Contexte** : RAG Phase 3.6 implémenté (Vision-Aware + Digit-Aware + Hard Priority)
> **Status** : 🏗️ En cours d'implémentation (Sprint 1)

---

## 🎯 Vision Globale

**Principe** : Ne pas remplacer le RAG, mais l'**augmenter** avec le LLM pour :
1. ✅ Combiner intelligemment plusieurs chunks
2. ✅ Produire des réponses structurées et contextuelles
3. ✅ Gérer les questions complexes nécessitant synthèse
4. ✅ Améliorer le recall avec query rewriting (FR → EN surtout)

**Infrastructure disponible** :
- ✅ RAG solide (Vision-Aware + Digit-Aware + Hard Priority)
- ✅ Retrieval performant (hybrid search + numerical reranking)
- ⚠️ LLM à intégrer : API externe (OpenAI/Anthropic) ou modèle local via Candle

---

## 📝 Review Technique - Points Clés

### ✅ Ce qui est très solide

1. **Architecture globale** : Les 3 niveaux sont bien séparés et le principe "augmenter, pas remplacer" est respecté
2. **Structs** : `LlmChatResponse`, `SourceRef`, `LlmResponseMetadata` sont bien pensés pour l'explainability
3. **Prompts** : Templates clairs avec instructions strictes ("ne réponds que depuis le document")
4. **Observabilité** : Logs structurés et métriques détaillées pensés dès le départ

### ⚠️ Simplifications Recommandées

1. **Troncature contexte** (Niveau 1)
   - ✅ Ajouter `.take(800)` sur `chunk.content` → évite overflow tokens
   - 📊 10 chunks × 800 chars = ~2000 tokens max

2. **Confidence** (Niveau 1)
   - ✅ Simplifier : `confidence = chunks.first().map(|c| c.score).unwrap_or(0.0)`
   - ❌ Éviter : Calculs complexes de "vrai LLM confidence"

3. **Page index** (Niveau 1)
   - ⚠️ Utiliser `chunk.page_index` réel si disponible
   - 🔧 `start_line` est un placeholder temporaire

4. **Cache API** (Niveau 2)
   - ✅ API séparée : `cache.get()` / `cache.set()`
   - ❌ Éviter : Closure `get_or_rewrite()` complexe

5. **Politique de rewriting** (Niveau 2)
   - 💡 Vu le use case (docs EN, queries FR), considérer rewrite systématique
   - 🎯 Garder heuristiques surtout pour éviter double call LLM

6. **Niveau 3 : Reranking**
   - 🟢 Priorité BASSE - commencer sans
   - 📉 Activer uniquement sur "mode expert" ou queries difficiles
   - ⚖️ Mesurer coût/bénéfice avant déploiement

### 🎯 Ordre d'Implémentation Recommandé

** (1-2 jours)
- Focus : `llm_answer_with_context()` avec troncature
- Objectif : Réponses synthétisées au lieu de chunks brutsSprint 1 : Niveau 1 UNIQUEMENT**
- Impact : **Massif** - expérience utilisateur transformée

**Sprint 2 : Niveau 2** (1 jour)
- Focus : FR → EN rewriting
- Objectif : Améliorer recall sur docs anglais
- Impact : **Important** - queries quotidiennes

**Sprint 3 : Décision Niveau 3** (si nécessaire)
- Évaluer : Le RAG + N1 + N2 suffisent-ils ?
- Activer : Seulement si gap mesurable
- Impact : **Marginal** - cas edge complexes

---

## 📊 Trois Niveaux d'Intégration

### Niveau 1 : LLM Response Generation (PRIORITÉ HAUTE) 🔴

**Impact** : Immédiat et massif
**Complexité** : Faible
**Timeline** : 1-2 jours

#### Concept

```
User Query → RAG Retrieval → Top-K Chunks → LLM Synthesis → Structured Answer
```

**Ce qui change** :
- ❌ AVANT : Retourner les chunks bruts
- ✅ APRÈS : Synthèse LLM avec citations

#### Architecture

```rust
// Dans DirectChatManager
pub async fn chat(
    &self,
    session_id: &str,
    query: &str,
) -> Result<LlmResponse> {
    // 1. RAG retrieval (existant)
    let chunks = self.search_in_session(session_id, query, Some(10)).await?;

    // 2. ⭐ NOUVEAU : LLM synthesis
    let llm_answer = self.llm_answer_with_context(query, &chunks).await?;

    Ok(llm_answer)
}

struct LlmResponse {
    answer: String,              // Réponse synthétisée
    sources: Vec<SourceRef>,     // Références aux chunks utilisés
    confidence: f32,             // Confiance du LLM
    has_numeric_data: bool,      // Si réponse contient données chiffrées
}

struct SourceRef {
    chunk_id: String,
    excerpt: String,            // Extrait pertinent
    page: Option<u32>,
    figure_id: Option<String>,  // Si source = figure
    relevance: f32,             // Score d'utilisation par le LLM
}
```

#### Prompt Template

```rust
const LLM_ANSWER_PROMPT: &str = r#"Tu es un assistant qui répond UNIQUEMENT à partir du document fourni.

RÈGLES STRICTES :
1. Si le document ne contient pas la réponse, dis-le explicitement
2. Cite les sources utilisées (ex: "Selon la Figure 3...")
3. Pour les données chiffrées, CITE la source exacte et ajoute un avertissement si OCR
4. Réponds en français, de manière concise et précise
5. Structure ta réponse avec des bullet points si plusieurs informations

DOCUMENT :
{context}

QUESTION : {question}

RÉPONSE (en français) :"#;

fn build_context_string(chunks: &[ScoredChunk]) -> String {
    chunks.iter()
        .enumerate()
        .map(|(i, chunk)| {
            let source_label = match chunk.chunk.chunk_source {
                ChunkSource::FigureCaption => format!("Figure Caption - {}",
                    chunk.chunk.figure_id.as_deref().unwrap_or("Unknown")),
                ChunkSource::FigureRegionText => format!("Figure OCR - {}",
                    chunk.chunk.figure_id.as_deref().unwrap_or("Unknown")),
                ChunkSource::Table => "Table",
                _ => "Document Text"
            };

            // ⚠️ IMPORTANT: Tronquer le contenu pour éviter token overflow
            // 800 chars ~= 200 tokens, 10 chunks = ~2000 tokens max
            let content = chunk.chunk.content
                .chars()
                .take(800)
                .collect::<String>();

            format!(
                "### Source {} - {} (Page {}, Confidence: {:.0}%)\n{}\n",
                i + 1,
                source_label,
                chunk.chunk.start_line, // TODO: Utiliser vrai page_index si disponible
                chunk.chunk.metadata.confidence * 100.0,
                content
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n")
}
```

**⚠️ Points d'attention (review technique)** :
1. **Troncature à 800 chars** : Évite token overflow pour 10 chunks (~2000 tokens context)
2. **Page number** : Utiliser `chunk.page_index` réel quand disponible au lieu de `start_line`
3. **Confidence** : Simplifier en remontant le score du top-1 chunk plutôt que calcul complexe

#### Gestion des Avertissements OCR

```rust
fn should_add_ocr_warning(chunks: &[ScoredChunk]) -> bool {
    chunks.iter().any(|c|
        matches!(c.chunk.chunk_source, ChunkSource::FigureRegionText)
    )
}

fn build_llm_response(
    answer: String,
    chunks: &[ScoredChunk],
) -> LlmResponse {
    let has_ocr = should_add_ocr_warning(chunks);

    let final_answer = if has_ocr {
        format!(
            "{}\n\n⚠️ Note: Cette réponse contient des données extraites par OCR. \
            Vérifiez visuellement dans le document pour les valeurs exactes.",
            answer
        )
    } else {
        answer
    };

    // Confidence simplifiée : score du top-1 chunk
    let confidence = chunks.first()
        .map(|c| c.score)
        .unwrap_or(0.0);

    LlmResponse {
        answer: final_answer,
        sources: extract_sources(chunks),
        confidence, // Simple et efficace
        has_numeric_data: contains_numeric_data(&answer),
    }
}

fn extract_sources(chunks: &[ScoredChunk]) -> Vec<SourceRef> {
    chunks.iter().map(|chunk| {
        SourceRef {
            chunk_id: chunk.chunk.id.clone(),
            excerpt: chunk.chunk.content.chars().take(150).collect(),
            page: None, // TODO: Utiliser page_index réel
            figure_id: chunk.chunk.figure_id.clone(),
            source_type: chunk.chunk.chunk_source.clone(),
            confidence: chunk.chunk.metadata.confidence,
        }
    }).collect()
}
```

#### Logging

```rust
info!("💬 LLM synthesis for query: '{}'", query);
info!("📚 Using {} chunks as context", chunks.len());
info!("✅ LLM response generated (confidence: {:.0}%)", response.confidence * 100.0);

if response.has_numeric_data {
    info!("🔢 Response contains numerical data");
}
```

#### Checklist Implémentation Niveau 1

- [ ] Créer `LlmResponse` et `SourceRef` structs
- [ ] Impl `build_context_string()` avec formatting par source
- [ ] Impl `llm_answer_with_context()` avec prompt template
- [ ] 🏗️ **CRITIQUE** : Créer le module `crate::llm` (inexistant)
- [ ] Wrapper autour de l'API LLM (Provider à définir : OpenAI/Anthropic/Local)
- [ ] Gestion des erreurs LLM (timeout, rate limit, etc.)
- [ ] Tests avec queries réelles :
  - [ ] Query textuelle simple
  - [ ] Query avec données numériques
  - [ ] Query nécessitant synthèse de plusieurs chunks
  - [ ] Query sans réponse dans le document
- [ ] Logging structuré
- [ ] Métriques (latency, token usage)

---

### Niveau 2 : Query Rewriting (PRIORITÉ MOYENNE) 🟡

**Impact** : Améliore le recall, surtout FR → EN
**Complexité** : Faible
**Timeline** : 1 jour

#### Concept

```
User Query (FR) → LLM Rewrite → Optimized Query (EN) → RAG → Top-K → LLM Answer (FR)
```

**Cas d'usage** :
- Query en français alors que le document est en anglais
- Query verbeuse → Query courte et technique
- Query ambiguë → Query précise

#### Architecture

```rust
pub async fn chat(
    &self,
    session_id: &str,
    query: &str,
) -> Result<LlmResponse> {
    // 1. ⭐ NOUVEAU : Query rewriting
    let (rewritten_query, should_rewrite) = self.maybe_rewrite_query(query).await?;

    info!("🔄 Original: '{}' | Rewritten: '{}'", query, rewritten_query);

    // 2. RAG retrieval avec query optimisée
    let chunks = self.search_in_session(
        session_id,
        &rewritten_query,  // ← Query réécrite
        Some(10)
    ).await?;

    // 3. LLM synthesis avec query ORIGINALE
    let llm_answer = self.llm_answer_with_context(
        query,  // ← Question FR d'origine pour la réponse
        &chunks
    ).await?;

    Ok(llm_answer)
}
```

#### Détection Automatique du Besoin de Rewriting

```rust
fn should_rewrite_query(query: &str) -> bool {
    // 1. Détection langue (FR → EN)
    let is_french = detect_french(query);

    // 2. Query trop longue (> 15 mots)
    let word_count = query.split_whitespace().count();
    let is_verbose = word_count > 15;

    // 3. Présence de mots "conversationnels"
    let conversational_patterns = [
        "peux-tu", "pourrais-tu", "j'aimerais savoir",
        "dis-moi", "explique-moi", "selon toi"
    ];
    let is_conversational = conversational_patterns.iter()
        .any(|p| query.to_lowercase().contains(p));

    is_french || is_verbose || is_conversational
}
```

#### Prompt Template

```rust
const QUERY_REWRITE_PROMPT: &str = r#"Tu réécris des questions pour optimiser la recherche dans un article scientifique en anglais.

INSTRUCTIONS :
1. Traduis en anglais si nécessaire
2. Rends la question plus courte et directe
3. Garde TOUS les termes techniques et nombres
4. Transforme en keywords si c'est une question ouverte
5. Ne réponds PAS à la question, réécris-la SEULEMENT

EXEMPLES :

Original: "Quelle idée le document propose-t-il pour gérer les contextes longs ?"
Réécrit: "How does the paper handle long-context compression?"

Original: "Peux-tu me dire quelle est la précision de décodage à compression inférieur à 10x ?"
Réécrit: "decoding accuracy compression < 10x"

Original: "Selon le document, quelle est la capacité de production de données ?"
Réécrit: "data generation capacity production"

QUESTION À RÉÉCRIRE :
{query}

QUESTION RÉÉCRITE (en anglais, courte, technique) :"#;
```

#### Gestion du Cache

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct QueryRewriteCache {
    cache: Arc<RwLock<HashMap<String, String>>>,
    max_size: usize,
}

impl QueryRewriteCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub async fn get(&self, query: &str) -> Option<String> {
        let cache = self.cache.read().await;
        cache.get(query).cloned()
    }

    pub async fn set(&self, query: &str, rewritten: &str) {
        let mut cache = self.cache.write().await;

        // Simple eviction: clear tout si dépassement
        // ⚠️ TODO: Remplacer par vrai LRU si nécessaire
        if cache.len() >= self.max_size {
            warn!("Query rewrite cache full, clearing {} entries", cache.len());
            cache.clear();
        }

        cache.insert(query.to_string(), rewritten.to_string());
    }
}
```

**⚠️ Point d'attention (review)** : API simplifiée avec `get()` / `set()` séparés, évite la closure complexe.

#### Checklist Implémentation Niveau 2

- [ ] Créer `should_rewrite_query()` avec heuristiques
- [ ] Impl `llm_rewrite_query()` avec prompt template
- [ ] Cache pour éviter rewrites répétés
- [ ] Logging des transformations
- [ ] Tests A/B :
  - [ ] Comparer recall AVANT/APRÈS rewriting
  - [ ] Mesurer impact sur queries FR
  - [ ] Vérifier que les termes techniques sont préservés
- [ ] Métriques (% queries rewritten, avg improvement)

---

### Niveau 3 : LLM Reranking (PRIORITÉ BASSE) 🟢

**Impact** : Amélioration marginale sur cas edge
**Complexité** : Moyenne
**Timeline** : 2 jours

#### Concept

```
RAG Retrieval → Top-20 → LLM Rerank → Top-10 → LLM Synthesis
```

**Cas d'usage** :
- Query très complexe nécessitant compréhension sémantique profonde
- Chunks similaires en score mais différents en pertinence réelle
- Fallback si hard priority ne suffit pas

#### Architecture

```rust
pub async fn chat(
    &self,
    session_id: &str,
    query: &str,
) -> Result<LlmResponse> {
    // 1. Query rewriting (optionnel)
    let rewritten = self.maybe_rewrite_query(query).await?;

    // 2. RAG retrieval (top-20 au lieu de 10)
    let chunks = self.search_in_session(session_id, &rewritten, Some(20)).await?;

    // 3. ⭐ NOUVEAU : LLM reranking
    let reranked = self.llm_rerank_chunks(query, chunks).await?;

    // Prendre top-10 après reranking
    let top_chunks = reranked.into_iter().take(10).collect::<Vec<_>>();

    // 4. LLM synthesis
    let llm_answer = self.llm_answer_with_context(query, &top_chunks).await?;

    Ok(llm_answer)
}
```

#### Prompt Template

```rust
const LLM_RERANK_PROMPT: &str = r#"Tu es un système de reranking. On te donne une question et plusieurs extraits d'un document.

TON RÔLE :
Classe ces extraits du PLUS pertinent au MOINS pertinent pour répondre à la question.

CRITÈRES DE PERTINENCE :
1. L'extrait répond DIRECTEMENT à la question
2. L'extrait contient des données factuelles (chiffres, noms, dates)
3. L'extrait est spécifique (pas générique/introduction)

FORMAT DE RÉPONSE :
Réponds STRICTEMENT en JSON avec un tableau d'indices (1-based).
Exemple: [3, 1, 5, 2, 4]

QUESTION :
{question}

EXTRAITS :
{chunks}

RÉPONSE (JSON uniquement) :"#;

fn build_chunks_for_rerank(chunks: &[ScoredChunk]) -> String {
    chunks.iter()
        .enumerate()
        .map(|(i, chunk)| {
            format!(
                "[{}] {}\n",
                i + 1,
                chunk.chunk.content.chars().take(300).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

#### Parsing de la Réponse LLM

```rust
fn parse_rerank_response(response: &str) -> Result<Vec<usize>> {
    // Parse JSON: [3, 1, 5, 2, 4]
    let indices: Vec<usize> = serde_json::from_str(response)
        .map_err(|e| format!("Failed to parse rerank response: {}", e))?;

    // Validation
    if indices.is_empty() {
        return Err("Empty rerank indices".into());
    }

    Ok(indices)
}

fn apply_reranking(
    chunks: Vec<ScoredChunk>,
    indices: &[usize],
) -> Vec<ScoredChunk> {
    let mut reranked = Vec::new();

    for &idx in indices {
        if idx > 0 && idx <= chunks.len() {
            reranked.push(chunks[idx - 1].clone());
        }
    }

    // Fallback: si parsing échoue, retourner ordre original
    if reranked.is_empty() {
        return chunks;
    }

    reranked
}
```

#### Checklist Implémentation Niveau 3

- [ ] Impl `llm_rerank_chunks()` avec prompt JSON
- [ ] Parser robuste pour réponse LLM
- [ ] Fallback si parsing échoue (garder ordre RAG)
- [ ] Logging comparatif (ordre AVANT/APRÈS)
- [ ] Tests :
  - [ ] Query complexe nécessitant reranking
  - [ ] Vérifier que top-1 change effectivement
  - [ ] Mesurer coût (tokens, latency)
- [ ] Décision : activer seulement si query "hard" ?

---

## 🏗️ Implémentation Complète : DirectChatManager Annoté

### Flow Complet avec LLM

```rust
// src/rag/core/direct_chat_manager.rs

use crate::llm::{LlmClient, LlmRequest, LlmResponse};

pub struct DirectChatManager {
    sessions: Arc<RwLock<HashMap<String, DirectChatSession>>>,
    ttl_seconds: u64,
    llm_client: LlmClient,  // ⭐ NOUVEAU
    query_cache: QueryRewriteCache,  // ⭐ NOUVEAU
}

impl DirectChatManager {
    /// Chat avec intégration LLM complète (Niveau 1 + 2 + 3)
    pub async fn chat(
        &self,
        session_id: &str,
        user_query: &str,
    ) -> DirectChatResult<LlmChatResponse> {
        info!("💬 Chat request: '{}'", user_query);

        // ========== PHASE 1: QUERY REWRITING (Niveau 2) ==========

        let should_rewrite = should_rewrite_query(user_query);

        let search_query = if should_rewrite {
            let rewritten = self.llm_rewrite_query(user_query).await?;
            info!("🔄 Query rewritten: '{}' → '{}'", user_query, rewritten);
            rewritten
        } else {
            user_query.to_string()
        };

        // ========== PHASE 2: RAG RETRIEVAL ==========

        let retrieval_limit = 20;  // Plus pour permettre reranking

        let chunks = self.search_in_session(
            session_id,
            &search_query,
            Some(retrieval_limit),
        ).await?;

        info!("📚 Retrieved {} chunks", chunks.len());

        // ========== PHASE 3: LLM RERANKING (Niveau 3 - optionnel) ==========

        let reranked_chunks = if self.should_llm_rerank(user_query, &chunks) {
            info!("🔀 Applying LLM reranking");
            self.llm_rerank_chunks(user_query, chunks).await?
        } else {
            chunks
        };

        // Prendre top-10 après reranking
        let top_chunks: Vec<_> = reranked_chunks.into_iter().take(10).collect();

        // ========== PHASE 4: LLM SYNTHESIS (Niveau 1) ==========

        info!("🤖 Generating LLM response from {} chunks", top_chunks.len());

        let llm_response = self.llm_answer_with_context(
            user_query,  // Question originale (FR)
            &top_chunks,
        ).await?;

        info!("✅ LLM response generated (confidence: {:.0}%)",
              llm_response.confidence * 100.0);

        Ok(llm_response)
    }

    /// Niveau 1 : LLM synthesis avec context
    async fn llm_answer_with_context(
        &self,
        query: &str,
        chunks: &[ScoredChunk],
    ) -> DirectChatResult<LlmChatResponse> {
        // Build context string
        let context = build_context_string(chunks);

        // Build prompt
        let prompt = LLM_ANSWER_PROMPT
            .replace("{context}", &context)
            .replace("{question}", query);

        // Call LLM
        let llm_response = self.llm_client.complete(LlmRequest {
            prompt,
            max_tokens: 1000,
            temperature: 0.3,  // Faible pour rester factuel
        }).await?;

        // Parse response
        let answer = llm_response.text;

        // Build structured response
        Ok(build_llm_response(answer, chunks))
    }

    /// Niveau 2 : Query rewriting
    async fn llm_rewrite_query(&self, query: &str) -> DirectChatResult<String> {
        // Check cache
        if let Some(cached) = self.query_cache.get(query).await {
            debug!("🎯 Cache hit for query rewrite");
            return Ok(cached);
        }

        // Build prompt
        let prompt = QUERY_REWRITE_PROMPT.replace("{query}", query);

        // Call LLM
        let response = self.llm_client.complete(LlmRequest {
            prompt,
            max_tokens: 100,
            temperature: 0.2,
        }).await?;

        let rewritten = response.text.trim().to_string();

        // Cache result
        self.query_cache.set(query, &rewritten).await;

        Ok(rewritten)
    }

    /// Niveau 3 : LLM reranking
    async fn llm_rerank_chunks(
        &self,
        query: &str,
        chunks: Vec<ScoredChunk>,
    ) -> DirectChatResult<Vec<ScoredChunk>> {
        if chunks.is_empty() {
            return Ok(chunks);
        }

        // Build prompt
        let chunks_text = build_chunks_for_rerank(&chunks);
        let prompt = LLM_RERANK_PROMPT
            .replace("{question}", query)
            .replace("{chunks}", &chunks_text);

        // Call LLM
        let response = self.llm_client.complete(LlmRequest {
            prompt,
            max_tokens: 200,
            temperature: 0.1,
        }).await?;

        // Parse indices
        let indices = parse_rerank_response(&response.text)
            .unwrap_or_else(|e| {
                warn!("Failed to parse rerank response: {}", e);
                (1..=chunks.len()).collect()  // Fallback: ordre original
            });

        // Apply reranking
        Ok(apply_reranking(chunks, &indices))
    }

    /// Décider si LLM reranking est nécessaire
    fn should_llm_rerank(&self, query: &str, chunks: &[ScoredChunk]) -> bool {
        // Heuristique : seulement si query complexe OU scores très proches

        let is_complex_query = query.split_whitespace().count() > 10;

        let has_close_scores = if chunks.len() >= 2 {
            let score_diff = chunks[0].score - chunks[1].score;
            score_diff < 0.1  // Scores très proches
        } else {
            false
        };

        is_complex_query || has_close_scores
    }
}

/// Struct pour réponse LLM enrichie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatResponse {
    pub answer: String,
    pub sources: Vec<SourceRef>,
    pub confidence: f32,
    pub has_numeric_data: bool,
    pub has_ocr_warning: bool,

    // Metadata pour debugging
    pub metadata: LlmResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponseMetadata {
    pub query_rewritten: bool,
    pub rewritten_query: Option<String>,
    pub llm_reranked: bool,
    pub chunks_used: usize,
    pub total_tokens: usize,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub chunk_id: String,
    pub excerpt: String,
    pub page: Option<u32>,
    pub figure_id: Option<String>,
    pub source_type: ChunkSource,
    pub confidence: f32,
}
```

---

## 📊 Métriques et Observabilité

### Logging Structuré

```rust
info!("💬 LLM Chat Pipeline Started");
info!("  Query: '{}'", query);
info!("  Session: {}", session_id);

// Phase 1: Rewriting
if rewritten {
    info!("🔄 Query Rewriting");
    info!("  Original: '{}'", original);
    info!("  Rewritten: '{}'", rewritten);
    info!("  Cache hit: {}", cache_hit);
}

// Phase 2: Retrieval
info!("🔍 RAG Retrieval");
info!("  Query kind: {:?}", query_kind);
info!("  Chunks retrieved: {}", chunks.len());
info!("  Top score: {:.3}", top_score);

// Phase 3: Reranking
if llm_reranked {
    info!("🔀 LLM Reranking");
    info!("  Top chunk BEFORE: {}", before_top_id);
    info!("  Top chunk AFTER: {}", after_top_id);
    info!("  Order changed: {}", changed);
}

// Phase 4: Synthesis
info!("🤖 LLM Synthesis");
info!("  Chunks used: {}", chunks_used);
info!("  Response length: {} chars", response.len());
info!("  Confidence: {:.0}%", confidence * 100.0);
info!("  Has numeric data: {}", has_numeric);
info!("  Has OCR warning: {}", has_ocr);

// Metrics
info!("📈 Pipeline Metrics");
info!("  Total latency: {}ms", total_latency);
info!("  RAG latency: {}ms", rag_latency);
info!("  LLM latency: {}ms", llm_latency);
info!("  Tokens used: {}", tokens);
```

### Métriques à Tracker

```rust
struct LlmPipelineMetrics {
    // Latency
    query_rewrite_latency_ms: u64,
    rag_retrieval_latency_ms: u64,
    llm_rerank_latency_ms: u64,
    llm_synthesis_latency_ms: u64,
    total_latency_ms: u64,

    // Token usage
    rewrite_tokens: usize,
    rerank_tokens: usize,
    synthesis_tokens: usize,
    total_tokens: usize,

    // Quality
    confidence: f32,
    chunks_used: usize,
    sources_cited: usize,

    // Flags
    query_was_rewritten: bool,
    llm_rerank_applied: bool,
    has_ocr_data: bool,
}
```

---

## 🧪 Tests et Validation

### Test Suite Niveau 1

```rust
#[tokio::test]
async fn test_llm_synthesis_simple_query() {
    let query = "What is DeepSeek-OCR?";
    let chunks = mock_chunks_about_deepseek();

    let response = llm_answer_with_context(query, &chunks).await.unwrap();

    assert!(response.answer.contains("DeepSeek"));
    assert!(response.sources.len() > 0);
    assert!(response.confidence > 0.7);
}

#[tokio::test]
async fn test_llm_synthesis_no_answer() {
    let query = "What is the meaning of life?";
    let chunks = mock_chunks_about_deepseek();

    let response = llm_answer_with_context(query, &chunks).await.unwrap();

    // Should indicate no answer in document
    assert!(
        response.answer.to_lowercase().contains("ne contient pas") ||
        response.answer.to_lowercase().contains("not found")
    );
}

#[tokio::test]
async fn test_llm_synthesis_numeric_data() {
    let query = "Quelle précision pour compression < 10x ?";
    let chunks = mock_chunks_with_table2();

    let response = llm_answer_with_context(query, &chunks).await.unwrap();

    assert!(response.has_numeric_data);
    assert!(response.answer.contains("96.5%") || response.answer.contains("6.7×"));
}

#[tokio::test]
async fn test_llm_ocr_warning() {
    let query = "Données de Table 2";
    let chunks = vec![mock_ocr_chunk()];

    let response = llm_answer_with_context(query, &chunks).await.unwrap();

    assert!(response.has_ocr_warning);
    assert!(response.answer.contains("⚠️") || response.answer.contains("OCR"));
}
```

### Test Suite Niveau 2

```rust
#[tokio::test]
async fn test_query_rewrite_french_to_english() {
    let query = "Quelle est la capacité de production de données ?";

    let rewritten = llm_rewrite_query(query).await.unwrap();

    // Should be in English
    assert!(!detect_french(&rewritten));
    assert!(rewritten.len() < query.len());  // Shorter
}

#[tokio::test]
async fn test_query_rewrite_preserves_numbers() {
    let query = "Précision à compression inférieur à 10x ?";

    let rewritten = llm_rewrite_query(query).await.unwrap();

    assert!(rewritten.contains("10"));
    assert!(rewritten.contains("x") || rewritten.contains("×"));
}

#[tokio::test]
async fn test_query_rewrite_cache() {
    let query = "Test query";

    let start = Instant::now();
    let first = llm_rewrite_query(query).await.unwrap();
    let first_latency = start.elapsed();

    let start = Instant::now();
    let second = llm_rewrite_query(query).await.unwrap();
    let second_latency = start.elapsed();

    assert_eq!(first, second);
    assert!(second_latency < first_latency / 2);  // Cache should be faster
}
```

### Tests A/B Complets

```rust
#[tokio::test]
async fn test_ab_with_vs_without_llm() {
    let test_queries = vec![
        "What is DeepSeek-OCR?",
        "Précision à compression < 10x ?",
        "Capacité de production de données",
    ];

    for query in test_queries {
        // WITHOUT LLM (RAG only)
        let rag_only = search_in_session(session_id, query, Some(10)).await.unwrap();

        // WITH LLM (full pipeline)
        let llm_response = chat(session_id, query).await.unwrap();

        // Compare
        println!("\n=== Query: {} ===", query);
        println!("RAG only: {} chunks", rag_only.len());
        println!("LLM response: {}", llm_response.answer);
        println!("Confidence: {:.0}%", llm_response.confidence * 100.0);
    }
}
```

---

## 📋 Ordre de Priorité d'Implémentation

### Sprint 1 : Niveau 1 (LLM Synthesis) - ✅ IMPLÉMENTÉ + ⚠️ AUDIT CRITIQUE (22 Nov 2024)

**Objectif Original** : Réponses structurées au lieu de chunks bruts

#### ✅ Ce qui a été implémenté (Niveau 1 - conforme roadmap)

**Tasks Niveau 1** :
1. ✅ Créer structs `LlmContextResponse`, `LlmChunkInfo` (Rust)
2. ✅ Impl `build_llm_context()` avec formatting par source type + troncature 800 chars
3. ✅ Impl commande Tauri `chat_with_llm_context`
4. ✅ Wrapper frontend `chatWithLlmSynthesis()` avec prompt template
5. ✅ Gestion erreurs LLM (try/catch + logs)
6. ✅ Logging structuré (Rust + TS)
7. ✅ Détection OCR automatique + warning
8. ✅ Métriques (search_time_ms, llm_time_ms, confidence)

**Fichiers créés/modifiés (Niveau 1)** :
- ✅ `gravis-app/src-tauri/src/rag/direct_chat_commands.rs` (~450 lignes total)
- ✅ `gravis-app/src-tauri/src/lib.rs` (commande exposée)
- ✅ `gravis-app/src/lib/llm-synthesis.ts` (nouveau fichier, 207 lignes)
- ✅ `gravis-app/src/hooks/useDirectChat.ts` (intégré avec limit=7)
- ✅ `SPRINT1_INTEGRATION_GUIDE.md` (guide complet)

**Architecture choisie** :
- ✅ Rust backend retourne contexte formaté via `chat_with_llm_context`
- ✅ Frontend TypeScript appelle LLM via `LiteLLMClient` existant
- ✅ Réutilise infrastructure LLM déjà configurée (Model Selector)

#### ⚠️ DÉVIATIONS NON PLANIFIÉES - Sprint 1 "Niveau 1.5" (ajouté itérativement)

**❌ PROBLÈME : Sur-complexification du pipeline pour fixer 1 query test**

**Ajouts hors-roadmap** :
1. **Query-Aware Reranker** (`src-tauri/src/rag/search/query_aware_reranker.rs`, 274 lignes)
   - ❌ **NON planifié** dans roadmap original
   - Détection query type hardcodée (Goal/Method/Result/General)
   - 30+ marqueurs hardcodés ("objectif", "but", "goal", "SAM", "CLIP", etc.)
   - Pénalités/boosts heuristiques (benchmark noise -0.7, Abstract +0.5, etc.)
   - **RISQUE** : Sur-spécialisé pour queries "objectif", peut dégrader autres types

2. **Pipeline Reranking + Filtres 3-Pass** (dans `chat_with_llm_context`)
   - ❌ **NON planifié** — complexité ajoutée pour fix contamination
   - Phase 1: RAG retrieval (top-20 au lieu de top-10)
   - Phase 1.5: Query-aware reranking (20 → 10)
   - Phase 2: Filtre 3-pass (visual contamination, adaptive threshold, lexical overlap)
   - Phase 3: Top-7 final
   - **RISQUE** : 4 étapes de filtering, difficile à debugger, comportement imprévisible

3. **Prompt LLM sur-spécialisé** (`llm-synthesis.ts`)
   - ❌ Prompt original simple devenu trop prescriptif
   - Section "STRATEGIC VS TECHNICAL" (WHY vs HOW) — heuristique rigide
   - Instructions spécifiques "if objective query → answer WHY not HOW"
   - **RISQUE** : Bride le LLM au lieu de le guider, pas générique

**Fichiers supplémentaires créés (hors roadmap)** :
- ⚠️ `gravis-app/src-tauri/src/rag/search/query_aware_reranker.rs` (274 lignes)
- ⚠️ Modifications `direct_chat_commands.rs` (+300 lignes de filtres)

#### 🐛 PROBLÈMES IDENTIFIÉS (Audit Utilisateur 22 Nov 2024)

**Citation utilisateur** :
> "je pense qu'on fait de plus en plus de spécifique et pas assez de générique, on enferme la logique que pour cette question, ça ne marchera pas si on pose une autre question"

**Analyse** :

1. **Sur-spécialisation du Reranker**
   - ❌ 30+ marqueurs hardcodés optimisés pour query "Quel est l'objectif principal"
   - ❌ Pénalités agressives ("SAM", "CLIP", "benchmark") peuvent virer chunks pertinents
   - ❌ Boost massif "Abstract" (+0.5) peut dominer le score original (risque faux positifs)
   - ❌ Pas de validation sur queries variées

2. **Pipeline Trop Complexe (4 étapes)**
   - ❌ Impossible de savoir quelle étape cause problème
   - ❌ Logs insuffisants pour tracer décisions de filtering
   - ❌ Chaque étape peut introduire biais différent

3. **Prompt LLM Devenu Prescriptif**
   - ❌ "If question asks objective → answer WHY not HOW" = heuristique rigide
   - ❌ Risque de brider le LLM sur queries ambiguës
   - ❌ Pas testé sur queries hors "objectif"

4. **Méthodologie de Test Défaillante**
   - ❌ Optimisation basée sur **1 seule query** ("objectif principal DeepSeek-OCR")
   - ❌ Pas de test suite avec 10-15 queries variées
   - ❌ Pas de métriques quantitatives (recall@7, precision)
   - ❌ Changements itératifs sans validation systématique

5. **Résultats Toujours Insuffisants**
   - ❌ Après 3 itérations de "fixes", toujours du bruit (Table 3, benchmarks)
   - ❌ Score top-1 seulement 69% (devrait être 90%+ si chunks pertinents)
   - ❌ LLM synthesis latency 20s+ (problème infrastructure LLM?)

#### 🎯 ACTIONS RECOMMANDÉES (Post-Audit)

**PRIORITÉ 1 : RETOUR AUX BASES - Validation A/B**
- [ ] **Désactiver** query-aware reranker → tester pipeline RAG vanilla
- [ ] **Désactiver** filtres 3-pass → tester pipeline simple (RAG → top-10 → LLM)
- [ ] **Simplifier** prompt LLM → retour version originale sans WHY/HOW
- [ ] Comparer qualité réponses AVEC vs SANS chaque composant
- [ ] **Objectif** : Identifier quel composant aide vraiment vs. ajoute du bruit

**PRIORITÉ 2 : TEST SUITE SYSTÉMATIQUE**
- [ ] Créer 10-15 queries variées :
  - 3 queries "objectif/but" (goal)
  - 3 queries "méthode/architecture" (how)
  - 3 queries "résultats/performance" (results)
  - 3 queries factuelles simples ("What is X?")
- [ ] Pour chaque query, mesurer :
  - Recall@7 (chunks pertinents dans top-7)
  - Qualité réponse LLM (score 1-5)
  - Présence de contamination (oui/non)
- [ ] Comparer métriques pipeline simple vs. pipeline complexe

**PRIORITÉ 3 : DÉCISION GO/NO-GO par Composant**
- [ ] Si reranker **n'améliore pas** recall moyen > +5% → **RETIRER**
- [ ] Si filtres 3-pass **réduisent** recall (faux négatifs) → **SIMPLIFIER ou RETIRER**
- [ ] Si prompt prescriptif **dégrade** qualité sur queries variées → **REVENIR version simple**

**PRIORITÉ 4 : Si Nécessaire, Fix Infrastructure**
- [ ] Investiguer pourquoi LLM synthesis prend 20s+ (Modal latency? Model trop gros?)
- [ ] Vérifier si le problème vient du RAG (chunks bruits) ou du LLM (mauvaise synthèse)
- [ ] Possiblement tester avec LLM plus rapide (Mistral 7B local via Ollama?)

**Validation (mise à jour)** :
- ✅ Compilation Rust réussie
- ✅ Types TypeScript créés
- ⚠️ User teste avec **1 seule query** (insuffisant)
- ❌ Réponses > 80% pertinence subjective (**pas mesuré** sur test suite)
- ❌ Latency < 3s P95 (actuel: **21s+**, dont 20s LLM)
- ❌ Sources contiennent **encore du bruit** (Table 3, benchmarks)

**Status** : ✅ Niveau 1 fonctionnel mais ⚠️ **PIPELINE SUR-COMPLEXIFIÉ** — **BESOIN AUDIT/SIMPLIFICATION URGENTE**

**Conclusion Audit** :
> Le Niveau 1 fonctionne techniquement mais a été pollué par des optimisations prématurées basées sur 1 seule query. Le pipeline est devenu fragile et non générique. **Recommandation : Retour pipeline simple + test suite systématique avant d'ajouter toute optimisation.**

### Sprint 2 : Niveau 2 (Query Rewriting) - IMPORTANT

**Objectif** : Améliorer recall sur queries FR ou verboses

**Tasks** :
1. ✅ Impl `should_rewrite_query()` avec heuristiques
2. ✅ Impl `llm_rewrite_query()` avec prompt
3. ✅ Cache système (HashMap + RwLock)
4. ✅ Logging transformations
5. ✅ Tests A/B (recall AVANT/APRÈS)
6. ✅ Métriques (% queries rewritten, avg score improvement)

**Validation** :
- [ ] Queries FR → EN fonctionnent
- [ ] Recall amélioration mesurée > 10%
- [ ] Cache hit rate > 50% en production

### Sprint 3 : Niveau 3 (LLM Reranking) - OPTIONNEL

**Objectif** : Peaufiner pour cas edge très complexes

**Tasks** :
1. ✅ Impl `llm_rerank_chunks()` avec prompt JSON
2. ✅ Parser robuste + fallback
3. ✅ Heuristique `should_llm_rerank()`
4. ✅ Logging comparatif
5. ✅ Tests edge cases
6. ✅ Analyse coût/bénéfice

**Validation** :
- [ ] Décision GO/NO-GO basée sur metrics
- [ ] Si amélioration < 5%, désactiver

---

## 🎯 Success Metrics

| Métrique | Baseline (RAG only) | Target (with LLM) |
|----------|---------------------|-------------------|
| **Pertinence réponse** | 60% (subjective) | 85%+ |
| **Recall Top-10** | 75% | 85%+ |
| **Latency P95** | 150ms | < 2000ms |
| **User satisfaction** | N/A | > 4/5 |
| **Queries FR recall** | 50% (poor) | 80%+ |
| **Numerical queries accuracy** | 90% (avec hard priority) | 95%+ |

---

## 🚀 Architecture Finale Complète

```
┌─────────────────────────────────────────────────────────────┐
│                        USER QUERY                           │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │  NIVEAU 2: Query Rewriting  │
        │  - FR → EN                  │
        │  - Verbose → Concise        │
        │  - Cache enabled            │
        └─────────────┬───────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │   RAG RETRIEVAL (Phase 3.6) │
        │  - Hybrid Search            │
        │  - Bibliography Filter      │
        │  - Numerical Reranking      │
        │  - Hard Priority Sorting    │
        └─────────────┬───────────────┘
                      │
                      ▼ (Top-20 chunks)
        ┌─────────────────────────────┐
        │ NIVEAU 3: LLM Reranking     │
        │  - Sémantic understanding   │
        │  - JSON output              │
        │  - Fallback-safe            │
        └─────────────┬───────────────┘
                      │
                      ▼ (Top-10 chunks)
        ┌─────────────────────────────┐
        │  NIVEAU 1: LLM Synthesis    │
        │  - Context building         │
        │  - Structured answer        │
        │  - Source citations         │
        │  - OCR warnings             │
        └─────────────┬───────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │      LlmChatResponse        │
        │  - answer: String           │
        │  - sources: Vec<SourceRef>  │
        │  - confidence: f32          │
        │  - metadata: Metrics        │
        └─────────────────────────────┘
```

---

## 📋 Résumé de la Review Technique

### Verdict Global : ✅ Roadmap Production-Ready

**Forces** :
- 🏗️ Architecture solide : augmentation du RAG, pas remplacement
- 📊 Structs bien pensés : explainability native
- 📝 Prompts clairs : instructions strictes pour rester factuel
- 🔍 Observabilité : logs et métriques dès le départ

**Ajustements Intégrés** :
- ✅ Troncature contexte (800 chars/chunk) → évite token overflow
- ✅ Confidence simplifiée (score top-1) → évite over-engineering
- ✅ Cache API claire (get/set séparés) → évite closures complexes
- ✅ Priorités clarifiées : N1 → N2 → (N3 optionnel)

**Prochaine Étape Concrète** :
```rust
// À implémenter dans DirectChatManager
async fn llm_answer_with_context(
    &self,
    query: &str,
    chunks: &[ScoredChunk],
) -> Result<LlmChatResponse> {
    // 1. build_context_string() avec .take(800)
    // 2. Appel LLM avec prompt template
    // 3. build_llm_response() avec confidence = top-1.score
}
```

**Impact Attendu** :
- Niveau 1 seul : **Transformation de l'UX** (chunks bruts → réponse synthétisée)
- + Niveau 2 : **Unlock docs anglais** avec queries françaises
- + Niveau 3 : **Marginal** (à évaluer après N1+N2)

---

**Auteur** : Claude (Assistant IA Anthropic)
**Date** : 20 novembre 2024
**Dernière mise à jour** : 20 novembre 2024 (Review technique intégrée)
**Version** : 1.1 - LLM Integration Roadmap (Post-Review)
**Status** : ✅ Validé - Prêt pour implémentation Sprint 1
