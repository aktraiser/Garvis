// GRAVIS Numerical Reranker - Digit-Aware RAG
// Reranking spécialisé pour queries avec contraintes numériques

use regex::Regex;
use tracing::{debug, info};

/// Type de query détecté pour adapter la stratégie de retrieval
#[derive(Debug, Clone, PartialEq)]
pub enum QueryKind {
    /// Query textuelle simple: "DeepEncoder c'est quoi ?"
    TextAtomic,
    /// Query avec plusieurs concepts textuels: "DeepEncoder conv 16x"
    TextCombined,
    /// Query numérique pure: "95.1%", "10.5×"
    DigitAtomic,
    /// Query avec texte + contrainte numérique: "précision < 10x ?"
    DigitCombined,
}

/// Contrainte numérique extraite d'une query
#[derive(Debug, Clone)]
pub enum NumericalConstraint {
    /// Valeur exacte: "10x", "95.1%"
    Exact { value: f32, unit: String },
    /// Inférieur: "< 10x", "inférieur à 10x"
    LessThan { value: f32, unit: String },
    /// Supérieur: "> 10x", "supérieur à 10x"
    GreaterThan { value: f32, unit: String },
    /// Entre deux valeurs: "entre 5x et 10x"
    Between { min: f32, max: f32, unit: String },
}

/// Valeur numérique extraite d'un chunk
#[derive(Debug, Clone)]
pub struct ExtractedValue {
    pub value: f32,
    pub unit: String,
    pub raw_text: String,
    pub position: usize,
}

/// Détecteur de QueryKind et extracteur de contraintes
pub struct QueryKindDetector {
    digit_regex: Regex,
    percentage_regex: Regex,
    compression_regex: Regex,
    constraint_regex: Regex,
}

impl QueryKindDetector {
    pub fn new() -> Self {
        Self {
            // Détecte nombres décimaux
            digit_regex: Regex::new(r"\d+(\.\d+)?").expect("Invalid digit regex"),
            // Détecte pourcentages: "95.1%", "95.1 %"
            percentage_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*%").expect("Invalid percentage regex"),
            // Détecte ratios de compression: "10x", "10×", "10 x"
            compression_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*[x×X]").expect("Invalid compression regex"),
            // Détecte opérateurs de contrainte
            constraint_regex: Regex::new(
                r"(?i)(inférieur|supérieur|moins|plus|greater|less|between|entre|<|>|≤|≥)"
            ).expect("Invalid constraint regex"),
        }
    }

    /// Détecter le type de query
    pub fn detect_query_kind(&self, query: &str) -> QueryKind {
        let has_digits = self.digit_regex.is_match(query);
        let has_percentage = self.percentage_regex.is_match(query);
        let has_compression = self.compression_regex.is_match(query);
        let has_constraint = self.constraint_regex.is_match(query);

        let has_numeric = has_percentage || has_compression;

        // Mots-clés conceptuels
        let conceptual_keywords = [
            "compression", "précision", "accuracy", "precision", "performance",
            "taux", "ratio", "rate", "level", "niveau", "résultat", "result",
            "tokens", "quality", "qualité", "décodage", "decoding",
        ];
        let has_conceptual = conceptual_keywords.iter()
            .any(|&kw| query.to_lowercase().contains(kw));

        debug!("Query analysis: digits={}, numeric={}, constraint={}, conceptual={}",
            has_digits, has_numeric, has_constraint, has_conceptual);

        // Décision
        if has_numeric && (has_constraint || has_conceptual) {
            // "précision < 10x", "quel niveau à 10x compression"
            debug!("🔢 Query kind: DigitCombined");
            QueryKind::DigitCombined
        } else if has_numeric && !has_conceptual {
            // "95.1%", "10.5×" seul
            debug!("🔢 Query kind: DigitAtomic");
            QueryKind::DigitAtomic
        } else if has_conceptual && query.split_whitespace().count() > 5 {
            // "DeepEncoder avec conv 16x et SAM"
            debug!("📝 Query kind: TextCombined");
            QueryKind::TextCombined
        } else {
            // "DeepEncoder c'est quoi ?"
            debug!("📝 Query kind: TextAtomic");
            QueryKind::TextAtomic
        }
    }

