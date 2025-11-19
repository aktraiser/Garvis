// GRAVIS Scoring Engine - Recherche hybride avec normalisation et intent detection
// Amélioration du scoring pour meilleure précision sur requêtes techniques

use std::collections::HashMap;
use tracing::debug;

/// Type d'intent détecté dans la requête pour le scoring
#[derive(Debug, Clone, PartialEq)]
pub enum SearchIntent {
    /// Requête avec terme technique spécifique (ex: "DeepEncoder 16x")
    ExactPhrase,
    /// Requête conceptuelle (ex: "Comment fonctionne l'architecture ?")
    Conceptual,
    /// Mélange des deux
    Mixed,
}

/// Configuration des poids par intent
#[derive(Debug, Clone)]
pub struct IntentWeights {
    pub dense: f32,
    pub sparse: f32,
    pub keyword: f32,
}

impl IntentWeights {
    /// Poids pour requêtes exactes/techniques
    pub fn exact_phrase() -> Self {
        Self {
            dense: 0.3,
            sparse: 0.5,
            keyword: 0.2,
        }
    }

    /// Poids pour requêtes conceptuelles
    pub fn conceptual() -> Self {
        Self {
            dense: 0.5,
            sparse: 0.3,
            keyword: 0.2,
        }
    }

    /// Poids équilibrés (défaut)
    pub fn mixed() -> Self {
        Self {
            dense: 0.4,
            sparse: 0.4,
            keyword: 0.2,
        }
    }

    /// Obtenir les poids pour un intent donné
    pub fn for_intent(intent: &SearchIntent) -> Self {
        match intent {
            SearchIntent::ExactPhrase => Self::exact_phrase(),
            SearchIntent::Conceptual => Self::conceptual(),
            SearchIntent::Mixed => Self::mixed(),
        }
    }
}

/// Moteur de scoring avec normalisation et intent detection
pub struct ScoringEngine {
    /// IDF map pour détecter termes techniques rares
    idf_map: HashMap<String, f32>,
}

impl ScoringEngine {
    /// Créer un nouveau moteur de scoring
    pub fn new() -> Self {
        Self {
            idf_map: HashMap::new(),
        }
    }

    /// Initialiser l'IDF map à partir d'un corpus de documents
    pub fn build_idf_map(&mut self, documents: &[(String, String)]) {
        let num_docs = documents.len() as f32;
        let mut doc_frequencies: HashMap<String, usize> = HashMap::new();

        // Compter dans combien de documents apparaît chaque terme
        for (_doc_id, content) in documents {
            let tokens = self.tokenize(content);
            let unique_tokens: std::collections::HashSet<String> = tokens.into_iter().collect();

            for token in unique_tokens {
                *doc_frequencies.entry(token).or_insert(0) += 1;
            }
        }

        // Calculer IDF pour chaque terme
        for (term, doc_freq) in doc_frequencies {
            let idf = ((num_docs / doc_freq as f32) + 1.0).ln();
            self.idf_map.insert(term, idf);
        }

        debug!("Built IDF map with {} terms", self.idf_map.len());
    }

