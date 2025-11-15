// GRAVIS OCR - Pipeline PDF Simple avec pdf-extract
// Alternative la plus simple pour extraction de texte uniquement

use super::{OcrError, Result, normalize_and_log, OCRBlock, BlockType, BoundingBox as SemanticBoundingBox};
use pdf_extract::{extract_text, extract_text_from_mem};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};

/// Configuration simple pour pdf-extract
#[derive(Debug, Clone)]
pub struct PdfExtractConfig {
    /// Seuil minimum de tokens pour considérer l'extraction réussie
    pub min_tokens: usize,
    /// Timeout pour l'extraction
    pub timeout: Duration,
    /// Activer la normalisation Unicode (ligatures, espaces)
    pub normalize_unicode: bool,
}

impl Default for PdfExtractConfig {
    fn default() -> Self {
        Self {
            min_tokens: 10,
            timeout: Duration::from_secs(30),
            normalize_unicode: true,  // Activé par défaut pour RAG
        }
    }
}

/// Résultat d'extraction simple
#[derive(Debug, Clone)]
pub struct SimpleExtractionResult {
    pub text: String,
    pub token_count: usize,
    pub char_count: usize,
    pub processing_time: Duration,
    pub success: bool,
    pub image_blocks: Vec<OCRBlock>,  // Blocks détectés comme images/figures
    pub layout_blocks: Vec<OCRBlock>,  // 🆕 Blocks avec layout analysis (texte + positions)
    pub page_dimensions: std::collections::HashMap<u32, (f64, f64)>,  // 🆕 Dimensions réelles de chaque page
}

/// Processeur simple avec pdf-extract
pub struct SimplePdfExtractor {
    config: PdfExtractConfig,
}

impl SimplePdfExtractor {
    /// Créer un nouveau extracteur simple
    pub fn new(config: PdfExtractConfig) -> Self {
        info!("🚀 Initializing Simple PDF Extractor (pdf-extract)");
        Self { config }
    }
    
    /// Extraire le texte d'un PDF complet (méthode simple)
    pub async fn extract_pdf_text(&self, pdf_path: &Path) -> Result<SimpleExtractionResult> {
        let start_time = Instant::now();
        info!("📄 Extracting text from PDF: {:?}", pdf_path);
        
        // Vérifier que le fichier existe
        if !pdf_path.exists() {
            return Err(OcrError::FileNotFound(pdf_path.to_string_lossy().to_string()));
        }
        
        // Extraction avec timeout
        let result = tokio::time::timeout(
            self.config.timeout,
            tokio::task::spawn_blocking({
                let path = pdf_path.to_path_buf();
                move || extract_text(&path)
            })
        ).await;
        
        let raw_text = match result {
            Ok(Ok(Ok(text))) => text,
            Ok(Ok(Err(e))) => {
                return Err(OcrError::ImageProcessing(format!("pdf-extract failed: {:?}", e)));
            }
            Ok(Err(e)) => {
                return Err(OcrError::TesseractCommand(format!("Task failed: {}", e)));
            }
            Err(_) => {
                return Err(OcrError::Timeout);
            }
        };
        
        // Normalisation Unicode pour RAG
        let text = if self.config.normalize_unicode {
            normalize_and_log(&raw_text, "pdf-extract")
        } else {
            raw_text
        };
        
        let processing_time = start_time.elapsed();
        let token_count = text.split_whitespace().count();
        let char_count = text.len();
        let success = token_count >= self.config.min_tokens;
        
        info!("✅ PDF text extraction completed in {:.2}s: {} tokens, {} chars", 
              processing_time.as_secs_f32(), token_count, char_count);
        
        if !success {
            warn!("⚠️ Low token count: {} (minimum: {})", token_count, self.config.min_tokens);
        }

        // Extract images/figures from PDF
        let image_blocks = self.extract_pdf_images(pdf_path).await.unwrap_or_default();
        if !image_blocks.is_empty() {
            debug!("📊 Detected {} image/figure blocks in PDF", image_blocks.len());
        }

        // 🆕 Extract layout blocks (text + positions) from PDF
        let (layout_blocks, page_dimensions) = self.extract_layout_blocks_from_text(pdf_path, &text).await
            .unwrap_or_else(|_| (Vec::new(), std::collections::HashMap::new()));
        if !layout_blocks.is_empty() {
            info!("📐 Extracted {} layout blocks from PDF", layout_blocks.len());
        }

        Ok(SimpleExtractionResult {
            text,
            token_count,
            char_count,
            processing_time,
            success,
            image_blocks,
            layout_blocks,
            page_dimensions,
        })
    }
    
