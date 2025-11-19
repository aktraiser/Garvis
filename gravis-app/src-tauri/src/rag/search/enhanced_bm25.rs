// GRAVIS Enhanced BM25 - Support n-grams et termes techniques
// Amélioration du tokenizer pour meilleure précision sur termes composés

use std::collections::HashMap;
use tracing::debug;

/// Termes techniques à préserver intacts (sans split)
const TECHNICAL_TERMS: &[&str] = &[
    "deepencoder", "deepseek", "internvl", "onechart",
    "convolutionnel", "compresseur", "encoder", "decoder",
    "transformer", "attention", "16x", "32x", "64x",
    "baseline", "sota", "benchmark", "architecture",
];

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

    /// Tokenization améliorée avec n-grams et préservation termes techniques
    fn enhanced_tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let text_lower = text.to_lowercase();

        // 1. Détecter et préserver termes techniques intacts
        for &tech_term in TECHNICAL_TERMS {
            if text_lower.contains(tech_term) {
                tokens.push(tech_term.to_string());
            }
        }

        // 2. Tokenisation standard par mots
        let standard_tokens: Vec<String> = text_lower
            .split_whitespace()
            .map(|s| self.normalize_token(s))
            .filter(|s| !s.is_empty())
            .collect();

        tokens.extend(standard_tokens.clone());

        // 3. Génération de bigrams pour termes composés
        for window in standard_tokens.windows(2) {
            let bigram = format!("{}_{}", window[0], window[1]);
            tokens.push(bigram);
        }

        // 4. Générer variantes pour termes comme "DeepEncoder"
        for token in &standard_tokens {
            if let Some(variants) = self.generate_variants(token) {
                tokens.extend(variants);
            }
        }

        tokens
    }

    /// Générer variantes orthographiques pour termes composés
    fn generate_variants(&self, token: &str) -> Option<Vec<String>> {
        let mut variants = Vec::new();

        // Skip si le token contient des caractères non-ASCII (chinois, etc.)
        if !token.is_ascii() {
            return None;
        }

        // Patterns CamelCase détectés (ex: "deepencoder" → "deep_encoder")
        if token.len() > 6 {
            // Essayer de split en 2 parties égales - Safe avec chars()
            let char_count = token.chars().count();
            let mid = char_count / 2;
            if mid > 2 {
                let part1: String = token.chars().take(mid).collect();
                let part2: String = token.chars().skip(mid).collect();
                variants.push(format!("{}_{}", part1, part2));
            }

            // Patterns connus
            if token.contains("encoder") {
                let prefix = token.replace("encoder", "");
                if !prefix.is_empty() {
                    variants.push(format!("{}_encoder", prefix));
                }
            }

            if token.contains("decoder") {
                let prefix = token.replace("decoder", "");
                if !prefix.is_empty() {
                    variants.push(format!("{}_decoder", prefix));
                }
            }
        }

        if variants.is_empty() {
            None
        } else {
            Some(variants)
        }
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

    /// Calculer keyword boost pour un terme technique exact
    pub fn keyword_boost(&self, query: &str, content: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        let mut boost: f32 = 0.0;

        // Extraire termes techniques de la requête
        for &tech_term in TECHNICAL_TERMS {
            if query_lower.contains(tech_term) {
                // Match exact
                if content_lower.contains(tech_term) {
                    let base_boost = match tech_term {
                        "deepencoder" | "deepseek" | "internvl" => 0.5, // Boost fort pour noms de modèles
                        "16x" | "32x" | "64x" => 0.3,                   // Boost pour ratios spécifiques
                        _ => 0.2,                                        // Boost standard
                    };

                    // Boost additionnel si le chunk contient des mots explicatifs
                    let has_explanation = self.has_explanatory_context(&content_lower, tech_term);
                    let explanation_bonus = if has_explanation { 0.2 } else { 0.0 };

                    boost += base_boost + explanation_bonus;
                }

                // Variantes avec tirets/espaces
                let variants = vec![
                    tech_term.replace("_", " "),
                    tech_term.replace("_", "-"),
                ];

                for variant in variants {
                    if content_lower.contains(&variant) {
                        boost += 0.3; // Boost pour variantes
                    }
                }
            }
        }

        // Cap à 1.0
        boost.min(1.0_f32)
    }

    /// Détecte si un chunk contient un contexte explicatif autour d'un terme technique
    fn has_explanatory_context(&self, content: &str, tech_term: &str) -> bool {
        // Mots clés qui indiquent une explication
        let explanation_keywords = [
            "permet", "fonction", "role", "rôle", "but", "objectif",
            "utilise", "used", "allows", "enables", "purpose",
            "pour", "for", "afin", "to", "in order",
            "réduire", "reduce", "compress", "compresse",
            "transformer", "transform", "convert", "convertir",
            // Ajout pour données chiffrées et résultats expérimentaux
            "achieve", "atteint", "précision", "accuracy", "precision",
            "résultat", "result", "performance", "measure", "mesure",
            "expérience", "experiment", "evaluation", "évaluation",
            "ratio", "taux", "rate", "level", "niveau",
        ];

        // Chercher si le terme technique apparaît près de mots explicatifs
        if let Some(term_pos) = content.find(tech_term) {
            // Extraire contexte autour du terme (±200 chars)
            let start = term_pos.saturating_sub(200);
            let end = (term_pos + tech_term.len() + 200).min(content.len());
            let context = &content[start..end];

            // Vérifier présence de mots explicatifs dans le contexte
            explanation_keywords.iter().any(|&keyword| context.contains(keyword))
        } else {
            false
        }
    }

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

        let text = "DeepEncoder uses convolutionnel 16x compression";
        let tokens = encoder.enhanced_tokenize(text);

        // Vérifier présence des tokens clés
        assert!(tokens.contains(&"deepencoder".to_string()));
        assert!(tokens.contains(&"convolutionnel".to_string()));
        assert!(tokens.contains(&"16x".to_string()));

        // Vérifier bigrams
        assert!(tokens.iter().any(|t| t.contains("_")));
    }

    #[test]
    fn test_keyword_boost() {
        let encoder = EnhancedBM25Encoder::new();

        let query = "DeepEncoder 16x compression";
        let content_relevant = "The DeepEncoder uses a 16x convolutional compressor";
        let content_irrelevant = "Some other content about transformers";

        let boost_relevant = encoder.keyword_boost(query, content_relevant);
        let boost_irrelevant = encoder.keyword_boost(query, content_irrelevant);

        assert!(boost_relevant > 0.5);
        assert!(boost_irrelevant < 0.1);
    }

    #[test]
    fn test_variants_generation() {
        let encoder = EnhancedBM25Encoder::new();

        let variants = encoder.generate_variants("deepencoder");
        assert!(variants.is_some());

        let variants_vec = variants.unwrap();
        assert!(variants_vec.iter().any(|v| v.contains("encoder")));
    }

    #[test]
    fn test_bm25_scoring() {
        let mut encoder = EnhancedBM25Encoder::new();

        let docs = vec![
            ("doc1".to_string(), "DeepEncoder uses convolutional compression 16x".to_string()),
            ("doc2".to_string(), "InternVL2 uses parallel computation method".to_string()),
            ("doc3".to_string(), "Another document about transformers and attention".to_string()),
        ];

        encoder.index_documents(&docs);

        let query = "DeepEncoder 16x compression";

        let score1 = encoder.score(query, "doc1");
        let score2 = encoder.score(query, "doc2");

        // doc1 devrait scorer plus haut car contient les termes exacts
        assert!(score1 > score2);
    }
}