    /// Tokenization simple pour IDF
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>())
            .filter(|s| !s.is_empty() && s.len() > 2)
            .collect()
    }

    /// Extraire les termes techniques (IDF élevé) d'une requête
    pub fn extract_technical_terms(&self, query: &str, top_k: usize) -> Vec<(String, f32)> {
        let query_tokens = self.tokenize(query);

        let mut scored: Vec<(String, f32)> = query_tokens
            .into_iter()
            .filter_map(|token| {
                self.idf_map.get(&token).map(|&idf| (token, idf))
            })
            .collect();

        // Trier par IDF décroissant
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Prendre les top_k
        scored.truncate(top_k);
        scored
    }

    /// Détecter l'intent de la requête
    pub fn detect_intent(&self, query: &str) -> SearchIntent {
        let query_lower = query.to_lowercase();
        let tokens = self.tokenize(query);

        // 1. Détection de termes techniques via IDF
        let technical_terms = self.extract_technical_terms(query, 3);
        let has_high_idf_terms = technical_terms.iter().any(|(_, idf)| *idf > 2.5);

        // 2. Détection de patterns exacts
        let has_quotes = query.contains('"');
        let has_specific_numbers = regex::Regex::new(r"\b\d+x\b|v\d+|\d+%")
            .unwrap()
            .is_match(query);

        // 3. Détection de questions conceptuelles
        let conceptual_patterns = [
            "comment", "pourquoi", "qu'est-ce", "quelle est", "quel est",
            "how", "why", "what is", "which",
            "expliquer", "décrire", "explain", "describe",
        ];
        let is_conceptual = conceptual_patterns.iter()
            .any(|pattern| query_lower.contains(pattern));

        // 4. Longueur et complexité
        let is_short_query = tokens.len() <= 3;

        // Décision finale avec priorité aux termes techniques
        // Si la requête contient des termes techniques spécifiques (16x, DeepEncoder, etc.)
        // même si elle est formulée comme une question, on privilégie le match exact
        if has_quotes || has_specific_numbers {
            debug!("🎯 Query intent: ExactPhrase (specific numbers/quotes: {:?})", technical_terms);
            SearchIntent::ExactPhrase
        } else if has_high_idf_terms && (is_short_query || !is_conceptual) {
            debug!("🎯 Query intent: ExactPhrase (high IDF terms: {:?})", technical_terms);
            SearchIntent::ExactPhrase
        } else if is_conceptual && !has_high_idf_terms {
            debug!("🧠 Query intent: Conceptual");
            SearchIntent::Conceptual
        } else {
            debug!("🔀 Query intent: Mixed");
            SearchIntent::Mixed
        }
    }

    /// Normaliser un vecteur de scores avec MinMax
    pub fn normalize_minmax(&self, scores: &[f32]) -> Vec<f32> {
        if scores.is_empty() {
            return vec![];
        }

        let min = scores.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        let range = (max - min).max(1e-6); // Éviter division par zéro

        scores.iter()
            .map(|s| (s - min) / range)
            .collect()
    }

    /// Calculer le score hybride normalisé avec poids adaptatifs
    pub fn compute_hybrid_scores(
        &self,
        dense_scores: &[f32],
        sparse_scores: &[f32],
        keyword_boosts: &[f32],
        query_intent: &SearchIntent,
    ) -> Vec<f32> {
        assert_eq!(dense_scores.len(), sparse_scores.len());
        assert_eq!(dense_scores.len(), keyword_boosts.len());

        // 1. Normaliser chaque composante
        let dense_norm = self.normalize_minmax(dense_scores);
        let sparse_norm = self.normalize_minmax(sparse_scores);
        let keyword_norm = self.normalize_minmax(keyword_boosts);

        debug!("📊 Normalized score ranges: dense=[{:.3},{:.3}], sparse=[{:.3},{:.3}], kw=[{:.3},{:.3}]",
               dense_norm.iter().cloned().fold(f32::INFINITY, f32::min),
               dense_norm.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
               sparse_norm.iter().cloned().fold(f32::INFINITY, f32::min),
               sparse_norm.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
               keyword_norm.iter().cloned().fold(f32::INFINITY, f32::min),
               keyword_norm.iter().cloned().fold(f32::NEG_INFINITY, f32::max));

        // 2. Obtenir les poids selon l'intent
        let weights = IntentWeights::for_intent(query_intent);

        debug!("⚖️  Intent weights: dense={:.1}, sparse={:.1}, keyword={:.1}",
               weights.dense, weights.sparse, weights.keyword);

        // 3. Calculer scores hybrides
        dense_norm.iter()
            .zip(sparse_norm.iter())
            .zip(keyword_norm.iter())
            .map(|((d, s), k)| {
                weights.dense * d + weights.sparse * s + weights.keyword * k
            })
            .collect()
    }

    /// Boost additionnel pour termes techniques détectés dynamiquement
    pub fn apply_dynamic_technical_boost(
        &self,
        query: &str,
        content: &str,
        base_boost: f32,
    ) -> f32 {
        let technical_terms = self.extract_technical_terms(query, 5);

        if technical_terms.is_empty() {
            return base_boost;
        }

        let content_lower = content.to_lowercase();
        let mut additional_boost = 0.0;

        for (term, idf) in technical_terms {
            if content_lower.contains(&term) {
                // Boost proportionnel à l'IDF (termes rares = boost plus fort)
                let term_boost = (idf / 5.0).min(0.2); // Cap à 0.2 par terme
                additional_boost += term_boost;
                debug!("🔍 Technical term match: '{}' (IDF={:.2}) → +{:.3}", term, idf, term_boost);
            }
        }

        (base_boost + additional_boost).min(1.0)
    }

    /// Obtenir l'IDF d'un terme (pour debug)
    pub fn get_idf(&self, term: &str) -> Option<f32> {
        self.idf_map.get(&term.to_lowercase()).copied()
    }
}

