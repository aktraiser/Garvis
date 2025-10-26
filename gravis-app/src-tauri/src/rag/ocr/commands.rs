// GRAVIS OCR - Tauri Commands Integration
// Phase 2: Commands pour l'interface frontend via Tauri

use super::{
    OcrConfig, OcrResult, TesseractProcessor, TesseractConfig,
    OcrCache,
    get_available_languages, get_tesseract_version, detect_file_format, FileFormat
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, error, debug};

/// État global OCR pour Tauri
pub struct OcrState {
    processor: Arc<Mutex<Option<TesseractProcessor>>>,
    cache: Arc<Mutex<Option<OcrCache>>>,
    config: Arc<Mutex<OcrConfig>>,
}

impl OcrState {
    pub fn new() -> Self {
        Self {
            processor: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(OcrConfig::default())),
        }
    }
}

/// Réponse pour les commandes Tauri
#[derive(Debug, Serialize, Deserialize)]
pub struct OcrCommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> OcrCommandResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Commands Tauri pour OCR
pub struct OcrCommands;

/// Initialiser le système OCR
#[tauri::command]
pub async fn ocr_initialize(
    config: Option<OcrConfig>,
    state: tauri::State<'_, OcrState>
) -> Result<String, String> {
    info!("🚀 Initializing OCR system");
    
    let final_config = config.unwrap_or_default();
    
    // Créer le processeur Tesseract
    let tesseract_config = TesseractConfig {
        languages: final_config.languages.clone(),
        psm: final_config.psm,
        oem: final_config.oem,
        preprocessing: final_config.preprocessing.clone(),
        confidence_threshold: 0.7,
        temp_dir: std::env::temp_dir().join("gravis_ocr"),
        max_concurrent: final_config.performance.max_concurrent_jobs,
        timeout: final_config.performance.timeout_per_page,
    };
    
    match TesseractProcessor::new(tesseract_config).await {
        Ok(processor) => {
            // Créer le cache
            match OcrCache::new(final_config.cache_config.clone()).await {
                Ok(cache) => {
                    // Stocker dans l'état
                    if let Ok(mut proc_guard) = state.processor.lock() {
                        *proc_guard = Some(processor);
                    }
                    if let Ok(mut cache_guard) = state.cache.lock() {
                        *cache_guard = Some(cache);
                    }
                    if let Ok(mut config_guard) = state.config.lock() {
                        *config_guard = final_config;
                    }
                    
                    info!("✅ OCR system initialized successfully");
                    Ok(serde_json::to_string(&OcrCommandResponse::ok("OCR system initialized".to_string())).unwrap_or_default())
                }
                Err(e) => {
                    error!("Failed to create cache: {}", e);
                    Ok(serde_json::to_string(&OcrCommandResponse::<String>::error(format!("Cache initialization failed: {}", e))).unwrap_or_default())
                }
            }
        }
        Err(e) => {
            error!("Failed to create processor: {}", e);
            Ok(serde_json::to_string(&OcrCommandResponse::<String>::error(format!("Processor initialization failed: {}", e))).unwrap_or_default())
        }
    }
}

/// Traiter une image unique
#[tauri::command]
pub async fn ocr_process_image(
    image_path: String,
    state: tauri::State<'_, OcrState>
) -> Result<String, String> {
    info!("🔄 Processing image: {}", image_path);
    
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error("File not found".to_string())).unwrap_or_default());
    }
    
    // Vérifier le format
    match detect_file_format(&path) {
        Ok(FileFormat::Pdf) => {
            return Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error("PDF processing not yet implemented".to_string())).unwrap_or_default());
        }
        Ok(_) => {} // Image formats OK
        Err(e) => {
            return Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error(format!("Unsupported format: {}", e))).unwrap_or_default());
        }
    }
    
    // Créer processeur temporaire (architecture à améliorer)
    let config = if let Ok(config_guard) = state.config.lock() {
        config_guard.clone()
    } else {
        return Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error("Failed to get config".to_string())).unwrap_or_default());
    };
    
    let tesseract_config = TesseractConfig {
        languages: config.languages,
        psm: config.psm,
        oem: config.oem,
        preprocessing: config.preprocessing,
        confidence_threshold: 0.7,
        temp_dir: std::env::temp_dir().join("gravis_ocr"),
        max_concurrent: config.performance.max_concurrent_jobs,
        timeout: config.performance.timeout_per_page,
    };
    
    let processor = match TesseractProcessor::new(tesseract_config).await {
        Ok(proc) => proc,
        Err(e) => return Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error(format!("Failed to create processor: {}", e))).unwrap_or_default()),
    };
    
    // Traiter l'image
    match processor.process_image(&path).await {
        Ok(result) => {
            info!("✅ Image processed successfully: {:.1}% confidence", result.confidence * 100.0);
            Ok(serde_json::to_string(&OcrCommandResponse::ok(result)).unwrap_or_default())
        }
        Err(e) => {
            error!("Image processing failed: {}", e);
            Ok(serde_json::to_string(&OcrCommandResponse::<OcrResult>::error(format!("Processing failed: {}", e))).unwrap_or_default())
        }
    }
}

