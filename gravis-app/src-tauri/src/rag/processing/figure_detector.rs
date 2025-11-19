// GRAVIS Figure Detector - Vision-Aware RAG Phase 3
// Détection de légendes de figures et tables dans le texte extrait

use regex::Regex;
use tracing::{debug, info};

/// Information sur une figure ou table détectée
#[derive(Debug, Clone)]
pub struct DetectedFigure {
    /// ID de la figure (ex: "Figure 3", "Table 1")
    pub figure_id: String,
    /// Type de figure
    pub figure_type: FigureType,
    /// Numéro extrait
    pub number: String,
    /// Légende complète
    pub caption: String,
    /// Page où se trouve la figure
    pub page_index: u32,
    /// Position approximative dans le texte de la page (offset en chars)
    pub text_position: usize,
}

/// Type de figure détectée
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FigureType {
    Figure,
    Table,
    Chart,
    Graph,
}

impl FigureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FigureType::Figure => "Figure",
            FigureType::Table => "Table",
            FigureType::Chart => "Chart",
            FigureType::Graph => "Graph",
        }
    }
}

/// Détecteur de figures dans le texte
pub struct FigureDetector {
    /// Regex pour détecter les légendes de figures
    figure_regex: Regex,
    /// Regex pour détecter les tables
    table_regex: Regex,
}

impl FigureDetector {
    /// Créer un nouveau détecteur
    pub fn new() -> Self {
        // Pattern pour figures (multilingue)
        // Exemples:
        // - "Figure 3: Compression ratio"
        // - "Fig. 2. Model architecture"
        // - "Figure 1 - Results"
        let figure_regex = Regex::new(
            r"(?mi)^[ \t]*(Figure|Fig\.?|Graphique|Graph)\s+(\d+)[\.:–\-\s]+(.+?)$"
        ).expect("Failed to compile figure regex");

        // Pattern pour tables
        // Exemples:
        // - "Table 1: Benchmark results"
        // - "Tableau 2. Performance metrics"
        let table_regex = Regex::new(
            r"(?mi)^[ \t]*(Table|Tableau|Tab\.?)\s+(\d+)[\.:–\-\s]+(.+?)$"
        ).expect("Failed to compile table regex");

        Self {
            figure_regex,
            table_regex,
        }
    }

    /// Détecter toutes les figures dans le texte d'une page
    pub fn detect_figures_in_page(
        &self,
        page_text: &str,
        page_index: u32,
    ) -> Vec<DetectedFigure> {
        let mut figures = Vec::new();

        // Détecter les figures
        for caps in self.figure_regex.captures_iter(page_text) {
            let match_str = caps.get(0).unwrap();
            let kind = caps.get(1).unwrap().as_str();
            let number = caps.get(2).unwrap().as_str().to_string();
            let caption_rest = caps.get(3).unwrap().as_str().trim().to_string();

            let figure_type = if kind.to_lowercase().contains("graph") {
                FigureType::Graph
            } else {
                FigureType::Figure
            };

            let figure_id = format!("{} {}", figure_type.as_str(), number);
            let full_caption = format!("{}: {}", figure_id, caption_rest);

            figures.push(DetectedFigure {
                figure_id,
                figure_type,
                number,
                caption: full_caption,
                page_index,
                text_position: match_str.start(),
            });
        }

        // Détecter les tables
        for caps in self.table_regex.captures_iter(page_text) {
            let match_str = caps.get(0).unwrap();
            let _kind = caps.get(1).unwrap().as_str();
            let number = caps.get(2).unwrap().as_str().to_string();
            let caption_rest = caps.get(3).unwrap().as_str().trim().to_string();

            let figure_id = format!("Table {}", number);
            let full_caption = format!("{}: {}", figure_id, caption_rest);

            figures.push(DetectedFigure {
                figure_id,
                figure_type: FigureType::Table,
                number,
                caption: full_caption,
                page_index,
                text_position: match_str.start(),
            });
        }

        if !figures.is_empty() {
            debug!(
                "Detected {} figure(s)/table(s) on page {}",
                figures.len(),
                page_index + 1
            );
        }

        figures
    }

    /// Détecter toutes les figures dans un document complet
    pub fn detect_figures_in_document(
        &self,
        pages_text: &[(u32, String)], // (page_index, text)
    ) -> Vec<DetectedFigure> {
        let mut all_figures = Vec::new();

        for (page_index, page_text) in pages_text {
            let page_figures = self.detect_figures_in_page(page_text, *page_index);
            all_figures.extend(page_figures);
        }

        if !all_figures.is_empty() {
            info!(
                "Detected total of {} figures/tables in document",
                all_figures.len()
            );
        }

        all_figures
    }

    /// Extraire le contexte autour d'une figure (pour analyse future)
    pub fn extract_figure_context(
        &self,
        page_text: &str,
        figure: &DetectedFigure,
        context_chars: usize,
    ) -> String {
        let start = figure.text_position.saturating_sub(context_chars);
        let end = (figure.text_position + figure.caption.len() + context_chars)
            .min(page_text.len());

        page_text[start..end].to_string()
    }
}

impl Default for FigureDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_figure_basic() {
        let detector = FigureDetector::new();
        let text = r#"
The results are shown below.

Figure 3: Compression ratio vs accuracy

As we can see, the model achieves...
        "#;

        let figures = detector.detect_figures_in_page(text, 0);
        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].number, "3");
        assert_eq!(figures[0].figure_type, FigureType::Figure);
        assert!(figures[0].caption.contains("Compression ratio"));
    }

    #[test]
    fn test_detect_table() {
        let detector = FigureDetector::new();
        let text = r#"
Performance metrics are summarized below.

Table 1: Benchmark results on ImageNet

The table shows...
        "#;

        let figures = detector.detect_figures_in_page(text, 0);
        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].figure_type, FigureType::Table);
        assert_eq!(figures[0].number, "1");
    }

    #[test]
    fn test_detect_multiple() {
        let detector = FigureDetector::new();
        let text = r#"
Figure 1: Architecture overview
Some text here.
Table 1: Performance metrics
More text.
Figure 2. Detailed results
        "#;

        let figures = detector.detect_figures_in_page(text, 0);
        assert_eq!(figures.len(), 3);
        assert_eq!(figures[0].number, "1");
        assert_eq!(figures[1].figure_type, FigureType::Table);
        assert_eq!(figures[2].number, "2");
    }

    #[test]
    fn test_french_detection() {
        let detector = FigureDetector::new();
        let text = r#"
Graphique 1: Résultats de compression
Tableau 2: Métriques de performance
        "#;

        let figures = detector.detect_figures_in_page(text, 0);
        assert_eq!(figures.len(), 2);
        assert_eq!(figures[0].figure_type, FigureType::Graph);
        assert_eq!(figures[1].figure_type, FigureType::Table);
    }

    #[test]
    fn test_extract_context() {
        let detector = FigureDetector::new();
        let text = "Before text. Figure 1: Caption here. After text.";

        let figures = detector.detect_figures_in_page(text, 0);
        let context = detector.extract_figure_context(text, &figures[0], 20);

        assert!(context.contains("Before"));
        assert!(context.contains("Figure 1"));
        assert!(context.contains("After"));
    }
}
