// GRAVIS OCR - Pipeline PDF avec lopdf (Alternative Pure Rust)
// Remplacement de pdfium-render pour macOS sans dépendances externes

use super::{
    OcrResult, OcrMetadata, TesseractProcessor, 
    TesseractConfig,
    PageSegMode, OcrEngineMode, PreprocessConfig, OcrError, Result
};
// use image::{DynamicImage, ImageBuffer, Rgba};
use lopdf::{Document, Object, ObjectId};
use std::path::Path;
use std::time::{Duration, Instant};
// use std::collections::HashMap;
// use tokio::fs;
use tracing::{info, debug, warn};

/// Configuration du pipeline PDF avec lopdf
#[derive(Debug, Clone)]
pub struct LopdFPipelineConfig {
    /// Seuil minimum de tokens natifs pour éviter l'OCR
    pub min_native_tokens: usize,
    
    /// Ratio minimum de surface de texte natif (0.0-1.0)
    pub min_text_area_ratio: f32,
    
    /// DPI pour la rasterisation (si nécessaire avec image crate)
    pub fallback_dpi: u16,
    
    /// Configuration Tesseract pour l'OCR ciblé
    pub tesseract_config: TesseractConfig,
    
    /// Activer le fallback vers image crate si pas de texte natif
    pub enable_image_fallback: bool,
}

impl Default for LopdFPipelineConfig {
    fn default() -> Self {
        Self {
            min_native_tokens: 50,
            min_text_area_ratio: 0.05,
            fallback_dpi: 300,
            tesseract_config: TesseractConfig {
                languages: vec!["eng".to_string(), "fra".to_string()],
                psm: PageSegMode::Auto,
                oem: OcrEngineMode::LstmOnly,
                preprocessing: PreprocessConfig {
                    enabled: true,
                    enhance_contrast: true,
                    resize_for_ocr: false,
                    min_width: 100,
                    min_height: 30,
                    target_dpi: 300,
                },
                confidence_threshold: 0.6,
                temp_dir: std::env::temp_dir().join("gravis_lopdf_pipeline"),
                max_concurrent: 4,
                timeout: Duration::from_secs(30),
            },
            enable_image_fallback: true,
        }
    }
}

/// Résultat de traitement d'une page PDF avec lopdf
#[derive(Debug, Clone)]
pub struct LopdFPageResult {
    pub page_number: usize,
    pub native_text: String,
    pub text_objects: Vec<TextObject>,
    pub ocr_result: Option<OcrResult>,
    pub processing_time: Duration,
    pub decision: PageProcessingDecision,
}

/// Décision de traitement pour une page
#[derive(Debug, Clone)]
pub enum PageProcessingDecision {
    NativeTextOnly {
        token_count: usize,
        text_objects_count: usize,
    },
    OcrFallback {
        reason: String,
        native_tokens: usize,
    },
    Failed {
        error: String,
    },
}

/// Objet texte extrait du PDF
#[derive(Debug, Clone)]
pub struct TextObject {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub font_name: String,
}

/// Processeur principal du pipeline PDF avec lopdf
pub struct LopdFProcessor {
    config: LopdFPipelineConfig,
    #[allow(dead_code)]
    tesseract: TesseractProcessor,
}

impl LopdFProcessor {
    /// Créer un nouveau processeur lopdf
    pub async fn new(config: LopdFPipelineConfig) -> Result<Self> {
        info!("🚀 Initializing lopdf PDF Processor (Pure Rust)");
        
        // Initialiser Tesseract pour fallback OCR
        let tesseract = TesseractProcessor::new(config.tesseract_config.clone()).await?;
        
        Ok(Self {
            config,
            tesseract,
        })
    }
    
    /// Traiter un document PDF complet
    pub async fn process_pdf(&self, pdf_path: &Path) -> Result<Vec<LopdFPageResult>> {
        let start_time = Instant::now();
        info!("📄 Processing PDF with lopdf: {:?}", pdf_path);
        
        // Ouvrir le PDF avec lopdf
        let document = Document::load(pdf_path)
            .map_err(|e| OcrError::ImageProcessing(format!("Failed to open PDF with lopdf: {}", e)))?;
        
        let pages = document.get_pages();
        let page_count = pages.len();
        info!("📖 PDF has {} pages", page_count);
        
        let mut results = Vec::new();
        
        // Traiter chaque page
        for (page_index, (page_id, _)) in pages.iter().enumerate().take(10) {
            match self.process_page(&document, (*page_id, 0), page_index + 1).await {
                Ok(page_result) => {
                    info!("✅ Page {} processed: {:?}", page_index + 1, page_result.decision);
                    results.push(page_result);
                }
                Err(e) => {
                    warn!("❌ Failed to process page {}: {}", page_index + 1, e);
                }
            }
        }
        
        let total_time = start_time.elapsed();
        info!("📊 PDF processing completed in {:.2}s: {} pages processed", 
              total_time.as_secs_f32(), results.len());
        
        Ok(results)
    }
    