/// Obtenir les langues disponibles
#[tauri::command]
pub async fn ocr_get_available_languages() -> String {
    debug!("📋 Getting available languages");
    
    match get_available_languages().await {
        Ok(languages) => {
            info!("✅ Found {} available languages", languages.len());
            serde_json::to_string(&OcrCommandResponse::ok(languages)).unwrap_or_default()
        }
        Err(e) => {
            error!("Failed to get languages: {}", e);
            serde_json::to_string(&OcrCommandResponse::<Vec<String>>::error(format!("Failed to get languages: {}", e))).unwrap_or_default()
        }
    }
}

/// Obtenir la version de Tesseract
#[tauri::command]
pub async fn ocr_get_version() -> String {
    debug!("📋 Getting Tesseract version");
    
    match get_tesseract_version().await {
        Ok(version) => {
            serde_json::to_string(&OcrCommandResponse::ok(version)).unwrap_or_default()
        }
        Err(e) => {
            error!("Failed to get version: {}", e);
            serde_json::to_string(&OcrCommandResponse::<String>::error(format!("Failed to get version: {}", e))).unwrap_or_default()
        }
    }
}

/// Obtenir les statistiques du cache
#[tauri::command]
pub async fn ocr_get_cache_stats(
    state: tauri::State<'_, OcrState>
) -> Result<String, String> {
    debug!("📊 Getting cache statistics");
    
    if let Ok(cache_guard) = state.cache.lock() {
        if let Some(ref cache) = *cache_guard {
            let stats = cache.get_cache_info();
            Ok(serde_json::to_string(&OcrCommandResponse::ok(stats)).unwrap_or_default())
        } else {
            Ok(serde_json::to_string(&OcrCommandResponse::<HashMap<String, serde_json::Value>>::error("Cache not initialized".to_string())).unwrap_or_default())
        }
    } else {
        Ok(serde_json::to_string(&OcrCommandResponse::<HashMap<String, serde_json::Value>>::error("Failed to access cache".to_string())).unwrap_or_default())
    }
}

/// Vider le cache
#[tauri::command]
pub async fn ocr_clear_cache(
    state: tauri::State<'_, OcrState>
) -> Result<String, String> {
    info!("🗑️ Clearing OCR cache");
    
    if let Ok(cache_guard) = state.cache.lock() {
        if let Some(ref cache) = *cache_guard {
            match cache.clear() {
                Ok(_) => Ok(serde_json::to_string(&OcrCommandResponse::ok("Cache cleared".to_string())).unwrap_or_default()),
                Err(e) => Ok(serde_json::to_string(&OcrCommandResponse::<String>::error(format!("Failed to clear cache: {}", e))).unwrap_or_default()),
            }
        } else {
            Ok(serde_json::to_string(&OcrCommandResponse::<String>::error("Cache not initialized".to_string())).unwrap_or_default())
        }
    } else {
        Ok(serde_json::to_string(&OcrCommandResponse::<String>::error("Failed to access cache".to_string())).unwrap_or_default())
    }
}

/// Obtenir la configuration actuelle
#[tauri::command]
pub async fn ocr_get_config(
    state: tauri::State<'_, OcrState>
) -> Result<String, String> {
    debug!("📋 Getting OCR configuration");
    
    if let Ok(config_guard) = state.config.lock() {
        Ok(serde_json::to_string(&OcrCommandResponse::ok(config_guard.clone())).unwrap_or_default())
    } else {
        Ok(serde_json::to_string(&OcrCommandResponse::<OcrConfig>::error("Failed to access config".to_string())).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ocr_commands_basic() {
        let _state = OcrState::new();
        
        // Test d'obtention des langues disponibles
        let response = ocr_get_available_languages().await;
        println!("✅ Language command response: {}", response);
        
        // Test d'obtention de la version
        let response = ocr_get_version().await;
        println!("✅ Version command response: {}", response);
        
        println!("✅ OCR commands basic test completed");
    }
}