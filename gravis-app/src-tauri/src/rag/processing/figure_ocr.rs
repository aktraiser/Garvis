// GRAVIS Figure OCR Extractor - Vision-Aware RAG Phase 3
// Extraction OCR ciblée pour figures et graphiques

use crate::rag::ocr::{TesseractProcessor, TesseractConfig, OcrError};
use image::DynamicImage;
use std::path::Path;
use tracing::{debug, warn};

/// Configuration pour l'OCR de figures
#[derive(Debug, Clone)]
pub struct FigureOcrConfig {
    /// Whitelist de caractères pour données chiffrées (nombres, %, x, etc.)
    pub char_whitelist: Option<String>,
    /// Seuil de confiance minimum
    pub confidence_threshold: f32,
    /// Languages à utiliser
    pub languages: Vec<String>,
}

impl Default for FigureOcrConfig {
    fn default() -> Self {
        Self {
            // Optimisé pour graphiques avec données numériques
            char_whitelist: Some(
                "0123456789.%xX-+abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ "
                    .to_string(),
            ),
            confidence_threshold: 0.5, // Plus permissif pour chiffres dans graphiques
            languages: vec!["eng".to_string(), "fra".to_string()],
        }
    }
}

/// Extracteur OCR pour figures
pub struct FigureOcrExtractor {
    tesseract: TesseractProcessor,
    config: FigureOcrConfig,
}

impl FigureOcrExtractor {
    /// Créer un nouvel extracteur avec configuration par défaut
    pub async fn new() -> Result<Self, OcrError> {
        let config = FigureOcrConfig::default();
        Self::with_config(config).await
    }

    /// Créer un extracteur avec configuration personnalisée
    pub async fn with_config(config: FigureOcrConfig) -> Result<Self, OcrError> {
        let mut tesseract_config = TesseractConfig::default();
        tesseract_config.languages = config.languages.clone();
        tesseract_config.confidence_threshold = config.confidence_threshold;

        let tesseract = TesseractProcessor::new(tesseract_config).await?;

        Ok(Self { tesseract, config })
    }

    /// OCR d'une page complète (stratégie simple - v1)
    ///
    /// Dans v1, on OCR toute la page car:
    /// - Pas besoin de détection de bbox complexe
    /// - Tesseract va extraire le texte des zones avec texte
    /// - On filtre ensuite par proximité textuelle avec la légende
    pub async fn ocr_page_for_figures(
        &self,
        image_path: &Path,
        page_index: u32,
    ) -> Result<String, OcrError> {
        debug!(
            "Running OCR on page {} for figure extraction: {:?}",
            page_index + 1,
            image_path
        );

        // Appel Tesseract via le processeur existant
        let ocr_result = self.tesseract.process_image(image_path).await?;

        if ocr_result.confidence < self.config.confidence_threshold {
            warn!(
                "Low confidence OCR on page {}: {:.2}%",
                page_index + 1,
                ocr_result.confidence * 100.0
            );
        }

        Ok(ocr_result.text)
    }

    /// OCR d'une image en mémoire (future: pour crop de régions spécifiques)
    pub async fn ocr_image_region(
        &self,
        image: &DynamicImage,
        figure_id: &str,
    ) -> Result<String, OcrError> {
        debug!("Running OCR on image region for {}", figure_id);

        // Pour v1, on sauvegarde temporairement l'image
        let temp_path = std::env::temp_dir().join(format!("gravis_fig_ocr_{}.png", figure_id));

        if let Err(e) = image.save(&temp_path) {
            return Err(OcrError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save temp image: {}", e),
            )));
        }

        let result = self.tesseract.process_image(&temp_path).await;

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);

        result.map(|r| r.text)
    }

    /// Filtrer le texte OCR pour ne garder que les données numériques pertinentes
    ///
    /// Utile pour réduire le bruit et se concentrer sur les valeurs chiffrées
    pub fn filter_numeric_data(&self, ocr_text: &str) -> String {
        ocr_text
            .lines()
            .filter(|line| {
                // Garder les lignes contenant des chiffres ou symboles importants
                line.chars().any(|c| {
                    c.is_numeric()
                        || c == '%'
                        || c == 'x'
                        || c == 'X'
                        || c == '.'
                        || c == ','
                })
            })
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extraire les paires clé-valeur numériques (ex: "Accuracy: 95.1%")
    pub fn extract_key_value_pairs(&self, ocr_text: &str) -> Vec<(String, String)> {
        use regex::Regex;

        let kv_regex = Regex::new(
            r"(?i)(accuracy|precision|recall|f1|compression|ratio|level|rate)[\s:=]+([0-9.]+\s*%?)"
        ).expect("Invalid key-value regex");

        let mut pairs = Vec::new();

        for caps in kv_regex.captures_iter(ocr_text) {
            let key = caps.get(1).unwrap().as_str().to_string();
            let value = caps.get(2).unwrap().as_str().trim().to_string();
            pairs.push((key, value));
        }

        if !pairs.is_empty() {
            debug!("Extracted {} key-value pairs from OCR", pairs.len());
        }

        pairs
    }
}

// Note: Default impl removed because new() is async
// Use FigureOcrExtractor::new().await instead

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filter_numeric_data() {
        let extractor = FigureOcrExtractor::new().await.unwrap();

        let ocr_text = r#"
Some random text
Accuracy 95.1%
More text here
Compression: 10x
Irrelevant line
Precision 0.87
        "#;

        let filtered = extractor.filter_numeric_data(ocr_text);

        assert!(filtered.contains("95.1%"));
        assert!(filtered.contains("10x"));
        assert!(filtered.contains("0.87"));
        assert!(!filtered.contains("Irrelevant"));
    }

    #[tokio::test]
    async fn test_extract_key_value_pairs() {
        let extractor = FigureOcrExtractor::new().await.unwrap();

        let ocr_text = r#"
The model achieves:
Accuracy: 95.1%
Compression ratio = 10x
Precision 87.5%
F1 score: 0.92
        "#;

        let pairs = extractor.extract_key_value_pairs(ocr_text);

        assert_eq!(pairs.len(), 4);
        assert!(pairs.iter().any(|(k, v)| k == "Accuracy" && v == "95.1%"));
        assert!(pairs
            .iter()
            .any(|(k, v)| k.to_lowercase().contains("compression") && v.contains("10")));
    }

    #[test]
    fn test_char_whitelist_config() {
        let config = FigureOcrConfig::default();
        assert!(config.char_whitelist.is_some());

        let whitelist = config.char_whitelist.unwrap();
        assert!(whitelist.contains('0'));
        assert!(whitelist.contains('%'));
        assert!(whitelist.contains('x'));
    }
}
