// Query-Aware Reranker - Sprint 1 Niveau 1.5
// Reranking sémantique léger basé sur la correspondance query-chunk
// Objectif: Améliorer le ranking pour queries "objectif", "but", "goal", etc.

use tracing::debug;
use std::collections::HashSet;

/// Configuration du reranker query-aware
pub struct QueryAwareReranker {
    /// Poids du score original (0.0-1.0)
    original_weight: f32,
    /// Poids du score sémantique query-aware (0.0-1.0)
    semantic_weight: f32,
}

impl Default for QueryAwareReranker {
    fn default() -> Self {
        Self {
            original_weight: 0.7,  // 70% score original
            semantic_weight: 0.3,  // 30% boost sémantique
        }
    }
}

impl QueryAwareReranker {
    pub fn new(original_weight: f32, semantic_weight: f32) -> Self {
        Self {
            original_weight: original_weight.max(0.0_f32).min(1.0_f32),
            semantic_weight: semantic_weight.max(0.0_f32).min(1.0_f32),
        }
    }

    /// Reranking query-aware pour améliorer la pertinence
    pub fn rerank<T>(
        &self,
        query: &str,
        items: Vec<(T, f32)>,  // (item, score_original)
        get_content: impl Fn(&T) -> &str,
    ) -> Vec<(T, f32)> {
        if items.is_empty() {
            return items;
        }

        let query_lower = query.to_lowercase();

        // Détecter le type de query
        let query_type = self.detect_query_type(&query_lower);

        let mut reranked: Vec<(T, f32, f32)> = items
            .into_iter()
            .map(|(item, original_score)| {
                let content = get_content(&item);
                let semantic_score = self.compute_semantic_score(&query_lower, content, &query_type);

                // Score final: weighted combination
                let final_score =
                    self.original_weight * original_score +
                    self.semantic_weight * semantic_score;

                (item, final_score, semantic_score)
            })
            .collect();

        // Trier par score final (desc)
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!(
            "Query-aware reranking: type={:?}, top score boost: {:.3} → {:.3}",
            query_type,
            reranked.first().map(|(_, _, s)| *s).unwrap_or(0.0),
            reranked.first().map(|(_, f, _)| *f).unwrap_or(0.0)
        );

        // Retourner avec scores finaux
        reranked.into_iter().map(|(item, final_score, _)| (item, final_score)).collect()
    }

    /// Détecte le type de query pour adapter le scoring
    fn detect_query_type(&self, query: &str) -> QueryType {
        // Patterns "objectif/but/goal"
        let goal_patterns = ["objectif", "but", "goal", "aim", "purpose", "objective"];
        if goal_patterns.iter().any(|p| query.contains(p)) {
            return QueryType::Goal;
        }

        // Patterns "méthode/approche/how"
        let method_patterns = ["méthode", "approche", "comment", "how", "method", "approach"];
        if method_patterns.iter().any(|p| query.contains(p)) {
            return QueryType::Method;
        }

        // Patterns "résultat/performance"
        let result_patterns = ["résultat", "performance", "result", "accuracy", "score"];
        if result_patterns.iter().any(|p| query.contains(p)) {
            return QueryType::Result;
        }

        QueryType::General
    }

    /// Calcule un score sémantique basé sur la correspondance query-chunk
    fn compute_semantic_score(&self, query: &str, content: &str, query_type: &QueryType) -> f32 {
        let content_lower = content.to_lowercase();

        match query_type {
            QueryType::Goal => self.score_goal_query(&content_lower),
            QueryType::Method => self.score_method_query(&content_lower),
            QueryType::Result => self.score_result_query(&content_lower),
            QueryType::General => self.score_general(&content_lower, query),
        }
    }

