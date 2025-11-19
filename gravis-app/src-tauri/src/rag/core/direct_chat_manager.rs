// Phase 2: Gestionnaire de sessions temporaires pour chat direct
// Gestion TTL et nettoyage automatique des sessions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

use super::direct_chat::{
    DirectChatSession, DirectChatError, DirectChatResult, SelectionContext
};
use crate::rag::{EnrichedChunk, CustomE5Embedder, EnhancedBM25Encoder, ScoringEngine};

/// Gestionnaire de sessions temporaires
#[derive(Clone)]
pub struct DirectChatManager {
    sessions: Arc<RwLock<HashMap<String, DirectChatSession>>>,
    pub embedder: Arc<CustomE5Embedder>, // Public pour accès direct pendant traitement
    ttl_seconds: u64, // Time-to-live par défaut
}

impl DirectChatManager {
    /// Créer nouveau gestionnaire
    pub fn new(embedder: Arc<CustomE5Embedder>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            embedder,
            ttl_seconds: 3600, // 1 heure par défaut
        }
    }

    /// Créer nouveau gestionnaire avec TTL personnalisé
    pub fn with_ttl(embedder: Arc<CustomE5Embedder>, ttl_seconds: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            embedder,
            ttl_seconds,
        }
    }

    /// Stocker une session temporaire
    pub async fn store_session(&self, mut session: DirectChatSession) -> DirectChatResult<()> {
        // Générer embeddings pour les chunks si pas déjà fait
        if session.embedded_chunks_count() == 0 {
            debug!("Generating embeddings for {} chunks in session {}", 
                   session.chunks.len(), session.session_id);
            
            let mut embedded_count = 0;
            for chunk in &mut session.chunks {
                if !chunk.content.trim().is_empty() 
                    && !chunk.content.starts_with("EXTRACTION FAILED") {
                    
                    match self.embedder.encode_document(&chunk.content).await {
                        Ok(embedding) => {
                            chunk.embedding = Some(embedding);
                            embedded_count += 1;
                        }
                        Err(e) => {
                            warn!("Failed to embed chunk {} in session {}: {}", 
                                  chunk.id, session.session_id, e);
                        }
                    }
                }
            }
            
            info!("Generated {} embeddings for session {}", 
                  embedded_count, session.session_id);
        }

        let session_id = session.session_id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        info!("Stored direct chat session: {} (TTL: {}s)", 
              session_id, self.ttl_seconds);
        
        Ok(())
    }

    /// Récupérer une session
    pub async fn get_session(&self, session_id: &str) -> DirectChatResult<DirectChatSession> {
        let sessions = self.sessions.read().await;
        
        match sessions.get(session_id) {
            Some(session) => {
                // Vérifier expiration
                if session.is_expired(self.ttl_seconds) {
                    drop(sessions);
                    self.remove_session(session_id).await?;
                    return Err(DirectChatError::SessionExpired(session_id.to_string()));
                }
                
                Ok(session.clone())
            }
            None => Err(DirectChatError::SessionNotFound(session_id.to_string())),
        }
    }

    /// Supprimer une session
    pub async fn remove_session(&self, session_id: &str) -> DirectChatResult<()> {
        let mut sessions = self.sessions.write().await;
        
        match sessions.remove(session_id) {
            Some(_) => {
                info!("Removed direct chat session: {}", session_id);
                Ok(())
            }
            None => Err(DirectChatError::SessionNotFound(session_id.to_string())),
        }
    }

    /// Nettoyer sessions expirées (appelé périodiquement)
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        // Identifier sessions expirées
        let expired_ids: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired(self.ttl_seconds))
            .map(|(id, _)| id.clone())
            .collect();

        // Supprimer sessions expirées
        for id in &expired_ids {
            sessions.remove(id);
        }

        let cleaned_count = expired_ids.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired direct chat sessions", cleaned_count);
        }

        cleaned_count
    }

    /// Recherche hybride avec normalisation et poids adaptatifs par intent
    pub async fn search_in_session(
        &self,
        session_id: &str,
        query: &str,
        selection: Option<SelectionContext>,
        limit: Option<usize>,
    ) -> DirectChatResult<Vec<ScoredChunk>> {
        let session = self.get_session(session_id).await?;

        // Générer embedding de la requête
        let query_embedding = self.embedder
            .encode(&query)
            .await
            .map_err(|e| DirectChatError::EmbeddingFailed(e.to_string()))?;

        // Filtrer chunks selon la sélection utilisateur
        let chunks_to_search = if let Some(sel) = selection {
            self.filter_chunks_by_selection(&session.chunks, &sel)?
        } else {
            session.chunks.clone()
        };

        debug!("🔍 Hybrid search in {} chunks for session {}",
               chunks_to_search.len(), session_id);

        // === SCORING ENGINE: Normalisation + Intent Detection + IDF ===

        // 1. Initialiser BM25 encoder
        let mut bm25_encoder = EnhancedBM25Encoder::new();

        // Préparer documents pour BM25
        let bm25_docs: Vec<(String, String)> = chunks_to_search
            .iter()
            .map(|chunk| (chunk.id.clone(), chunk.content.clone()))
            .collect();

        bm25_encoder.index_documents(&bm25_docs);

        // 2. Initialiser Scoring Engine avec IDF
        let mut scoring_engine = ScoringEngine::new();
        scoring_engine.build_idf_map(&bm25_docs);

        // 3. Détecter l'intent de la requête
        let query_intent = scoring_engine.detect_intent(query);

        info!("🎯 Query: '{}' | Intent: {:?}", query, query_intent);

        // 4. Calculer tous les scores bruts
        let mut dense_scores = Vec::new();
        let mut sparse_scores = Vec::new();
        let mut keyword_boosts = Vec::new();

        for chunk in &chunks_to_search {
            // Score dense (embeddings sémantiques)
            let dense_score = if let Some(ref chunk_embedding) = chunk.embedding {
                cosine_similarity(&query_embedding, chunk_embedding)
            } else {
                0.0
            };

            // Score sparse (BM25 lexical)
            let sparse_score = bm25_encoder.score(query, &chunk.id);

            // Keyword boost de base
            let base_boost = bm25_encoder.keyword_boost(query, &chunk.content);

            // Boost additionnel pour termes techniques détectés dynamiquement via IDF
            let keyword_boost = scoring_engine.apply_dynamic_technical_boost(
                query,
                &chunk.content,
                base_boost
            );

            dense_scores.push(dense_score);
            sparse_scores.push(sparse_score);
            keyword_boosts.push(keyword_boost);
        }

        // 5. Calculer scores hybrides normalisés avec poids adaptatifs
        let hybrid_scores = scoring_engine.compute_hybrid_scores(
            &dense_scores,
            &sparse_scores,
            &keyword_boosts,
            &query_intent
        );

        // 6. Créer les scored chunks
        let mut scored_chunks: Vec<ScoredChunk> = chunks_to_search
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                debug!("🎯 Chunk {}: dense={:.3}, sparse={:.3}, boost={:.3}, hybrid={:.3}",
                       &chunk.id[..12.min(chunk.id.len())],
                       dense_scores[i], sparse_scores[i], keyword_boosts[i], hybrid_scores[i]);

                ScoredChunk {
                    chunk,
                    score: hybrid_scores[i],
                }
            })
            .collect();

        // Trier par score hybride décroissant
        scored_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Log top chunks pour debug
        if !scored_chunks.is_empty() {
            info!("🏆 Top chunk: score={:.3}, preview: {}",
                  scored_chunks[0].score,
                  scored_chunks[0].chunk.content.chars().take(80).collect::<String>());
        }

        // Limiter résultats
        let limit = limit.unwrap_or(10);
        scored_chunks.truncate(limit);

        Ok(scored_chunks)
    }

    /// Filtrer chunks selon sélection utilisateur
    fn filter_chunks_by_selection(
        &self,
        chunks: &[EnrichedChunk],
        selection: &SelectionContext,
    ) -> DirectChatResult<Vec<EnrichedChunk>> {
        match (selection.text.as_ref(), selection.bounding_rect.as_ref()) {
            // Filtrage par texte sélectionné
            (Some(selected_text), _) => {
                let filtered: Vec<EnrichedChunk> = chunks
                    .iter()
                    .filter(|chunk| {
                        // Score de similarité textuelle simple
                        let similarity = text_similarity(&chunk.content, selected_text);
                        similarity > 0.3 // Seuil de pertinence
                    })
                    .cloned()
                    .collect();

                debug!("Filtered {} chunks by text selection: '{}'", 
                       filtered.len(), selected_text.chars().take(50).collect::<String>());
                
                Ok(filtered)
            }
            
            // Filtrage par bbox (nécessiterait source spans)
            (None, Some(_bbox)) => {
                // TODO: Implémenter filtrage par intersection bbox avec source spans
                warn!("Bbox filtering not yet implemented, returning all chunks");
                Ok(chunks.to_vec())
            }
            
            // Pas de filtrage
            (None, None) => Ok(chunks.to_vec()),
        }
    }

    /// Obtenir statistiques des sessions actives
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        
        let total_sessions = sessions.len();
        let total_chunks: usize = sessions.values()
            .map(|s| s.chunks.len())
            .sum();
        let embedded_chunks: usize = sessions.values()
            .map(|s| s.embedded_chunks_count())
            .sum();
        let expired_count = sessions.values()
            .filter(|s| s.is_expired(self.ttl_seconds))
            .count();

        SessionStats {
            total_sessions,
            total_chunks,
            embedded_chunks,
            expired_sessions: expired_count,
            ttl_seconds: self.ttl_seconds,
        }
    }

    /// Lister toutes les sessions actives (pour debug/admin)
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        
        sessions.values()
            .map(|session| SessionInfo {
                session_id: session.session_id.clone(),
                document_name: session.document_name.clone(),
                chunks_count: session.chunks.len(),
                embedded_chunks: session.embedded_chunks_count(),
                created_at: session.created_at,
                is_expired: session.is_expired(self.ttl_seconds),
            })
            .collect()
    }
}

