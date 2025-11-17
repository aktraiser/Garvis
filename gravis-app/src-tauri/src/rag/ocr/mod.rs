// GRAVIS OCR Module - Phase 2: Command-based Implementation
// Architecture stable sans leptess, utilisant Command::new("tesseract")

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

pub mod tesseract;
pub mod preprocessor;
pub mod cache;
pub mod commands;
pub mod text_normalizer;
pub mod types;
pub mod layout_analyzer;

// === Alternatives PDF (pures Rust et sans dépendances externes) ===
pub mod pdf_lopdf;          // Alternative #1: lopdf (Pure Rust, recommandé)
pub mod pdf_extract_simple; // Alternative #2: pdf-extract (Simple)

// Phase 2 exports
pub use tesseract::{TesseractProcessor, TesseractConfig};
pub use cache::{OcrCache, CacheConfig};
pub use commands::{OcrCommands, OcrState};
pub use text_normalizer::{normalize_for_rag, normalize_and_log, normalize_fast, needs_normalization, NormalizationStats};
pub use layout_analyzer::{LayoutAnalyzer, LayoutAnalyzerConfig};
pub use types::{BoundingBox, OCRBlock, BlockType, BoundingBoxExt};

// === Exports des alternatives PDF ===
pub use pdf_lopdf::{LopdFProcessor, LopdFPipelineConfig, LopdFPageResult};
pub use pdf_extract_simple::{SimplePdfExtractor, PdfExtractConfig, SimpleExtractionResult, quick_extract_text};

/// Configuration OCR simplifiée pour Command-based approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    pub languages: Vec<String>,           // ["eng", "fra", "deu"]
    pub psm: PageSegMode,                 // Page Segmentation Mode
    pub oem: OcrEngineMode,              // OCR Engine Mode  
    pub preprocessing: PreprocessConfig,
    pub cache_config: CacheConfig,
    pub performance: PerformanceConfig,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            languages: vec!["eng".to_string(), "fra".to_string()],
            psm: PageSegMode::AutoOsd,
            oem: OcrEngineMode::LstmOnly,
            preprocessing: PreprocessConfig::default(),
            cache_config: CacheConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Page Segmentation Mode Tesseract
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PageSegMode {
    AutoOsd = 1,        // Auto détection orientation/script
    Auto = 3,           // Auto sans OSD
    SingleColumn = 4,   // Colonne unique
    SingleBlock = 6,    // Bloc de texte unique
    SingleLine = 7,     // Ligne unique
    SingleWord = 8,     // Mot unique
    SingleChar = 10,    // Caractère unique
}

impl PageSegMode {
    pub fn as_string(&self) -> String {
        (*self as u32).to_string()
    }
}

/// OCR Engine Mode Tesseract
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OcrEngineMode {
    LegacyOnly = 0,     // Legacy engine seulement
    NeuralOnly = 1,     // Neural network seulement  
    LstmOnly = 2,       // LSTM seulement (recommandé)
    Default = 3,        // Legacy + LSTM
}

impl OcrEngineMode {
    pub fn as_string(&self) -> String {
        (*self as u32).to_string()
    }
}

/// Configuration preprocessing via image crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessConfig {
    pub enabled: bool,                  // Activer preprocessing
    pub enhance_contrast: bool,         // Amélioration contraste
    pub resize_for_ocr: bool,          // Redimensionner pour OCR
    pub min_width: u32,                // Largeur minimale
    pub min_height: u32,               // Hauteur minimale
    pub target_dpi: u32,               // DPI cible pour OCR
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enhance_contrast: true,
            resize_for_ocr: true,
            min_width: 1200,
            min_height: 800,
            target_dpi: 300,
        }
    }
}

/// Configuration performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_jobs: usize,     // Workers Tesseract parallèles
    pub timeout_per_page: Duration,
    pub use_spawn_blocking: bool,       // tokio::spawn_blocking pour CPU-bound
    pub temp_dir: Option<String>,       // Répertoire temporaire
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 4,
            timeout_per_page: Duration::from_secs(30),
            use_spawn_blocking: true,
            temp_dir: None,
        }
    }
}

/// Résultat OCR avec métadonnées complètes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
    pub bounding_boxes: Vec<TesseractBoundingBox>,
    pub processing_time: Duration,
    pub engine_used: String,             // Toujours "Tesseract"
    pub tesseract_version: String,
    pub metadata: OcrMetadata,
    pub ocr_blocks: Option<Vec<OCRBlock>>,  // Layout analysis blocks
}

/// Résultat OCR par page (pour PDF)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrPageResult {
    pub page_number: usize,
    pub result: OcrResult,
    pub page_image_path: Option<String>,
}

