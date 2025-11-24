// GRAVIS Enhanced BM25 - Support n-grams et termes techniques
// Amélioration du tokenizer pour meilleure précision sur termes composés

use std::collections::HashMap;
use tracing::debug;

// SIMPLIFICATION 23 Nov V2: TECHNICAL_TERMS complètement supprimé
// IDF dynamique détecte automatiquement TOUS les termes rares
// Aucun hardcoding = 100% générique

/// Enhanced BM25 Encoder avec support n-grams
#[derive(Clone)]
pub struct EnhancedBM25Encoder {
    k1: f32,
    b: f32,
    avg_doc_length: f32,
    doc_lengths: HashMap<String, usize>,
    term_frequencies: HashMap<String, HashMap<String, usize>>,
    num_docs: usize,
}

impl EnhancedBM25Encoder {
    /// Créer un nouvel encodeur BM25 avec paramètres standards
    pub fn new() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            avg_doc_length: 0.0,
            doc_lengths: HashMap::new(),
            term_frequencies: HashMap::new(),
            num_docs: 0,
        }
    }

    /// Indexer un ensemble de documents
    pub fn index_documents(&mut self, documents: &[(String, String)]) {
        self.num_docs = documents.len();

        let mut total_length = 0;

        for (doc_id, content) in documents {
            let tokens = self.enhanced_tokenize(content);
            let doc_length = tokens.len();

            self.doc_lengths.insert(doc_id.clone(), doc_length);
            total_length += doc_length;

            // Compter fréquences des termes
            let mut term_freq = HashMap::new();
            for token in tokens {
                *term_freq.entry(token).or_insert(0) += 1;
            }

            self.term_frequencies.insert(doc_id.clone(), term_freq);
        }

        self.avg_doc_length = if self.num_docs > 0 {
            total_length as f32 / self.num_docs as f32
        } else {
            0.0
        };

        debug!("Indexed {} documents, avg length: {:.1} tokens",
               self.num_docs, self.avg_doc_length);
    }

    /// Tokenization améliorée avec n-grams - 100% générique
    /// SIMPLIFICATION 23 Nov V2: TECHNICAL_TERMS supprimé, IDF détecte automatiquement termes rares
    fn enhanced_tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let text_lower = text.to_lowercase();

        // 1. Tokenisation standard par mots
        let standard_tokens: Vec<String> = text_lower
            .split_whitespace()
            .map(|s| self.normalize_token(s))
            .filter(|s| !s.is_empty())
            .collect();

        tokens.extend(standard_tokens.clone());

        // 2. Génération de bigrams pour termes composés
        for window in standard_tokens.windows(2) {
            let bigram = format!("{}_{}", window[0], window[1]);
            tokens.push(bigram);
        }

        tokens
    }

    /// Normaliser un token
    fn normalize_token(&self, token: &str) -> String {
        token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase()
    }

    /// Calculer score BM25 pour une requête sur un document
    pub fn score(&self, query: &str, doc_id: &str) -> f32 {
        let query_tokens = self.enhanced_tokenize(query);

        let doc_length = match self.doc_lengths.get(doc_id) {
            Some(&len) => len,
            None => return 0.0,
        };

        let term_freq = match self.term_frequencies.get(doc_id) {
            Some(tf) => tf,
            None => return 0.0,
        };

        let mut score = 0.0;

        for query_term in query_tokens.iter() {
            let tf = *term_freq.get(query_term).unwrap_or(&0) as f32;

            if tf == 0.0 {
                continue;
            }

            // IDF calculation
            let doc_freq = self.document_frequency(query_term);
            let idf = if doc_freq > 0 {
                ((self.num_docs as f32 - doc_freq as f32 + 0.5) / (doc_freq as f32 + 0.5) + 1.0).ln()
            } else {
                0.0
            };

            // BM25 formula
            let length_norm = 1.0 - self.b + self.b * (doc_length as f32 / self.avg_doc_length);
            let tf_component = (tf * (self.k1 + 1.0)) / (tf + self.k1 * length_norm);

            score += idf * tf_component;
        }

        score
    }

    /// Calculer la fréquence documentaire d'un terme
    fn document_frequency(&self, term: &str) -> usize {
        self.term_frequencies
            .values()
            .filter(|tf| tf.contains_key(term))
            .count()
    }

    /// Calculer keyword boost basé sur co-occurrence query-content (100% générique)
    /// SIMPLIFICATION 23 Nov V2: Plus de TECHNICAL_TERMS hardcodés
    /// Boost proportionnel au nombre de termes rares (bas IDF) partagés entre query et content
    pub fn keyword_boost(&self, query: &str, content: &str) -> f32 {
        let query_tokens = self.enhanced_tokenize(query);
        let content_lower = content.to_lowercase();

        let mut boost: f32 = 0.0;
        let mut rare_term_matches = 0;

        // Pour chaque terme de la query, vérifier s'il est rare (bas doc_freq) ET présent dans content
        for token in query_tokens.iter() {
            // Ignorer tokens très courts (stop words implicites)
            if token.len() < 3 {
                continue;
            }

            // Vérifier si le token est présent dans le contenu
            if content_lower.contains(token.as_str()) {
                // Calculer rareté du terme via IDF
                let doc_freq = self.document_frequency(token);
                let idf = if doc_freq > 0 && self.num_docs > 0 {
                    ((self.num_docs as f32 - doc_freq as f32 + 0.5) / (doc_freq as f32 + 0.5) + 1.0).ln()
                } else {
                    0.0
                };

                // Boost proportionnel à la rareté du terme (IDF normalisé)
                // IDF typique: 0-5, on normalise à 0-0.2 par terme
                let normalized_idf_boost = (idf / 5.0).min(0.2);
                boost += normalized_idf_boost;

                // Compter les termes rares (IDF > 2.0 = apparaît dans <15% des docs)
                if idf > 2.0 {
                    rare_term_matches += 1;
                }
            }
        }

        // Bonus si plusieurs termes rares matchent (signe de haute pertinence)
        if rare_term_matches >= 2 {
            boost += 0.2;
        }

        // Cap à 1.0
        boost.min(1.0_f32)
    }

    // SIMPLIFICATION 23 Nov: has_explanatory_context() supprimée
    // Raison: Heuristiques linguistiques fragiles et langue-dépendantes
    // Le LLM est bien meilleur pour détecter le contexte explicatif

    /// Détecte si un chunk fait référence à une figure ou tableau avec données
    pub fn has_figure_reference(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();

        // Patterns de référence à des figures/tableaux
        let figure_patterns = [
            "figure", "fig.", "fig ", "table", "tableau",
            "graph", "graphique", "chart", "courbe",
            "shows", "montre", "illustre", "illustrates",
            "depicts", "représente",
        ];

        // Patterns de données chiffrées
        let data_patterns = [
            "compression ratio", "taux de compression",
            "accuracy", "précision", "performance",
            "%", "percent", "pourcent",
        ];

        let has_figure = figure_patterns.iter().any(|&p| content_lower.contains(p));
        let has_data = data_patterns.iter().any(|&p| content_lower.contains(p));

        has_figure && has_data
    }
}

