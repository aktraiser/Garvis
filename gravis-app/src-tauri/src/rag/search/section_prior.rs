// Section Prior - Reranking simple et générique basé sur le type de section
// Remplace les filtres 3-pass over-engineered par une approche minimale et robuste
//
// Principe:
// - Boost chunks venant de sections stratégiques (Abstract, Intro, Conclusion)
// - Penalty chunks venant de sections techniques (Benchmarks, Tables, Related Work)
// - Hard drop contamination évidente (bibliographie, hallucinations OCR)

use tracing::debug;

/// Reranker basé sur section prior + contamination filter
pub struct SectionPriorReranker;

impl SectionPriorReranker {
    /// Apply section-based reranking + contamination filtering
    pub fn rerank_and_filter<T>(
        items: Vec<(T, f32)>,  // (item, original_score)
        get_content: impl Fn(&T) -> &str,
        get_source_type: impl Fn(&T) -> &str,
    ) -> Vec<(T, f32)> {
        let mut reranked: Vec<(T, f32)> = items
            .into_iter()
            .filter_map(|(item, score)| {
                let content = get_content(&item);
                let source_type = get_source_type(&item);

                // HARD DROP contamination évidente
                if Self::is_contaminated(content, source_type) {
                    debug!("🚫 Dropped contaminated chunk: {}", &content[..50.min(content.len())]);
                    return None;  // Éliminer complètement
                }

                // Section prior adjustment
                let section_boost = Self::section_prior_boost(content, source_type);
                let final_score = (score + section_boost).max(0.0).min(1.0);

                Some((item, final_score))
            })
            .collect();

        // Trier par score final (desc)
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        reranked
    }

    /// Détection contamination évidente (bibliographie, hallucinations OCR, etc.)
    fn is_contaminated(content: &str, source_type: &str) -> bool {
        let content_lower = content.to_lowercase();

        // 1. Bibliographie / Références
        let bib_patterns = ["et al.", "arxiv", "preprint", "doi:", "http://", "https://"];
        let bib_match_count = bib_patterns.iter().filter(|p| content_lower.contains(*p)).count();
        if bib_match_count >= 3 {
            return true;  // Chunk de bibliographie
        }

        // 2. Hallucinations OCR visuelles (furniture, library, room, shelves)
        let visual_hallucination_patterns = ["library", "room", "furniture", "shelves", "dedicated to books"];
        let visual_match_count = visual_hallucination_patterns.iter()
            .filter(|p| content_lower.contains(*p))
            .count();
        if visual_match_count >= 2 {
            return true;  // Hallucination OCR (description visuelle hors-sujet)
        }

        // 3. Figure/Table caption avec très peu de contenu utile
        if source_type.contains("Figure Caption") || source_type.contains("Table Caption") {
            if content.len() < 100 {  // Caption trop court = peu informatif
                return true;
            }
        }

        false
    }

    /// Section prior: boost/penalty selon type de section
    fn section_prior_boost(content: &str, source_type: &str) -> f32 {
        let content_lower = content.to_lowercase();

        // BOOST sections stratégiques (+0.10 à +0.15)
        if content_lower.starts_with("abstract") || content_lower.contains("in this paper we") {
            return 0.15;  // Abstract = très stratégique
        }
        if content_lower.contains("introduction") && content_lower.contains("we propose") {
            return 0.12;  // Introduction avec objectif
        }
        if content_lower.contains("conclusion") || content_lower.contains("in summary") {
            return 0.10;  // Conclusion
        }

        // PENALTY sections techniques/benchmarks (-0.10 à -0.20)
        if source_type.contains("Table") && content_lower.contains("benchmark") {
            return -0.15;  // Table de benchmark
        }
        if content_lower.contains("experiments") || content_lower.contains("evaluation") {
            return -0.10;  // Section expérimentale
        }
        if source_type.contains("Figure Caption") {
            return -0.05;  // Figure caption (souvent moins informatif pour objectifs)
        }

        // PENALTY FORTE si liste de modèles (Qwen, OLMOCR, etc.)
        let model_list_patterns = ["qwen", "olmocr", "internvl", "mineru"];
        let model_match_count = model_list_patterns.iter()
            .filter(|p| content_lower.contains(*p))
            .count();
        if model_match_count >= 2 {
            return -0.20;  // Probablement une table de comparaison
        }

        0.0  // Neutre par défaut
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contamination_detection() {
        // Bibliographie
        let bib_chunk = "Smith et al. (2020). arxiv:2010.12345. doi:10.1234/test. https://example.com";
        assert!(SectionPriorReranker::is_contaminated(bib_chunk, "Document Text"));

        // Hallucination OCR
        let visual_chunk = "The room may be part of a library with furniture and shelves dedicated to books";
        assert!(SectionPriorReranker::is_contaminated(visual_chunk, "Document Text"));

        // Caption court
        let short_caption = "Figure 3: Model";
        assert!(SectionPriorReranker::is_contaminated(short_caption, "Figure Caption"));
    }

    #[test]
    fn test_section_prior_boost() {
        // Abstract = boost fort
        let abstract_chunk = "Abstract: In this paper we propose a novel approach...";
        assert_eq!(SectionPriorReranker::section_prior_boost(abstract_chunk, "Document Text"), 0.15);

        // Table benchmark = penalty
        let bench_chunk = "Table 3 shows benchmark results on OmniDocBench...";
        assert_eq!(SectionPriorReranker::section_prior_boost(bench_chunk, "Table"), -0.15);

        // Liste modèles = penalty forte
        let models_chunk = "We compare against Qwen, OLMOCR, InternVL, and MinerU...";
        assert_eq!(SectionPriorReranker::section_prior_boost(models_chunk, "Document Text"), -0.20);
    }
}