    /// Scoring pour queries de type "objectif/but"
    fn score_goal_query(&self, content: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Boost fort si contient des marqueurs d'objectif
        let goal_markers = [
            "objective", "goal", "aim", "purpose", "propose",
            "objectif", "but", "vise", "visons",
            "we propose", "our goal", "this paper", "we aim",
            "explore a potential solution", "leveraging", "enables",
            // Marqueurs de contexte long et limitations
            "context", "long", "longer", "token", "limitation", "window",
            "longer contexts", "context length", "token reduction",
            "thousands of tokens", "hundreds of tokens", "context window",
            "enable", "permet", "permettre", "gérer", "manage",
        ];

        for marker in goal_markers {
            if content.contains(marker) {
                score += 0.2;  // +20% par marqueur
            }
        }

        // Pénalité forte si c'est juste une table/liste de résultats
        if content.contains("table") && content.matches(',').count() > 5 {
            score -= 0.5;  // Augmenté de 0.3 → 0.5
        }

        // Pénalité si c'est une description visuelle
        if content.contains("library") || content.contains("room") || content.contains("furniture") {
            score -= 0.5;
        }

        // Pénalité forte si c'est une liste de modèles comparés (benchmark noise)
        let benchmark_noise_patterns = [
            "qwen", "olmocr", "ocrflux", "internvl", "mineru",
            "llama", "gpt-4", "claude", "gemini",
            "omnidocbench", "docvqa", "benchmark",
        ];
        let mut benchmark_match_count = 0;
        for pattern in benchmark_noise_patterns {
            if content.contains(pattern) {
                benchmark_match_count += 1;
            }
        }
        // Si 2+ modèles détectés → probablement une table de comparaison
        if benchmark_match_count >= 2 {
            score -= 0.7;  // Pénalité très forte
        }

        // Pénalité si caption de figure/table (souvent hors-sujet pour goal queries)
        if content.contains("figure caption") || content.contains("[figure caption") {
            score -= 0.4;
        }

        // Pénalité FORTE pour détails techniques d'architecture (SAM, CLIP, components, layers)
        // Ces chunks décrivent le HOW, pas le WHY
        let technical_details_patterns = [
            "sam", "clip", "vitdet", "patch embedding", "layer",
            "components", "pipeline", "encoder", "decoder",
            "architecture", "attention", "embedding layer",
            "we remove", "we use", "we employ", "based on",
        ];
        let mut technical_match_count = 0;
        for pattern in technical_details_patterns {
            if content.contains(pattern) {
                technical_match_count += 1;
            }
        }
        // Si 3+ termes techniques détectés → probablement une description d'implémentation
        if technical_match_count >= 3 {
            score -= 0.6;  // Pénalité forte pour goal queries
        }

        // BOOST MASSIF si section Abstract/Introduction/Conclusion
        // Ces sections contiennent l'objectif stratégique (WHY)
        if content.starts_with("abstract") || content.contains("in this paper") || content.contains("in this work") {
            score += 0.5;  // Augmenté de 0.3 → 0.5
        }
        if content.contains("our goal") || content.contains("we propose to") || content.contains("this paper proposes") {
            score += 0.4;  // Boost supplémentaire pour phrases d'objectif
        }

        f32::max(0.0, f32::min(1.0, score))
    }

    /// Scoring pour queries de type "méthode/approche"
    fn score_method_query(&self, content: &str) -> f32 {
        let mut score: f32 = 0.0;

        let method_markers = [
            "method", "approach", "architecture", "pipeline",
            "méthode", "approche", "architecture",
            "we use", "we employ", "using", "based on",
        ];

        for marker in method_markers {
            if content.contains(marker) {
                score += 0.2;
            }
        }

        // Bonus si c'est une figure d'architecture
        if content.contains("figure") && content.contains("architecture") {
            score += 0.3;
        }

        f32::max(0.0, f32::min(1.0, score))
    }

    /// Scoring pour queries de type "résultat/performance"
    fn score_result_query(&self, content: &str) -> f32 {
        let mut score: f32 = 0.0;

        let result_markers = [
            "result", "performance", "accuracy", "score",
            "résultat", "performance", "précision",
            "table", "benchmark", "achieves", "outperforms",
        ];

        for marker in result_markers {
            if content.contains(marker) {
                score += 0.15;
            }
        }

        // Bonus si contient des chiffres (probablement des résultats)
        let digit_count = content.chars().filter(|c| c.is_numeric()).count();
        if digit_count > 3 {
            score += 0.2;
        }

        f32::max(0.0, f32::min(1.0, score))
    }

    /// Scoring général basé sur overlap lexical enrichi
    fn score_general(&self, content: &str, query: &str) -> f32 {
        // Extraire tokens significatifs de la query
        let query_tokens: HashSet<&str> = query
            .split_whitespace()
            .filter(|t| t.len() > 3)  // Ignorer mots courts
            .collect();

        if query_tokens.is_empty() {
            return 0.5;  // Score neutre
        }

        // Compter matches dans le contenu
        let matches = query_tokens.iter()
            .filter(|&token| content.contains(token))
            .count();

        let ratio = matches as f32 / query_tokens.len() as f32;
        f32::max(0.0, f32::min(1.0, ratio))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QueryType {
    Goal,      // "objectif", "but", "goal"
    Method,    // "méthode", "approche", "how"
    Result,    // "résultat", "performance"
    General,   // Autre
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_query_detection() {
        let reranker = QueryAwareReranker::default();

        let query = "quel est l'objectif principal du modèle";
        let query_type = reranker.detect_query_type(query);
        assert_eq!(query_type, QueryType::Goal);
    }

    #[test]
    fn test_goal_scoring() {
        let reranker = QueryAwareReranker::default();

        // Chunk avec marqueurs d'objectif
        let good_chunk = "We propose a novel approach to leverage visual modality as compression";
        let score_good = reranker.score_goal_query(good_chunk);

        // Chunk qui est juste une table
        let bad_chunk = "Table 3: model1, 0.95, model2, 0.87, model3, 0.92";
        let score_bad = reranker.score_goal_query(bad_chunk);

        assert!(score_good > score_bad, "Goal chunk should score higher than table");
    }

    #[test]
    fn test_reranking() {
        let reranker = QueryAwareReranker::new(0.5, 0.5);

        let items = vec![
            ("Table 3: benchmark results", 0.99),  // Score original élevé mais hors-sujet
            ("We propose to leverage visual compression", 0.85),  // Score moyen mais pertinent
            ("Abstract: Our goal is to compress text", 0.80),  // Bon pour goal query
        ];

        let query = "What is the objective of the model?";

        let reranked = reranker.rerank(query, items.clone(), |(content, _)| content);

        // Vérifier que les chunks pertinents remontent
        assert_eq!(reranked[0].0, "We propose to leverage visual compression");
    }
}