    /// Extraire le texte d'un PDF depuis la mémoire
    pub async fn extract_pdf_text_from_memory(&self, pdf_data: &[u8]) -> Result<SimpleExtractionResult> {
        let start_time = Instant::now();
        info!("📄 Extracting text from PDF in memory ({} bytes)", pdf_data.len());
        
        // Extraction avec timeout
        let result = tokio::time::timeout(
            self.config.timeout,
            tokio::task::spawn_blocking({
                let data = pdf_data.to_vec();
                move || extract_text_from_mem(&data)
            })
        ).await;
        
        let raw_text = match result {
            Ok(Ok(Ok(text))) => text,
            Ok(Ok(Err(e))) => {
                return Err(OcrError::ImageProcessing(format!("pdf-extract failed: {:?}", e)));
            }
            Ok(Err(e)) => {
                return Err(OcrError::TesseractCommand(format!("Task failed: {}", e)));
            }
            Err(_) => {
                return Err(OcrError::Timeout);
            }
        };
        
        // Normalisation Unicode pour RAG
        let text = if self.config.normalize_unicode {
            normalize_and_log(&raw_text, "pdf-extract")
        } else {
            raw_text
        };
        
        let processing_time = start_time.elapsed();
        let token_count = text.split_whitespace().count();
        let char_count = text.len();
        let success = token_count >= self.config.min_tokens;
        
        info!("✅ PDF memory extraction completed in {:.2}s: {} tokens, {} chars", 
              processing_time.as_secs_f32(), token_count, char_count);
        
        Ok(SimpleExtractionResult {
            text,
            token_count,
            char_count,
            processing_time,
            success,
            image_blocks: Vec::new(), // Memory extraction doesn't support image detection yet
            layout_blocks: Vec::new(), // Memory extraction doesn't support layout blocks yet
            page_dimensions: std::collections::HashMap::new(), // Memory extraction doesn't support page dimensions yet
        })
    }
    
    /// Vérifier si un PDF contient du texte extractible
    pub async fn has_extractable_text(&self, pdf_path: &Path) -> Result<bool> {
        match self.extract_pdf_text(pdf_path).await {
            Ok(result) => Ok(result.success),
            Err(_) => Ok(false),
        }
    }