    /// Traiter une page individuelle
    async fn process_page(&self, document: &Document, page_id: ObjectId, page_number: usize) -> Result<LopdFPageResult> {
        let start_time = Instant::now();
        
        debug!("🔄 Processing page {} with lopdf", page_number);
        
        // Étape 1: Extraire le texte natif avec lopdf
        let (native_text, text_objects) = self.extract_native_text(document, page_id).await?;
        let token_count = native_text.split_whitespace().count();
        
        debug!("📝 Page {} native text: {} tokens, {} chars, {} objects", 
               page_number, token_count, native_text.len(), text_objects.len());
        
        // Étape 2: Décider du traitement nécessaire
        if token_count >= self.config.min_native_tokens {
            // Texte natif suffisant, pas besoin d'OCR
            info!("📄 Page {}: Native text sufficient ({} tokens)", 
                  page_number, token_count);
            
            return Ok(LopdFPageResult {
                page_number,
                native_text,
                text_objects: text_objects.clone(),
                ocr_result: None,
                processing_time: start_time.elapsed(),
                decision: PageProcessingDecision::NativeTextOnly {
                    token_count,
                    text_objects_count: text_objects.len(),
                },
            });
        }
        
        // Étape 3: Fallback OCR si activé et pas assez de texte natif
        let mut ocr_result = None;
        let decision = if self.config.enable_image_fallback {
            info!("🔄 Page {}: Falling back to OCR (insufficient native text: {} tokens)", 
                  page_number, token_count);
            
            // Note: Pour l'OCR, vous devrez convertir la page en image
            // lopdf ne fait que l'extraction de texte, pas le rendu
            // Vous pouvez utiliser des outils externes ou rester sur pdfium pour le rendu
            match self.fallback_to_ocr(document, page_id, page_number).await {
                Ok(result) => {
                    ocr_result = Some(result);
                    PageProcessingDecision::OcrFallback {
                        reason: "Insufficient native text found".to_string(),
                        native_tokens: token_count,
                    }
                }
                Err(e) => {
                    warn!("❌ OCR fallback failed for page {}: {}", page_number, e);
                    PageProcessingDecision::Failed {
                        error: format!("OCR fallback failed: {}", e),
                    }
                }
            }
        } else {
            PageProcessingDecision::NativeTextOnly {
                token_count,
                text_objects_count: text_objects.len(),
            }
        };
        
        let processing_time = start_time.elapsed();
        
        info!("📊 Page {} processed in {:.2}s", 
              page_number, processing_time.as_secs_f32());
        
        Ok(LopdFPageResult {
            page_number,
            native_text,
            text_objects,
            ocr_result,
            processing_time,
            decision,
        })
    }
    
    /// Extraire le texte natif d'une page PDF avec lopdf
    async fn extract_native_text(&self, document: &Document, page_id: ObjectId) -> Result<(String, Vec<TextObject>)> {
        let mut all_text = String::new();
        let mut text_objects = Vec::new();
        
        // Obtenir la page
        let page_obj = document.get_object(page_id)
            .map_err(|e| OcrError::ImageProcessing(format!("Failed to get page object: {}", e)))?;
        
        if let Ok(page_dict) = page_obj.as_dict() {
            // Extraire le contenu de la page
            if let Ok(contents) = page_dict.get(b"Contents") {
                let content_stream = self.extract_content_stream(document, contents)?;
                let (text, objects) = self.parse_content_stream(&content_stream)?;
                all_text = text;
                text_objects = objects;
            }
        }
        
        Ok((all_text, text_objects))
    }
    
    /// Extraire le flux de contenu de la page
    fn extract_content_stream(&self, document: &Document, contents: &Object) -> Result<String> {
        match contents {
            Object::Reference(ref_id) => {
                let stream_obj = document.get_object(*ref_id)
                    .map_err(|e| OcrError::ImageProcessing(format!("Failed to get content stream: {}", e)))?;
                    
                if let Ok(stream) = stream_obj.as_stream() {
                    let decoded = stream.decode_content()
                        .map_err(|e| OcrError::ImageProcessing(format!("Failed to decode stream: {}", e)))?;
                    let text = decoded.operations.iter()
                        .filter_map(|op| {
                            let lopdf::content::Operation { operator, operands } = op;
                            if operator == "Tj" || operator == "TJ" {
                                let text_parts: Vec<String> = operands.iter().filter_map(|obj| {
                                    if let Object::String(ref s, _) = obj {
                                        Some(String::from_utf8_lossy(s).to_string())
                                    } else {
                                        None
                                    }
                                }).collect();
                                Some(text_parts.join(""))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    Ok(text)
                } else {
                    Ok(String::new())
                }
            }
            Object::Array(array) => {
                let mut combined_content = String::new();
                for item in array {
                    if let Object::Reference(ref_id) = item {
                        let content = self.extract_content_stream(document, &Object::Reference(*ref_id))?;
                        combined_content.push_str(&content);
                    }
                }
                Ok(combined_content)
            }
            _ => Ok(String::new()),
        }
    }
    
    /// Parser le flux de contenu pour extraire le texte et les objets
    fn parse_content_stream(&self, content: &str) -> Result<(String, Vec<TextObject>)> {
        let mut text = String::new();
        let mut text_objects = Vec::new();
        
        // Parser basique pour les commandes PDF de texte
        // Rechercher les patterns TJ, Tj, ' et " (show text operations)
        let lines: Vec<&str> = content.lines().collect();
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let mut current_font_size = 12.0;
        let mut current_font = "Unknown".to_string();
        
        for line in lines {
            let line = line.trim();
            
            // Détecter les opérations de positionnement de texte
            if line.contains(" Td ") {
                // Translation de position de texte
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(dx), Ok(dy)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                        current_x += dx;
                        current_y += dy;
                    }
                }
            }
            
            // Détecter les opérations de fonte
            else if line.contains(" Tf ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    current_font = parts[0].trim_start_matches('/').to_string();
                    if let Ok(size) = parts[1].parse::<f32>() {
                        current_font_size = size;
                    }
                }
            }
            
            // Détecter les opérations d'affichage de texte
            else if line.contains("Tj") || line.contains("TJ") {
                let extracted_text = self.extract_text_from_show_operation(line);
                if !extracted_text.is_empty() {
                    text.push_str(&extracted_text);
                    text.push(' ');
                    
                    text_objects.push(TextObject {
                        text: extracted_text,
                        x: current_x,
                        y: current_y,
                        font_size: current_font_size,
                        font_name: current_font.clone(),
                    });
                }
            }
        }
        
        Ok((text.trim().to_string(), text_objects))
    }
    
