// GRAVIS OCR - Pipeline PDF Simple avec pdf-extract
// Alternative la plus simple pour extraction de texte uniquement

use super::{OcrError, Result, normalize_and_log};
use pdf_extract::{extract_text, extract_text_from_mem};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{info, warn};

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
        
        Ok(SimpleExtractionResult {
            text,
            token_count,
            char_count,
            processing_time,
            success,
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
        })
    }
    
    /// Vérifier si un PDF contient du texte extractible
    pub async fn has_extractable_text(&self, pdf_path: &Path) -> Result<bool> {
        match self.extract_pdf_text(pdf_path).await {
            Ok(result) => Ok(result.success),
            Err(_) => Ok(false),
        }
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