    /// Extraire les contraintes numériques d'une query
    pub fn extract_constraints(&self, query: &str) -> Vec<NumericalConstraint> {
        let mut constraints = Vec::new();
        let query_lower = query.to_lowercase();

        // Pattern 1: "< 10x", "> 95%"
        if let Some((op, value, unit)) = self.parse_symbolic_constraint(query) {
            constraints.push(match op {
                '<' | '≤' => NumericalConstraint::LessThan { value, unit },
                '>' | '≥' => NumericalConstraint::GreaterThan { value, unit },
                _ => NumericalConstraint::Exact { value, unit },
            });
        }

        // Pattern 2: "inférieur à 10x", "supérieur à 95%"
        if query_lower.contains("inférieur") || query_lower.contains("less") {
            if let Some((value, unit)) = self.extract_numeric_value(query) {
                constraints.push(NumericalConstraint::LessThan { value, unit });
            }
        } else if query_lower.contains("supérieur") || query_lower.contains("greater") {
            if let Some((value, unit)) = self.extract_numeric_value(query) {
                constraints.push(NumericalConstraint::GreaterThan { value, unit });
            }
        }

        // Pattern 3: "entre 5x et 10x"
        if let Some((min, max, unit)) = self.parse_between_constraint(query) {
            constraints.push(NumericalConstraint::Between { min, max, unit });
        }

        // Pattern 4: Valeur exacte "10x compression", "95.1% précision"
        if constraints.is_empty() {
            if let Some((value, unit)) = self.extract_numeric_value(query) {
                constraints.push(NumericalConstraint::Exact { value, unit });
            }
        }

        if !constraints.is_empty() {
            debug!("Extracted {} constraint(s): {:?}", constraints.len(), constraints);
        }

        constraints
    }

    /// Parser contrainte symbolique: "< 10x", "> 95%"
    fn parse_symbolic_constraint(&self, query: &str) -> Option<(char, f32, String)> {
        let symbolic_regex = Regex::new(
            r"([<>≤≥])\s*(\d+(?:\.\d+)?)\s*([x×X%])"
        ).expect("Invalid symbolic regex");

        if let Some(caps) = symbolic_regex.captures(query) {
            let op = caps.get(1)?.as_str().chars().next()?;
            let value: f32 = caps.get(2)?.as_str().parse().ok()?;
            let unit = caps.get(3)?.as_str().to_lowercase();
            let unit = if unit == "x" || unit == "×" { "x".to_string() } else { "%".to_string() };
            return Some((op, value, unit));
        }
        None
    }

    /// Parser contrainte "entre X et Y"
    fn parse_between_constraint(&self, query: &str) -> Option<(f32, f32, String)> {
        let between_regex = Regex::new(
            r"(?i)(entre|between)\s+(\d+(?:\.\d+)?)[x×X%]?\s+(et|and)\s+(\d+(?:\.\d+)?)\s*([x×X%])"
        ).expect("Invalid between regex");

        if let Some(caps) = between_regex.captures(query) {
            let min: f32 = caps.get(2)?.as_str().parse().ok()?;
            let max: f32 = caps.get(4)?.as_str().parse().ok()?;
            let unit = caps.get(5)?.as_str().to_lowercase();
            let unit = if unit == "x" || unit == "×" { "x".to_string() } else { "%".to_string() };
            return Some((min, max, unit));
        }
        None
    }

    /// Extraire valeur numérique simple (percentage ou compression)
    fn extract_numeric_value(&self, text: &str) -> Option<(f32, String)> {
        // Try percentage first
        if let Some(caps) = self.percentage_regex.captures(text) {
            let value: f32 = caps.get(1)?.as_str().parse().ok()?;
            return Some((value, "%".to_string()));
        }

        // Try compression ratio
        if let Some(caps) = self.compression_regex.captures(text) {
            let value: f32 = caps.get(1)?.as_str().parse().ok()?;
            return Some((value, "x".to_string()));
        }

        None
    }
}

impl Default for QueryKindDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracteur de valeurs numériques dans les chunks
pub struct ChunkValueExtractor {
    percentage_regex: Regex,
    compression_regex: Regex,
}

impl ChunkValueExtractor {
    pub fn new() -> Self {
        Self {
            percentage_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*%").expect("Invalid percentage regex"),
            compression_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*[x×X]").expect("Invalid compression regex"),
        }
    }

    /// Extraire toutes les valeurs numériques d'un chunk
    pub fn extract_values(&self, content: &str) -> Vec<ExtractedValue> {
        let mut values = Vec::new();

        // Extract percentages
        for caps in self.percentage_regex.captures_iter(content) {
            if let Some(m) = caps.get(0) {
                if let Ok(value) = caps.get(1).unwrap().as_str().parse::<f32>() {
                    values.push(ExtractedValue {
                        value,
                        unit: "%".to_string(),
                        raw_text: m.as_str().to_string(),
                        position: m.start(),
                    });
                }
            }
        }

        // Extract compression ratios
        for caps in self.compression_regex.captures_iter(content) {
            if let Some(m) = caps.get(0) {
                if let Ok(value) = caps.get(1).unwrap().as_str().parse::<f32>() {
                    values.push(ExtractedValue {
                        value,
                        unit: "x".to_string(),
                        raw_text: m.as_str().to_string(),
                        position: m.start(),
                    });
                }
            }
        }

        values
    }

