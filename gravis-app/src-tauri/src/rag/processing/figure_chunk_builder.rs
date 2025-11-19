// GRAVIS Figure Chunk Builder - Vision-Aware RAG Phase 3
// Construction de chunks enrichis à partir de figures détectées

use crate::rag::{
    EnrichedChunk, ChunkType, ChunkMetadata, ChunkSource, Priority,
    SourceType, ExtractionMethod
};
use super::{DetectedFigure, FigureOcrExtractor, FigureDetector};
use std::path::Path;
use tracing::{debug, info, warn};

/// Builder pour créer des chunks à partir de figures
pub struct FigureChunkBuilder {
    detector: FigureDetector,
    ocr_extractor: Option<FigureOcrExtractor>,
}

impl FigureChunkBuilder {
    /// Créer un builder sans OCR (captions seulement)
    pub fn new() -> Self {
        Self {
            detector: FigureDetector::new(),
            ocr_extractor: None,
        }
    }

    /// Créer un builder avec OCR activé
    pub async fn with_ocr() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            detector: FigureDetector::new(),
            ocr_extractor: Some(FigureOcrExtractor::new().await?),
        })
    }

    /// Traiter une page de texte et générer des chunks de figures
    ///
    /// # Arguments
    /// * `page_text` - Texte extrait de la page
    /// * `page_index` - Index de la page (0-based)
    /// * `page_image_path` - Optionnel: chemin vers l'image de la page pour OCR
    /// * `group_id` - ID du groupe de documents
    ///
    /// # Returns
    /// Vec de chunks générés (captions + optionnel OCR)
    pub async fn build_figure_chunks_for_page(
        &self,
        page_text: &str,
        page_index: u32,
        page_image_path: Option<&Path>,
        group_id: &str,
    ) -> Result<Vec<EnrichedChunk>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();

        // 1. Détecter les figures dans le texte
        let detected_figures = self.detector.detect_figures_in_page(page_text, page_index);

        if detected_figures.is_empty() {
            return Ok(chunks);
        }

        info!(
            "Building chunks for {} figure(s) on page {}",
            detected_figures.len(),
            page_index + 1
        );

        for figure in detected_figures {
            // 2. Créer chunk pour la légende
            let caption_chunk = self.create_caption_chunk(&figure, group_id);
            chunks.push(caption_chunk);

            // 3. Si OCR activé et image disponible, créer chunk OCR
            if let (Some(ocr), Some(image_path)) = (&self.ocr_extractor, page_image_path) {
                match self
                    .create_ocr_chunk(&figure, image_path, group_id)
                    .await
                {
                    Ok(Some(ocr_chunk)) => chunks.push(ocr_chunk),
                    Ok(None) => {}
                    Err(e) => {
                        warn!(
                            "Failed to create OCR chunk for {}: {}",
                            figure.figure_id, e
                        );
                    }
                }
            }
        }

        Ok(chunks)
    }

    /// Créer un chunk pour une légende de figure
    fn create_caption_chunk(&self, figure: &DetectedFigure, group_id: &str) -> EnrichedChunk {
        let chunk_id = format!(
            "fig_caption_{}_p{}",
            figure.figure_id.replace(' ', "_"),
            figure.page_index
        );

        let content = format!(
            "[FIGURE CAPTION - Page {}]\n{}",
            figure.page_index + 1,
            figure.caption
        );

        EnrichedChunk {
            id: chunk_id,
            content,
            start_line: figure.page_index as usize, // Approximation
            end_line: figure.page_index as usize,
            chunk_type: ChunkType::TextBlock,
            embedding: None, // Sera généré plus tard
            hash: blake3::hash(figure.caption.as_bytes()).to_hex().to_string(),
            metadata: ChunkMetadata {
                tags: vec![
                    "figure".to_string(),
                    "caption".to_string(),
                    figure.figure_type.as_str().to_lowercase(),
                ],
                priority: Priority::High, // Légendes importantes
                language: "auto".to_string(),
                symbol: None,
                context: Some(format!("Page {}", figure.page_index + 1)),
                confidence: 1.0, // Légende détectée avec regex = haute confiance
                ocr_metadata: None,
                source_type: SourceType::NativeText,
                extraction_method: ExtractionMethod::DirectRead,
            },
            group_id: group_id.to_string(),
            source_spans: None,
            chunk_source: ChunkSource::FigureCaption,
            figure_id: Some(figure.figure_id.clone()),
        }
    }

    /// Créer un chunk OCR pour une figure
    async fn create_ocr_chunk(
        &self,
        figure: &DetectedFigure,
        image_path: &Path,
        group_id: &str,
    ) -> Result<Option<EnrichedChunk>, Box<dyn std::error::Error>> {
        let ocr = self.ocr_extractor.as_ref().unwrap();

        // OCR de la page complète (v1 simple)
        let ocr_text = match ocr.ocr_page_for_figures(image_path, figure.page_index).await {
            Ok(text) => text,
            Err(e) => {
                warn!("OCR failed for page {}: {}", figure.page_index + 1, e);
                return Ok(None);
            }
        };

        // Filtrer pour garder que les données numériques
        let filtered_text = ocr.filter_numeric_data(&ocr_text);

        if filtered_text.trim().is_empty() {
            debug!("No numeric data found in OCR for {}", figure.figure_id);
            return Ok(None);
        }

        // Extraire paires clé-valeur si possible
        let key_values = ocr.extract_key_value_pairs(&filtered_text);
        let kv_summary = if !key_values.is_empty() {
            let kv_lines: Vec<String> = key_values
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            format!("\n\nExtracted values:\n{}", kv_lines.join("\n"))
        } else {
            String::new()
        };

        let chunk_id = format!(
            "fig_ocr_{}_p{}",
            figure.figure_id.replace(' ', "_"),
            figure.page_index
        );

        let content = format!(
            "[FIGURE OCR - {} - Page {}]\n{}{}\n\n⚠️ Note: Data extracted via OCR from graphic. Verify visually for exact values.",
            figure.figure_id,
            figure.page_index + 1,
            filtered_text,
            kv_summary
        );

        Ok(Some(EnrichedChunk {
            id: chunk_id,
            content,
            start_line: figure.page_index as usize,
            end_line: figure.page_index as usize,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: blake3::hash(filtered_text.as_bytes()).to_hex().to_string(),
            metadata: ChunkMetadata {
                tags: vec![
                    "figure".to_string(),
                    "ocr".to_string(),
                    "numeric_data".to_string(),
                    figure.figure_type.as_str().to_lowercase(),
                ],
                priority: Priority::Normal,
                language: "auto".to_string(),
                symbol: None,
                context: Some(format!("Page {} - OCR extraction", figure.page_index + 1)),
                confidence: 0.7, // OCR = confiance moyenne
                ocr_metadata: None,
                source_type: SourceType::OcrExtracted,
                extraction_method: ExtractionMethod::TesseractOcr {
                    confidence: 0.7,
                    language: "eng+fra".to_string(),
                },
            },
            group_id: group_id.to_string(),
            source_spans: None,
            chunk_source: ChunkSource::FigureRegionText,
            figure_id: Some(figure.figure_id.clone()),
        }))
    }

    /// Traiter un document complet et générer tous les chunks de figures
    ///
    /// # Arguments
    /// * `pages` - Vec de (page_index, page_text, optional_image_path)
    /// * `group_id` - ID du groupe
    pub async fn build_all_figure_chunks(
        &self,
        pages: Vec<(u32, String, Option<std::path::PathBuf>)>,
        group_id: &str,
    ) -> Result<Vec<EnrichedChunk>, Box<dyn std::error::Error>> {
        let mut all_chunks = Vec::new();

        for (page_index, page_text, image_path) in pages {
            let page_chunks = self
                .build_figure_chunks_for_page(
                    &page_text,
                    page_index,
                    image_path.as_deref(),
                    group_id,
                )
                .await?;

            all_chunks.extend(page_chunks);
        }

        if !all_chunks.is_empty() {
            info!(
                "Built total of {} figure chunks for document",
                all_chunks.len()
            );
        }

        Ok(all_chunks)
    }
}