impl Default for EnhancedBM25Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_tokenization() {
        let encoder = EnhancedBM25Encoder::new();

        let text = "The model uses convolutional compression with high ratio";
        let tokens = encoder.enhanced_tokenize(text);

        // Vérifier présence des tokens individuels
        assert!(tokens.contains(&"model".to_string()));
        assert!(tokens.contains(&"convolutional".to_string()));
        assert!(tokens.contains(&"compression".to_string()));

        // Vérifier génération de bigrams
        assert!(tokens.iter().any(|t| t.contains("_")));
        assert!(tokens.contains(&"model_uses".to_string()));
    }

    #[test]
    fn test_keyword_boost() {
        let mut encoder = EnhancedBM25Encoder::new();

        // Indexer des documents pour calculer IDF
        let docs = vec![
            ("doc1".to_string(), "neural network architecture compression".to_string()),
            ("doc2".to_string(), "deep learning model training".to_string()),
            ("doc3".to_string(), "transformer attention mechanism".to_string()),
            ("doc4".to_string(), "convolutional neural networks".to_string()),
            ("doc5".to_string(), "machine learning algorithms".to_string()),
        ];
        encoder.index_documents(&docs);

        let query = "neural compression architecture";
        let content_relevant = "The neural network uses advanced compression in its architecture";
        let content_irrelevant = "Some other content about general topics";

        let boost_relevant = encoder.keyword_boost(query, content_relevant);
        let boost_irrelevant = encoder.keyword_boost(query, content_irrelevant);

        // Le contenu pertinent devrait avoir un boost plus élevé grâce aux termes rares partagés
        assert!(boost_relevant > boost_irrelevant);
    }

    // SIMPLIFICATION 23 Nov: test_variants_generation supprimé
    // generate_variants() a été supprimée

    #[test]
    fn test_bm25_scoring() {
        let mut encoder = EnhancedBM25Encoder::new();

        let docs = vec![
            ("doc1".to_string(), "neural network uses convolutional compression architecture".to_string()),
            ("doc2".to_string(), "vision model uses parallel computation method".to_string()),
            ("doc3".to_string(), "another document about transformers and attention".to_string()),
        ];

        encoder.index_documents(&docs);

        let query = "neural compression architecture";

        let score1 = encoder.score(query, "doc1");
        let score2 = encoder.score(query, "doc2");

        // doc1 devrait scorer plus haut car contient les termes de la requête
        assert!(score1 > score2);
    }
}