    /// Vérifier si un chunk satisfait une contrainte
    pub fn matches_constraint(&self, content: &str, constraint: &NumericalConstraint) -> bool {
        let values = self.extract_values(content);

        for extracted in values {
            let matches_unit = match constraint {
                NumericalConstraint::Exact { unit, .. }
                | NumericalConstraint::LessThan { unit, .. }
                | NumericalConstraint::GreaterThan { unit, .. }
                | NumericalConstraint::Between { unit, .. } => {
                    extracted.unit == *unit
                }
            };

            if !matches_unit {
                continue;
            }

            let satisfies = match constraint {
                NumericalConstraint::Exact { value, .. } => {
                    // Tolérance de ±5% pour match "exact"
                    (extracted.value - value).abs() / value.max(1.0) < 0.05
                }
                NumericalConstraint::LessThan { value, .. } => extracted.value < *value,
                NumericalConstraint::GreaterThan { value, .. } => extracted.value > *value,
                NumericalConstraint::Between { min, max, .. } => {
                    extracted.value >= *min && extracted.value <= *max
                }
            };

            if satisfies {
                return true;
            }
        }

        false
    }
}

impl Default for ChunkValueExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Reranker numérique pour DigitAtomic et DigitCombined
pub struct NumericalReranker {
    detector: QueryKindDetector,
    extractor: ChunkValueExtractor,
}

impl NumericalReranker {
    pub fn new() -> Self {
        Self {
            detector: QueryKindDetector::new(),
            extractor: ChunkValueExtractor::new(),
        }
    }

    /// Reranker pour DigitAtomic: boost chunks qui contiennent la valeur exacte
    ///
    /// Returns: Vec<(chunk_id, score, has_match)> where has_match is true if chunk matches constraint
    pub fn rerank_digit_atomic(
        &self,
        query: &str,
        chunks: Vec<(String, f32)>, // (chunk_id, score)
        chunk_contents: &std::collections::HashMap<String, String>,
    ) -> Vec<(String, f32, bool)> {
        let constraints = self.detector.extract_constraints(query);

        if constraints.is_empty() {
            debug!("No numerical constraints found for DigitAtomic query");
            return chunks.into_iter().map(|(id, score)| (id, score, false)).collect();
        }

        let mut reranked = Vec::new();

        for (chunk_id, score) in chunks {
            let content = match chunk_contents.get(&chunk_id) {
                Some(c) => c,
                None => {
                    reranked.push((chunk_id, score, false));
                    continue;
                }
            };

            let mut has_match = false;

            // Vérifier si le chunk contient la valeur exacte
            for constraint in &constraints {
                if self.extractor.matches_constraint(content, constraint) {
                    has_match = true;
                    break;
                }
            }

            reranked.push((chunk_id, score, has_match));
        }

        // Don't sort here - let caller apply hard priority sorting
        let matched_count = reranked.iter().filter(|(_, _, has_match)| *has_match).count();
        info!("DigitAtomic reranking: {} chunks processed, {} matched", reranked.len(), matched_count);
        reranked
    }

    /// Reranker pour DigitCombined: boost chunks qui satisfont la contrainte numérique
    ///
    /// Returns: Vec<(chunk_id, score, has_match)> where has_match is true if chunk matches constraint
    pub fn rerank_digit_combined(
        &self,
        query: &str,
        chunks: Vec<(String, f32)>,
        chunk_contents: &std::collections::HashMap<String, String>,
    ) -> Vec<(String, f32, bool)> {
        let constraints = self.detector.extract_constraints(query);

        if constraints.is_empty() {
            debug!("No numerical constraints found for DigitCombined query");
            return chunks.into_iter().map(|(id, score)| (id, score, false)).collect();
        }

        let mut reranked = Vec::new();
        let mut matched_count = 0;

        for (chunk_id, score) in chunks {
            let content = match chunk_contents.get(&chunk_id) {
                Some(c) => c,
                None => {
                    reranked.push((chunk_id, score, false));
                    continue;
                }
            };

            let mut has_match = false;

            // Vérifier contraintes
            for constraint in &constraints {
                if self.extractor.matches_constraint(content, constraint) {
                    has_match = true;
                    matched_count += 1;

                    let values = self.extractor.extract_values(content);
                    debug!("✅ Chunk matched constraint! ID: {}, values: {:?}",
                        &chunk_id[..12.min(chunk_id.len())],
                        values.iter().map(|v| format!("{}{}", v.value, v.unit)).collect::<Vec<_>>()
                    );
                    break;
                }
            }

            reranked.push((chunk_id, score, has_match));
        }

        // Don't sort here - let caller apply hard priority sorting
        info!("DigitCombined reranking: {} chunks processed, {} matched constraints: {:?}",
            reranked.len(), matched_count, constraints);
        reranked
    }
}