    /// Extraire les blocs de texte avec positionnement par page
    /// Cette fonction génère les blocs OCR natifs pour l'overlay interactif
    /// Utilise le texte global extrait et le répartit sur les pages
    pub async fn extract_layout_blocks_from_text(&self, pdf_path: &Path, full_text: &str) -> Result<(Vec<OCRBlock>, std::collections::HashMap<u32, (f64, f64)>)> {
        use lopdf::Document;
        use std::collections::HashMap;

        debug!("🔍 Extracting layout blocks from PDF text: {:?}", pdf_path);

        // Charger le PDF avec lopdf pour obtenir le nombre de pages et dimensions
        let doc = tokio::task::spawn_blocking({
            let path = pdf_path.to_path_buf();
            move || Document::load(&path)
        }).await
            .map_err(|e| OcrError::ImageProcessing(format!("Task failed: {}", e)))?
            .map_err(|e| OcrError::ImageProcessing(format!("Failed to load PDF: {:?}", e)))?;

        let pages = doc.get_pages();
        let page_count = pages.len() as u32;

        if page_count == 0 {
            warn!("PDF has no pages");
            return Ok((Vec::new(), HashMap::new()));
        }

        // Découper le texte en paragraphes RÉELS (pas juste \n\n)
        // Fusion des lignes courtes pour éviter la fragmentation
        let raw_paragraphs: Vec<&str> = full_text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        // Regrouper les paragraphes trop courts (< 100 chars) avec le suivant
        let mut paragraphs = Vec::new();
        let mut current_group = String::new();

        for para in &raw_paragraphs {
            let trimmed = para.trim();

            // Si le paragraphe est court (<100 chars) ou se termine par un mot incomplet,
            // on le fusionne avec le suivant
            if trimmed.len() < 100 || (!trimmed.ends_with('.') && !trimmed.ends_with('!') && !trimmed.ends_with('?')) {
                if !current_group.is_empty() {
                    current_group.push(' ');
                }
                current_group.push_str(trimmed);
            } else {
                // Paragraphe complet, on ajoute le groupe actuel s'il existe
                if !current_group.is_empty() {
                    current_group.push(' ');
                    current_group.push_str(trimmed);
                    paragraphs.push(current_group.clone());
                    current_group.clear();
                } else {
                    paragraphs.push(trimmed.to_string());
                }
            }
        }

        // Ne pas oublier le dernier groupe
        if !current_group.is_empty() {
            paragraphs.push(current_group);
        }

        if paragraphs.is_empty() {
            warn!("No paragraphs found in text");
            return Ok((Vec::new(), HashMap::new()));
        }

        debug!("📝 Regrouped {} raw paragraphs into {} semantic blocks",
               raw_paragraphs.len(), paragraphs.len());

        // Répartir les paragraphes sur les pages (approximatif)
        let paragraphs_per_page = (paragraphs.len() as f64 / page_count as f64).ceil() as usize;
        let paragraphs_per_page = paragraphs_per_page.max(1);

        let mut all_blocks = Vec::new();
        let mut page_dimensions: HashMap<u32, (f64, f64)> = HashMap::new();

        for (page_idx, (page_num, page_id)) in pages.iter().enumerate() {
            // Extraire dimensions de la page
            let (page_width, page_height) = match self.get_page_dimensions(&doc, *page_id) {
                Ok(dims) => {
                    info!("📏 Page {}: Real dimensions {}x{}", page_num, dims.0, dims.1);
                    dims
                },
                Err(e) => {
                    warn!("⚠️ Could not extract dimensions for page {}: {:?}, using A4 default", page_num, e);
                    (595.0, 842.0)
                }
            };

            // Stocker les dimensions de cette page
            page_dimensions.insert(*page_num, (page_width, page_height));

            // Calculer quels paragraphes vont sur cette page
            let start_para = page_idx * paragraphs_per_page;
            let end_para = ((page_idx + 1) * paragraphs_per_page).min(paragraphs.len());

            if start_para >= paragraphs.len() {
                break;
            }

            let page_paragraphs = &paragraphs[start_para..end_para];

            let mut current_y = 50.0; // Marge top
            let margin_x = 50.0;

            for paragraph in page_paragraphs {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Détecter le type de bloc
                let block_type = if trimmed.lines().count() == 1 && trimmed.len() < 100 {
                    BlockType::Header
                } else if trimmed.lines().any(|l| l.trim_start().starts_with("•") ||
                                                      l.trim_start().starts_with("-") ||
                                                      l.trim_start().chars().next()
                                                          .map(|c| c.is_ascii_digit())
                                                          .unwrap_or(false)) {
                    BlockType::List
                } else {
                    BlockType::Text
                };

                // Calculer hauteur approximative (16pt line height * nb lignes)
                let line_count = trimmed.lines().count();
                let block_height = (line_count as f64 * 16.0).min(page_height - current_y - 50.0);

                let bbox = SemanticBoundingBox {
                    x: margin_x,
                    y: current_y,
                    width: page_width - (margin_x * 2.0),
                    height: block_height,
                };

                let block = OCRBlock {
                    page_number: *page_num,
                    block_type,
                    content: trimmed.to_string(),
                    bounding_box: bbox,
                    confidence: 0.75, // Confiance modérée pour blocs synthétiques répartis
                    spans: Vec::new(),
                };

                all_blocks.push(block);
                current_y += block_height + 10.0; // Espacement entre blocs

                // Si on dépasse la hauteur de la page, arrêter pour cette page
                if current_y > page_height - 100.0 {
                    break;
                }
            }
        }

        info!("✅ Extracted {} layout blocks distributed across {} pages", all_blocks.len(), page_count);
        info!("📐 Collected dimensions for {} pages", page_dimensions.len());
        Ok((all_blocks, page_dimensions))
    }

