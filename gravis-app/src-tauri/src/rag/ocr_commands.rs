// GRAVIS OCR - Commandes Tauri pour interface frontend
// Phase 1: Commandes simplifiées Tesseract

use super::ocr::{
    TesseractProcessor, OcrConfig, OcrResult, OcrPageResult, 
    PageSegMode, OcrEngineMode, OcrError
};
use std::path::PathBuf;
use tauri::State;
use std::sync::{Arc, Mutex};
use tracing::{info, error};

/// État global du processeur OCR
pub struct OcrState {
    pub processor: Arc<Mutex<Option<TesseractProcessor>>>,
}

impl OcrState {
    pub fn new() -> Self {
        Self {
            processor: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn initialize(&self, config: OcrConfig) -> Result<(), String> {
        match TesseractProcessor::new(config) {
            Ok(processor) => {
                *self.processor.lock().unwrap() = Some(processor);
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize OCR: {}", e)),
        }
    }
}

/// Initialiser le processeur OCR
#[tauri::command]
pub async fn ocr_initialize(
    languages: Vec<String>,
    state: State<'_, OcrState>,
) -> Result<String, String> {
    info!("🔄 Initializing OCR with languages: {:?}", languages);
    
    let config = OcrConfig {
        languages: languages.clone(),
        psm: PageSegMode::AutoOsd,
        oem: OcrEngineMode::LstmOnly,
        ..Default::default()
    };
    
    state.initialize(config)?;
    
    Ok(format!("OCR initialized with languages: {:?}", languages))
}

/// Traiter une image unique
#[tauri::command]
pub async fn ocr_process_image(
    file_path: String,
    languages: Vec<String>,
    state: State<'_, OcrState>,
) -> Result<OcrResult, String> {
    info!("🔄 Processing image: {}", file_path);
    
    // Vérifier si le processeur est initialisé
    let processor_guard = state.processor.lock().unwrap();
    let processor = processor_guard.as_ref()
        .ok_or("OCR not initialized. Call ocr_initialize first.")?;
    
    // Traiter l'image
    let path = PathBuf::from(file_path);
    processor.process_image(&path).await
        .map_err(|e| format!("OCR processing failed: {}", e))
}

/// Traiter un PDF (placeholder Phase 1)
#[tauri::command]
pub async fn ocr_process_pdf(
    file_path: String,
    languages: Vec<String>,
    state: State<'_, OcrState>,
) -> Result<Vec<OcrPageResult>, String> {
    info!("🔄 Processing PDF: {}", file_path);
    
    // Phase 1: Non implémenté
    Err("PDF processing not yet implemented in Phase 1".to_string())
}

/// Obtenir les langues Tesseract disponibles
#[tauri::command]
pub async fn ocr_get_supported_languages() -> Result<Vec<String>, String> {
    info!("📋 Getting supported languages");
    
    TesseractProcessor::get_available_languages()
        .map_err(|e| format!("Failed to get languages: {}", e))
}

/// Valider un fichier pour OCR
#[tauri::command]
pub async fn ocr_validate_file(file_path: String) -> Result<FileValidation, String> {
    info!("🔍 Validating file: {}", file_path);
    
    let path = PathBuf::from(&file_path);
    
    // Vérifier que le fichier existe
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Vérifier le format
    let format = super::ocr::detect_file_format(&path)
        .map_err(|e| format!("Unsupported format: {}", e))?;
    
    // Obtenir les métadonnées
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    Ok(FileValidation {
        is_valid: true,
        file_size: metadata.len(),
        format: format!("{:?}", format),
        estimated_processing_time: estimate_processing_time(metadata.len()),
        recommendations: generate_recommendations(&format, metadata.len()),
    })
}

/// Test de fonctionnement OCR
#[tauri::command]
pub async fn ocr_test_installation() -> Result<TestResult, String> {
    info!("🧪 Testing OCR installation");
    
    let mut results = TestResult {
        tesseract_available: false,
        languages_count: 0,
        available_languages: vec![],
        version: "unknown".to_string(),
        recommendations: vec![],
    };
    
    // Test Tesseract installation
    match std::process::Command::new("tesseract").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                results.tesseract_available = true;
                results.version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
            }
        }
        Err(_) => {
            results.recommendations.push("Install Tesseract: brew install tesseract tesseract-lang".to_string());
        }
    }
    
    // Test langues disponibles
    if results.tesseract_available {
        match TesseractProcessor::get_available_languages() {
            Ok(langs) => {
                results.languages_count = langs.len();
                results.available_languages = langs.into_iter().take(10).collect(); // Top 10
            }
            Err(e) => {
                results.recommendations.push(format!("Language detection failed: {}", e));
            }
        }
    }
    
    // Recommandations
    if results.languages_count < 10 {
        results.recommendations.push("Consider installing more language packs".to_string());
    }
    
    if results.tesseract_available && results.languages_count > 0 {
        results.recommendations.push("✅ OCR installation looks good!".to_string());
    }
    
    Ok(results)
}

/// Obtenir les statistiques du cache OCR
#[tauri::command]
pub async fn ocr_get_cache_stats(
    state: State<'_, OcrState>,
) -> Result<CacheStatsResponse, String> {
    let processor_guard = state.processor.lock().unwrap();
    let _processor = processor_guard.as_ref()
        .ok_or("OCR not initialized")?;
    
    // TODO: Implémenter récupération stats réelles du cache
    Ok(CacheStatsResponse {
        enabled: true,
        hit_rate: 0.75,
        entries_count: 42,
        memory_usage_mb: 15.5,
    })
}

/// Structures de réponse pour les commandes

#[derive(serde::Serialize)]
pub struct FileValidation {
    pub is_valid: bool,
    pub file_size: u64,
    pub format: String,
    pub estimated_processing_time: f32, // secondes
    pub recommendations: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct TestResult {
    pub tesseract_available: bool,
    pub languages_count: usize,
    pub available_languages: Vec<String>,
    pub version: String,
    pub recommendations: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct CacheStatsResponse {
    pub enabled: bool,
    pub hit_rate: f32,
    pub entries_count: usize,
    pub memory_usage_mb: f32,
}

/// Utilitaires

fn estimate_processing_time(file_size: u64) -> f32 {
    // Estimation basique : 1MB = ~3 secondes
    let mb = file_size as f32 / 1_048_576.0;
    (mb * 3.0).max(0.5) // Minimum 0.5s
}

fn generate_recommendations(format: &super::ocr::FileFormat, file_size: u64) -> Vec<String> {
    let mut recs = vec![];
    
    match format {
        super::ocr::FileFormat::Pdf => {
            recs.push("PDF processing will be available in Phase 2".to_string());
        }
        super::ocr::FileFormat::Jpeg => {
            recs.push("JPEG images may have compression artifacts. PNG recommended for OCR.".to_string());
        }
        _ => {}
    }
    
    if file_size > 10_000_000 { // 10MB
        recs.push("Large file detected. Processing may take longer.".to_string());
    }
    
    if file_size < 50_000 { // 50KB
        recs.push("Small image detected. Consider higher resolution for better OCR results.".to_string());
    }
    
    recs
}