/// Tesseract-specific bounding box pour localisation du texte au niveau word/line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TesseractBoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: String,
    pub confidence: f32,
    pub level: u32,  // Level de l'élément (word, line, paragraph)
}

/// Métadonnées OCR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrMetadata {
    pub source_file: String,
    pub file_size_bytes: u64,
    pub image_dimensions: (u32, u32),
    pub preprocessing_applied: Vec<String>,
    pub psm_used: PageSegMode,
    pub oem_used: OcrEngineMode,
    pub temp_files_created: Vec<String>,
}

/// Types d'erreurs OCR
#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("Tesseract command failed: {0}")]
    TesseractCommand(String),
    
    #[error("Image processing failed: {0}")]
    ImageProcessing(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Language not available: {0}")]
    LanguageNotAvailable(String),
    
    #[error("Timeout during processing")]
    Timeout,
    
    #[error("Parsing error: {0}")]
    Parsing(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OcrError>;

/// Validation des langues Tesseract disponibles
pub async fn validate_languages(languages: &[String]) -> Result<()> {
    let available = get_available_languages().await?;
    
    for lang in languages {
        if !available.contains(lang) {
            return Err(OcrError::LanguageNotAvailable(lang.clone()));
        }
    }
    
    Ok(())
}

/// Obtenir les langues Tesseract disponibles
pub async fn get_available_languages() -> Result<Vec<String>> {
    use std::process::Command;
    
    let output = tokio::task::spawn_blocking(|| {
        Command::new("tesseract")
            .arg("--list-langs")
            .output()
    }).await.map_err(|e| OcrError::TesseractCommand(format!("Failed to spawn task: {}", e)))?
    .map_err(|e| OcrError::TesseractCommand(format!("Failed to execute tesseract: {}", e)))?;
    
    if !output.status.success() {
        return Err(OcrError::TesseractCommand(
            "Failed to get language list".to_string()
        ));
    }
    
    let langs_output = String::from_utf8_lossy(&output.stdout);
    let languages: Vec<String> = langs_output
        .lines()
        .skip(1) // Skip header
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with("script/"))
        .map(|line| line.trim().to_string())
        .collect();
    
    Ok(languages)
}

/// Détection automatique du format de fichier
pub fn detect_file_format(path: &Path) -> Result<FileFormat> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => Ok(FileFormat::Pdf),
        Some("png") => Ok(FileFormat::Png),
        Some("jpg") | Some("jpeg") => Ok(FileFormat::Jpeg),
        Some("tiff") | Some("tif") => Ok(FileFormat::Tiff),
        Some("bmp") => Ok(FileFormat::Bmp),
        _ => Err(OcrError::UnsupportedFormat(
            path.to_string_lossy().to_string()
        )),
    }
}

/// Formats de fichier supportés
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileFormat {
    Pdf,
    Png,
    Jpeg,
    Tiff,
    Bmp,
}

/// Obtenir la version de Tesseract
pub async fn get_tesseract_version() -> Result<String> {
    use std::process::Command;
    
    let output = tokio::task::spawn_blocking(|| {
        Command::new("tesseract")
            .arg("--version")
            .output()
    }).await.map_err(|e| OcrError::TesseractCommand(format!("Failed to spawn task: {}", e)))?
    .map_err(|e| OcrError::TesseractCommand(format!("Failed to execute tesseract: {}", e)))?;
    
    if !output.status.success() {
        return Err(OcrError::TesseractCommand(
            "Failed to get version".to_string()
        ));
    }
    
    let version_output = String::from_utf8_lossy(&output.stdout);
    let version_line = version_output.lines().next()
        .unwrap_or("unknown")
        .to_string();
    
    Ok(version_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_available_languages() {
        match get_available_languages().await {
            Ok(langs) => {
                println!("✅ Available languages ({}): {:?}", 
                         langs.len(), langs.iter().take(5).collect::<Vec<_>>());
                assert!(!langs.is_empty());
                assert!(langs.contains(&"eng".to_string()));
            }
            Err(e) => println!("⚠️ Language test failed: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_validate_languages() {
        let languages = vec!["eng".to_string(), "fra".to_string()];
        match validate_languages(&languages).await {
            Ok(_) => println!("✅ Language validation passed"),
            Err(e) => println!("⚠️ Language validation failed: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_get_tesseract_version() {
        match get_tesseract_version().await {
            Ok(version) => {
                println!("✅ Tesseract version: {}", version);
                assert!(version.to_lowercase().contains("tesseract"));
            }
            Err(e) => println!("⚠️ Version test failed: {}", e),
        }
    }
}