    /// Extraire les dimensions d'une page PDF
    fn get_page_dimensions(&self, doc: &lopdf::Document, page_id: lopdf::ObjectId) -> Result<(f64, f64)> {
        use lopdf::Object;

        let page_obj = doc.get_object(page_id)
            .map_err(|e| OcrError::Parsing(format!("Failed to get page object: {:?}", e)))?;

        let page_dict = page_obj.as_dict()
            .map_err(|e| OcrError::Parsing(format!("Page is not a dict: {:?}", e)))?;

        // Chercher MediaBox (définit les dimensions de la page)
        if let Ok(media_box) = page_dict.get(b"MediaBox") {
            if let Ok(array) = media_box.as_array() {
                if array.len() >= 4 {
                    // lopdf::Object peut être Integer ou Real (f32), convertir en f64
                    let x1 = match &array[0] {
                        Object::Integer(i) => *i as f64,
                        Object::Real(r) => *r as f64,
                        _ => 0.0,
                    };
                    let y1 = match &array[1] {
                        Object::Integer(i) => *i as f64,
                        Object::Real(r) => *r as f64,
                        _ => 0.0,
                    };
                    let x2 = match &array[2] {
                        Object::Integer(i) => *i as f64,
                        Object::Real(r) => *r as f64,
                        _ => 595.0,
                    };
                    let y2 = match &array[3] {
                        Object::Integer(i) => *i as f64,
                        Object::Real(r) => *r as f64,
                        _ => 842.0,
                    };

                    let width = x2 - x1;
                    let height = y2 - y1;

                    debug!("Page dimensions: {}x{} (MediaBox)", width, height);
                    return Ok((width, height));
                }
            }
        }

        // Fallback: A4 dimensions
        warn!("Could not extract MediaBox, using A4 default");
        Ok((595.0, 842.0))
    }

    /// Extraire le texte d'une page spécifique avec lopdf
    fn extract_page_text(&self, doc: &lopdf::Document, page_num: u32, page_id: lopdf::ObjectId) -> Result<String> {
        use lopdf::Object;

        debug!("Extracting text from page {}", page_num);

        let page_obj = doc.get_object(page_id)
            .map_err(|e| OcrError::Parsing(format!("Failed to get page object: {:?}", e)))?;

        let page_dict = page_obj.as_dict()
            .map_err(|e| OcrError::Parsing(format!("Page is not a dict: {:?}", e)))?;

        // Récupérer le contenu de la page
        let mut text_content = String::new();

        if let Ok(contents_obj) = page_dict.get(b"Contents") {
            // Contents peut être un array ou un seul stream
            let content_streams = if let Ok(array) = contents_obj.as_array() {
                array.clone()
            } else {
                vec![contents_obj.clone()]
            };

            for stream_ref in content_streams {
                if let Ok(object_id) = stream_ref.as_reference() {
                    if let Ok(stream_obj) = doc.get_object(object_id) {
                        if let Ok(stream) = stream_obj.as_stream() {
                            // Décoder le stream - decode_content() retourne Content avec .operations
                            if let Ok(decoded) = stream.decode_content() {
                                // Convertir Content en string pour parser les opérateurs
                                let content_bytes = format!("{:?}", decoded);  // Fallback simple
                                let extracted = self.extract_text_from_content_stream(&content_bytes);
                                if !extracted.is_empty() {
                                    text_content.push_str(&extracted);
                                    text_content.push('\n');
                                }
                            }
                        }
                    }
                }
            }
        }

        if text_content.trim().is_empty() {
            debug!("No text content found on page {}", page_num);
            return Err(OcrError::Parsing(format!("No text on page {}", page_num)));
        }

        Ok(text_content)
    }