impl Default for NumericalReranker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_query_kind_text_atomic() {
        let detector = QueryKindDetector::new();

        let query = "DeepEncoder c'est quoi ?";
        assert_eq!(detector.detect_query_kind(query), QueryKind::TextAtomic);

        let query2 = "Qu'est-ce que Gundam mode ?";
        assert_eq!(detector.detect_query_kind(query2), QueryKind::TextAtomic);
    }

    #[test]
    fn test_detect_query_kind_text_combined() {
        let detector = QueryKindDetector::new();

        let query = "DeepEncoder avec conv 16x et SAM dans l'encodeur";
        assert_eq!(detector.detect_query_kind(query), QueryKind::TextCombined);
    }

    #[test]
    fn test_detect_query_kind_digit_atomic() {
        let detector = QueryKindDetector::new();

        let query = "95.1%";
        assert_eq!(detector.detect_query_kind(query), QueryKind::DigitAtomic);

        let query2 = "10.5×";
        assert_eq!(detector.detect_query_kind(query2), QueryKind::DigitAtomic);
    }

    #[test]
    fn test_detect_query_kind_digit_combined() {
        let detector = QueryKindDetector::new();

        let query = "précision de décodage à compression inférieur à 10x";
        assert_eq!(detector.detect_query_kind(query), QueryKind::DigitCombined);

        let query2 = "Quel niveau de précision à 10x compression ?";
        assert_eq!(detector.detect_query_kind(query2), QueryKind::DigitCombined);
    }

    #[test]
    fn test_extract_constraints_less_than() {
        let detector = QueryKindDetector::new();

        let query = "précision inférieur à 10x";
        let constraints = detector.extract_constraints(query);

        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            NumericalConstraint::LessThan { value, unit } => {
                assert_eq!(*value, 10.0);
                assert_eq!(unit, "x");
            }
            _ => panic!("Expected LessThan constraint"),
        }
    }

    #[test]
    fn test_extract_constraints_symbolic() {
        let detector = QueryKindDetector::new();

        let query = "< 10x";
        let constraints = detector.extract_constraints(query);

        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            NumericalConstraint::LessThan { value, unit } => {
                assert_eq!(*value, 10.0);
                assert_eq!(unit, "x");
            }
            _ => panic!("Expected LessThan constraint"),
        }
    }

    #[test]
    fn test_extract_values_from_chunk() {
        let extractor = ChunkValueExtractor::new();

        let content = "Tokens 600–700: 96.5% at 10.5× compression, 98.5% at 6.7×";
        let values = extractor.extract_values(content);

        assert_eq!(values.len(), 4);
        assert!(values.iter().any(|v| v.value == 96.5 && v.unit == "%"));
        assert!(values.iter().any(|v| v.value == 10.5 && v.unit == "x"));
        assert!(values.iter().any(|v| v.value == 98.5 && v.unit == "%"));
        assert!(values.iter().any(|v| v.value == 6.7 && v.unit == "x"));
    }

    #[test]
    fn test_matches_constraint() {
        let extractor = ChunkValueExtractor::new();

        let content = "Tokens 600–700: 96.5% at 10.5× compression, 98.5% at 6.7×";

        // Test LessThan 10x
        let constraint = NumericalConstraint::LessThan {
            value: 10.0,
            unit: "x".to_string(),
        };
        assert!(extractor.matches_constraint(content, &constraint)); // 6.7× < 10x

        // Test GreaterThan 95%
        let constraint2 = NumericalConstraint::GreaterThan {
            value: 95.0,
            unit: "%".to_string(),
        };
        assert!(extractor.matches_constraint(content, &constraint2)); // 96.5% > 95%
    }

    #[test]
    fn test_rerank_digit_combined() {
        let reranker = NumericalReranker::new();

        let chunks = vec![
            ("chunk1".to_string(), 0.5),
            ("chunk2".to_string(), 0.6),
        ];

        let mut contents = std::collections::HashMap::new();
        contents.insert(
            "chunk1".to_string(),
            "Abstract: This paper presents...".to_string(),
        );
        contents.insert(
            "chunk2".to_string(),
            "Table 2: 96.5% at 10.5×, 98.5% at 6.7×".to_string(),
        );

        let query = "précision inférieur à 10x";
        let reranked = reranker.rerank_digit_combined(query, chunks, &contents);

        // chunk2 (table) devrait scorer plus haut que chunk1 (abstract)
        assert_eq!(reranked[0].0, "chunk2");
        assert!(reranked[0].1 > 0.6); // Boost appliqué
    }
}
