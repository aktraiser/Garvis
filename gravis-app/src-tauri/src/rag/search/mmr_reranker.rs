// Module MMR (Maximal Marginal Relevance) pour re-ranking des résultats
use std::collections::HashSet;
use anyhow::Result;
use tracing::debug;

/// Structure pour les résultats de recherche avec scores
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub embedding: Vec<f32>,
}

/// Reranker MMR pour réduire la redondance
pub struct MMRReranker {
    lambda: f32,  // Balance relevance vs diversity (0.5 = équilibre)
}

impl Default for MMRReranker {
    fn default() -> Self {
        Self { lambda: 0.5 }
    }
}

impl MMRReranker {
    pub fn new(lambda: f32) -> Self {
        Self { lambda: lambda.clamp(0.0, 1.0) }
    }
    
    /// Applique MMR re-ranking sur les résultats
    pub fn rerank(
        &self,
        query_embedding: &[f32],
        results: &[SearchResult],
        k_final: usize,
    ) -> Result<Vec<SearchResult>> {
        if results.is_empty() || k_final == 0 {
            return Ok(Vec::new());
        }
        
        let k_final = k_final.min(results.len());
        let mut selected = Vec::with_capacity(k_final);
        let mut selected_ids = HashSet::new();
        let mut remaining: Vec<&SearchResult> = results.iter().collect();
        
        // Premier élément: meilleur score de similarité
        if let Some(best) = remaining.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap()) {
            selected.push((*best).clone());
            selected_ids.insert(best.id.clone());
            remaining.retain(|r| r.id != best.id);
        }
        
        // Sélection itérative basée sur MMR
        while selected.len() < k_final && !remaining.is_empty() {
            let mut best_candidate_id = None;
            let mut best_mmr_score = f32::NEG_INFINITY;
            
            for candidate in &remaining {
                // Score de similarité avec la requête
                let relevance = cosine_similarity(query_embedding, &candidate.embedding);
                
                // Score de diversité (1 - max_similarity avec sélectionnés)
                let mut max_similarity: f32 = 0.0;
                for selected_item in &selected {
                    let similarity = cosine_similarity(&candidate.embedding, &selected_item.embedding);
                    max_similarity = max_similarity.max(similarity);
                }
                let diversity = 1.0 - max_similarity;
                
                // Score MMR: λ * relevance + (1-λ) * diversity
                let mmr_score = self.lambda * relevance + (1.0 - self.lambda) * diversity;
                
                if mmr_score > best_mmr_score {
                    best_mmr_score = mmr_score;
                    best_candidate_id = Some(candidate.id.clone());
                }
            }
            
            // Ajouter le meilleur candidat
            if let Some(best_id) = best_candidate_id {
                // Trouver et cloner le candidat
                if let Some(best_candidate) = remaining.iter().find(|r| r.id == best_id) {
                    selected.push((*best_candidate).clone());
                    selected_ids.insert(best_id.clone());
                    
                    debug!("MMR selected: {} (score: {:.3})", best_id, best_mmr_score);
                }
                
                // Retirer de remaining
                remaining.retain(|r| r.id != best_id);
            } else {
                break;
            }
        }
        
        debug!("MMR reranking: {} → {} results (λ={})", results.len(), selected.len(), self.lambda);
        Ok(selected)
    }
}

/// Calcule la similarité cosine entre deux vecteurs
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_mmr_reranking() {
        let reranker = MMRReranker::new(0.5);
        
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                content: "test1".to_string(),
                score: 0.9,
                embedding: vec![1.0, 0.0, 0.0],
            },
            SearchResult {
                id: "2".to_string(),
                content: "test2".to_string(),
                score: 0.8,
                embedding: vec![0.9, 0.1, 0.0], // Similaire à 1
            },
            SearchResult {
                id: "3".to_string(),
                content: "test3".to_string(),
                score: 0.7,
                embedding: vec![0.0, 1.0, 0.0], // Différent
            },
        ];
        
        let query_embedding = vec![1.0, 0.0, 0.0];
        let reranked = reranker.rerank(&query_embedding, &results, 2).unwrap();
        
        assert_eq!(reranked.len(), 2);
        assert_eq!(reranked[0].id, "1"); // Meilleur score initial
        // Le second devrait être "3" (plus diverse) plutôt que "2" (redondant)
    }
}