    /// Extraire le texte des opérateurs de contenu PDF
    fn extract_text_from_content_stream(&self, content: &str) -> String {
        let mut text = String::new();
        let mut in_text_block = false;

        for line in content.lines() {
            let line = line.trim();

            // Détecter début de bloc texte
            if line == "BT" {
                in_text_block = true;
                continue;
            }

            // Détecter fin de bloc texte
            if line == "ET" {
                in_text_block = false;
                text.push('\n');
                continue;
            }

            if !in_text_block {
                continue;
            }

            // Extraire texte des opérateurs Tj et TJ
            if line.ends_with(" Tj") {
                // Format: (text) Tj ou <hex> Tj
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.rfind(')') {
                        let extracted = &line[start + 1..end];
                        text.push_str(extracted);
                        text.push(' ');
                    }
                }
            } else if line.ends_with(" TJ") {
                // Format: [(text1) offset (text2)] TJ
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.rfind(']') {
                        let array_content = &line[start + 1..end];
                        // Parser les strings dans l'array
                        for part in array_content.split('(') {
                            if let Some(text_end) = part.find(')') {
                                text.push_str(&part[..text_end]);
                                text.push(' ');
                            }
                        }
                    }
                }
            } else if line.ends_with(" '") || line.ends_with(" \"") {
                // Opérateurs ' et " : afficher string et nouvelle ligne
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.rfind(')') {
                        text.push_str(&line[start + 1..end]);
                        text.push('\n');
                    }
                }
            }
        }

        text
    }

    /// Extraire les images/figures d'un PDF
    async fn extract_pdf_images(&self, pdf_path: &Path) -> Result<Vec<OCRBlock>> {
        use lopdf::{Document, Object};

        debug!("🔍 Extracting images/figures from PDF: {:?}", pdf_path);

        // Charger le PDF avec lopdf
        let doc = tokio::task::spawn_blocking({
            let path = pdf_path.to_path_buf();
            move || Document::load(&path)
        }).await
            .map_err(|e| OcrError::ImageProcessing(format!("Task failed: {}", e)))?
            .map_err(|e| OcrError::ImageProcessing(format!("Failed to load PDF: {:?}", e)))?;

        let mut image_blocks = Vec::new();
        let mut image_counter = 0;

        // Parcourir toutes les pages
        let page_count = doc.get_pages().len();

        for (page_num, page_id) in doc.get_pages() {
            if let Ok(page_obj) = doc.get_object(page_id) {
                if let Ok(page_dict) = page_obj.as_dict() {
                    // Chercher les ressources de la page
                    if let Ok(resources) = page_dict.get(b"Resources") {
                        if let Ok(resources_dict) = resources.as_dict() {
                            // Chercher XObject (contient les images)
                            if let Ok(xobject) = resources_dict.get(b"XObject") {
                                if let Ok(xobject_dict) = xobject.as_dict() {
                                    // Parcourir tous les XObjects
                                    for (name, obj_ref) in xobject_dict.iter() {
                                        if let Ok(obj) = doc.get_object(obj_ref.as_reference()
                                            .map_err(|_| OcrError::Parsing("Invalid object reference".to_string()))?) {

                                            if let Ok(stream) = obj.as_stream() {
                                                let dict = &stream.dict;

                                                // Vérifier si c'est une image
                                                if let Ok(subtype) = dict.get(b"Subtype") {
                                                    if let Ok(subtype_name) = subtype.as_name_str() {
                                                        if subtype_name == "Image" {
                                                            image_counter += 1;

                                                            // Extraire les dimensions si disponibles
                                                            let width = dict.get(b"Width")
                                                                .ok()
                                                                .and_then(|w| w.as_i64().ok())
                                                                .unwrap_or(0) as f64;

                                                            let height = dict.get(b"Height")
                                                                .ok()
                                                                .and_then(|h| h.as_i64().ok())
                                                                .unwrap_or(0) as f64;

                                                            // Créer un OCRBlock pour cette image
                                                            let bbox = SemanticBoundingBox {
                                                                x: 0.0,
                                                                y: 0.0,
                                                                width: width.max(100.0),
                                                                height: height.max(100.0),
                                                            };

                                                            let name_str = String::from_utf8_lossy(name);

                                                            let block = OCRBlock {
                                                                page_number: page_num,
                                                                block_type: BlockType::Figure,
                                                                content: format!("[Figure/Image: {}]", name_str),
                                                                bounding_box: bbox,
                                                                confidence: 0.9,
                                                                spans: Vec::new(),  // No source spans for PDF images
                                                            };

                                                            image_blocks.push(block);

                                                            debug!("📊 Found image '{}' on page {} ({}x{})",
                                                                   name_str, page_num, width, height);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("📊 Extracted {} image/figure blocks from {} pages", image_blocks.len(), page_count);

        Ok(image_blocks)
    }
}

/// Fonction utilitaire pour extraction rapide
pub async fn quick_extract_text(pdf_path: &Path) -> Result<String> {
    let extractor = SimplePdfExtractor::new(PdfExtractConfig::default());
    let result = extractor.extract_pdf_text(pdf_path).await?;
    Ok(result.text)
}

/// Fonction utilitaire pour extraction avec seuil personnalisé
pub async fn extract_text_with_threshold(pdf_path: &Path, min_tokens: usize) -> Result<Option<String>> {
    let config = PdfExtractConfig {
        min_tokens,
        timeout: Duration::from_secs(30),
        normalize_unicode: true,
    };
    let extractor = SimplePdfExtractor::new(config);
    let result = extractor.extract_pdf_text(pdf_path).await?;
    
    if result.success {
        Ok(Some(result.text))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_simple_extractor_creation() {
        let config = PdfExtractConfig::default();
        let _extractor = SimplePdfExtractor::new(config);
        println!("✅ Simple PDF extractor created successfully");
    }
    
    #[tokio::test]
    async fn test_quick_extract() {
        let test_pdf = PathBuf::from("test.pdf");
        if test_pdf.exists() {
            match quick_extract_text(&test_pdf).await {
                Ok(text) => {
                    println!("✅ Quick extraction successful: {} chars", text.len());
                    if !text.is_empty() {
                        println!("Preview: {}...", &text[..text.len().min(100)]);
                    }
                }
                Err(e) => println!("⚠️ Quick extraction failed: {}", e),
            }
        } else {
            println!("📝 Test PDF not found, skipping test");
        }
    }
    
    #[tokio::test]
    async fn test_has_extractable_text() {
        let test_pdf = PathBuf::from("test.pdf");
        if test_pdf.exists() {
            let extractor = SimplePdfExtractor::new(PdfExtractConfig::default());
            match extractor.has_extractable_text(&test_pdf).await {
                Ok(has_text) => println!("✅ Has extractable text: {}", has_text),
                Err(e) => println!("⚠️ Text check failed: {}", e),
            }
        } else {
            println!("📝 Test PDF not found, skipping test");
        }
    }
}