impl Default for ScoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idf_computation() {
        let mut engine = ScoringEngine::new();

        let docs = vec![
            ("doc1".to_string(), "DeepEncoder uses convolutional compression".to_string()),
            ("doc2".to_string(), "InternVL2 uses parallel computation method".to_string()),
            ("doc3".to_string(), "Transformer architecture with attention mechanism".to_string()),
        ];

        engine.build_idf_map(&docs);

        // "deepencoder" apparaît dans 1 doc → IDF élevé
        let idf_deepencoder = engine.get_idf("deepencoder").unwrap();
        // "uses" apparaît dans 2 docs → IDF moyen
        let idf_uses = engine.get_idf("uses").unwrap();

        assert!(idf_deepencoder > idf_uses);
    }

    #[test]
    fn test_intent_detection() {
        let mut engine = ScoringEngine::new();

        let docs = vec![
            ("doc1".to_string(), "DeepEncoder uses convolutional compression 16x".to_string()),
            ("doc2".to_string(), "InternVL2 uses parallel computation method".to_string()),
        ];
        engine.build_idf_map(&docs);

        // Requête technique
        let intent1 = engine.detect_intent("DeepEncoder 16x compression");
        assert_eq!(intent1, SearchIntent::ExactPhrase);

        // Requête conceptuelle
        let intent2 = engine.detect_intent("Comment fonctionne l'architecture ?");
        assert_eq!(intent2, SearchIntent::Conceptual);

        // Requête mixte
        let intent3 = engine.detect_intent("Quelle est la méthode utilisée par DeepEncoder ?");
        assert_eq!(intent3, SearchIntent::Mixed);
    }

    #[test]
    fn test_normalization() {
        let engine = ScoringEngine::new();

        let scores = vec![0.2, 0.5, 0.8, 0.3];
        let normalized = engine.normalize_minmax(&scores);

        // Min devrait être 0.0, max devrait être 1.0
        assert!((normalized[0] - 0.0).abs() < 1e-6); // 0.2 est le min
        assert!((normalized[2] - 1.0).abs() < 1e-6); // 0.8 est le max

        // Tous les scores entre 0 et 1
        for score in &normalized {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
    }

    #[test]
    fn test_hybrid_scoring() {
        let mut engine = ScoringEngine::new();

        let docs = vec![
            ("doc1".to_string(), "DeepEncoder technical term".to_string()),
            ("doc2".to_string(), "Common words everywhere".to_string()),
        ];
        engine.build_idf_map(&docs);

        let dense_scores = vec![0.7, 0.3];
        let sparse_scores = vec![0.9, 0.1];
        let keyword_boosts = vec![0.8, 0.0];

        // Test ExactPhrase intent (favorise sparse)
        let intent = SearchIntent::ExactPhrase;
        let hybrid = engine.compute_hybrid_scores(
            &dense_scores,
            &sparse_scores,
            &keyword_boosts,
            &intent
        );

        // Doc1 devrait scorer plus haut car sparse + keyword élevés
        assert!(hybrid[0] > hybrid[1]);
    }

    #[test]
    fn test_technical_term_extraction() {
        let mut engine = ScoringEngine::new();

        let docs = vec![
            ("doc1".to_string(), "DeepEncoder is a rare technical term".to_string()),
            ("doc2".to_string(), "Common words like the and is appear everywhere".to_string()),
            ("doc3".to_string(), "Another common document with regular terms".to_string()),
        ];
        engine.build_idf_map(&docs);

        let technical_terms = engine.extract_technical_terms("DeepEncoder rare term", 3);

        // "deepencoder" et "rare" devraient avoir des IDF élevés
        assert!(!technical_terms.is_empty());
        assert!(technical_terms.iter().any(|(term, _)| term == "deepencoder"));
    }
}