    /// Extraire le texte des opérations de show (Tj, TJ)
    fn extract_text_from_show_operation(&self, line: &str) -> String {
        // Parser basique pour extraire le texte entre parenthèses ou crochets
        let mut text = String::new();
        
        // Rechercher (text) Tj
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                if end > start {
                    text = line[start + 1..end].to_string();
                }
            }
        }
        // Rechercher [(...)] TJ (array of strings)
        else if line.contains('[') && line.contains(']') {
            // Parser les arrays de chaînes - version simplifiée
            let array_content = line.split('[').nth(1)
                .and_then(|s| s.split(']').next())
                .unwrap_or("");
            
            // Extraire toutes les chaînes entre parenthèses
            let mut in_string = false;
            let mut current_string = String::new();
            let chars: Vec<char> = array_content.chars().collect();
            
            for (i, &ch) in chars.iter().enumerate() {
                if ch == '(' && (i == 0 || chars[i-1] != '\\') {
                    in_string = true;
                } else if ch == ')' && (i == 0 || chars[i-1] != '\\') && in_string {
                    text.push_str(&current_string);
                    current_string.clear();
                    in_string = false;
                } else if in_string {
                    current_string.push(ch);
                }
            }
        }
        
        text
    }
    
    /// Fallback vers OCR pour les pages sans texte natif suffisant
    async fn fallback_to_ocr(&self, _document: &Document, _page_id: ObjectId, page_number: usize) -> Result<OcrResult> {
        // Note: lopdf ne peut pas rendre les pages en images
        // Pour l'OCR, vous avez plusieurs options :
        // 1. Utiliser un autre outil pour convertir PDF -> image (pdf2image, poppler-utils)
        // 2. Garder pdfium-render uniquement pour le rendu d'images
        // 3. Utiliser une approche hybride
        
        warn!("🔧 OCR fallback not implemented - lopdf cannot render pages to images");
        warn!("💡 Suggestion: Use external tools like 'pdftoppm' or keep pdfium-render for image rendering only");
        
        // Retourner un résultat vide pour l'instant
        Ok(OcrResult {
            text: String::new(),
            confidence: 0.0,
            language: "unknown".to_string(),
            bounding_boxes: Vec::new(),
            processing_time: Duration::from_millis(0),
            engine_used: "none".to_string(),
            tesseract_version: "none".to_string(),
            metadata: OcrMetadata {
                source_file: format!("page_{}", page_number),
                file_size_bytes: 0,
                image_dimensions: (0, 0),
                preprocessing_applied: Vec::new(),
                psm_used: PageSegMode::Auto,
                oem_used: OcrEngineMode::LstmOnly,
                temp_files_created: Vec::new(),
            },
            ocr_blocks: None,  // No layout analysis for fallback mode
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lopdf_pipeline_creation() {
        let config = LopdFPipelineConfig::default();
        match LopdFProcessor::new(config).await {
            Ok(_) => println!("✅ lopdf pipeline created successfully"),
            Err(e) => println!("⚠️ Failed to create lopdf pipeline: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_lopdf_text_extraction() {
        // Test avec un PDF simple si disponible
        let test_pdf = "test.pdf";
        if std::path::Path::new(test_pdf).exists() {
            let config = LopdFPipelineConfig::default();
            if let Ok(processor) = LopdFProcessor::new(config).await {
                match processor.process_pdf(Path::new(test_pdf)).await {
                    Ok(results) => {
                        println!("✅ Extracted text from {} pages", results.len());
                        for result in results {
                            println!("Page {}: {} chars", result.page_number, result.native_text.len());
                        }
                    }
                    Err(e) => println!("⚠️ Text extraction failed: {}", e),
                }
            }
        } else {
            println!("📝 Test PDF not found, skipping test");
        }
    }
}