/// Chunk avec score de pertinence
#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub chunk: EnrichedChunk,
    pub score: f32,
}

/// Statistiques du gestionnaire de sessions
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub total_chunks: usize,
    pub embedded_chunks: usize,
    pub expired_sessions: usize,
    pub ttl_seconds: u64,
}

/// Information résumée sur une session
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub document_name: String,
    pub chunks_count: usize,
    pub embedded_chunks: usize,
    pub created_at: std::time::SystemTime,
    pub is_expired: bool,
}

// === Fonctions utilitaires ===

/// Calcul similarité cosinus entre deux vecteurs
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Similarité textuelle simple (Jaccard sur mots)
fn text_similarity(text1: &str, text2: &str) -> f32 {
    let text1_lower = text1.to_lowercase();
    let text2_lower = text2.to_lowercase();
    
    let words1: std::collections::HashSet<&str> = text1_lower
        .split_whitespace()
        .collect();
    let words2: std::collections::HashSet<&str> = text2_lower
        .split_whitespace()
        .collect();

    if words1.is_empty() && words2.is_empty() {
        return 1.0;
    }

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    intersection as f32 / union as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{
        DocumentType, ChunkType, ChunkMetadata, Priority, SourceType, 
        ExtractionMethod, CustomE5Config
    };
    use crate::rag::core::direct_chat::OCRContent;

    #[tokio::test]
    async fn test_session_management() {
        // Setup embedder mock
        let embedder = Arc::new(
            CustomE5Embedder::new(CustomE5Config::default())
                .await
                .expect("Failed to create embedder")
        );

        let manager = DirectChatManager::with_ttl(embedder, 60); // 1 minute TTL

        // Créer session test
        let session = DirectChatSession::new(
            std::path::PathBuf::from("/test.pdf"),
            DocumentType::PlainText,
            vec![],
            OCRContent::empty(),
        );
        let session_id = session.session_id.clone();

        // Stocker session
        manager.store_session(session).await.unwrap();

        // Récupérer session
        let retrieved = manager.get_session(&session_id).await.unwrap();
        assert_eq!(retrieved.session_id, session_id);

        // Vérifier stats
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_sessions, 1);

        // Supprimer session
        manager.remove_session(&session_id).await.unwrap();
        
        // Vérifier suppression
        assert!(manager.get_session(&session_id).await.is_err());
    }

    #[test]
    fn test_text_similarity() {
        let text1 = "Le chat mange la souris";
        let text2 = "La souris mange le fromage";
        
        let similarity = text_similarity(text1, text2);
        assert!(similarity > 0.0 && similarity < 1.0);

        let identical = text_similarity(text1, text1);
        assert_eq!(identical, 1.0);

        let different = text_similarity(text1, "Completely different text");
        assert!(different < 0.3);
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 1.0];
        let vec2 = vec![1.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 1.0]; // identique à vec1

        let sim_diff = cosine_similarity(&vec1, &vec2);
        assert!(sim_diff > 0.0 && sim_diff < 1.0);

        let sim_same = cosine_similarity(&vec1, &vec3);
        assert!((sim_same - 1.0).abs() < 1e-6);

        let sim_empty = cosine_similarity(&[], &[]);
        assert_eq!(sim_empty, 0.0);
    }
}