impl Default for FigureChunkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_caption_chunk() {
        let builder = FigureChunkBuilder::new();

        let figure = DetectedFigure {
            figure_id: "Figure 3".to_string(),
            figure_type: super::super::FigureType::Figure,
            number: "3".to_string(),
            caption: "Figure 3: Compression ratio vs accuracy".to_string(),
            page_index: 5,
            text_position: 100,
        };

        let chunk = builder.create_caption_chunk(&figure, "test_group");

        assert_eq!(chunk.chunk_source, ChunkSource::FigureCaption);
        assert_eq!(chunk.figure_id, Some("Figure 3".to_string()));
        assert!(chunk.content.contains("Compression ratio"));
        assert!(chunk.metadata.tags.contains(&"caption".to_string()));
    }

    #[tokio::test]
    async fn test_build_figure_chunks_no_figures() {
        let builder = FigureChunkBuilder::new();

        let page_text = "This is some regular text with no figures.";

        let chunks = builder
            .build_figure_chunks_for_page(page_text, 0, None, "test_group")
            .await
            .unwrap();

        assert!(chunks.is_empty());
    }

    #[tokio::test]
    async fn test_build_figure_chunks_with_caption() {
        let builder = FigureChunkBuilder::new();

        let page_text = r#"
Some text here.

Figure 1: Test caption for compression analysis

More text after.
        "#;

        let chunks = builder
            .build_figure_chunks_for_page(page_text, 0, None, "test_group")
            .await
            .unwrap();

        assert_eq!(chunks.len(), 1); // Only caption, no OCR
        assert_eq!(chunks[0].chunk_source, ChunkSource::FigureCaption);
        assert!(chunks[0].content.contains("Test caption"));
    }
}
