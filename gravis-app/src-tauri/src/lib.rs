// GRAVIS - Application principale avec module RAG intégré
// Phase 2: Intégration complète RAG + OCR Command-based

// Module RAG (Phase 2 - avec OCR intégré)
pub mod rag;
// Module AWCS (Phase 1 - Core)
pub mod awcs;
// Extension server (Phase 0 - Spike)
mod ext_server;
// Window management commands
mod window_commands;
// Menu bar natif macOS
mod menu;
// System tray / Menu bar icon
mod tray;

use rag::{DocumentGroup, OcrState, RagState};
use std::path::Path;
use uuid;
use tauri::State;
use rag::ocr::commands::{
    ocr_initialize, ocr_process_image, ocr_get_available_languages,
    ocr_get_version, ocr_get_cache_stats, ocr_clear_cache, ocr_get_config
};
use rag::commands::{
    add_document_intelligent, search_with_metadata, get_document_metadata, list_rag_documents, delete_rag_document, query_rag_with_context
};
use awcs::AWCSState;
use awcs::commands::{
    awcs_get_current_context, awcs_handle_query, awcs_check_permissions, awcs_request_permissions,
    awcs_setup_global_shortcut, awcs_get_state, awcs_set_state, awcs_cleanup, awcs_get_metrics,
    awcs_get_config, awcs_update_config, awcs_open_system_preferences, awcs_show_zone_selector,
    awcs_trigger_shortcut, awcs_test_extraction, awcs_get_context_ocr_direct, awcs_get_context_focused_ocr
};
use window_commands::{open_rag_storage_window, open_settings_window, open_model_selector_window, open_conversations_window, emit_model_changed, emit_parameters_changed, broadcast_to_window, get_active_windows, close_specific_window};


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


// === RAG Commands (Phase 1 - Basic) ===

#[tauri::command]
async fn rag_create_group(name: String, state: State<'_, RagState>) -> Result<DocumentGroup, String> {
    tracing::info!("Creating RAG group: {}", name);
    
    // Pour le groupe par défaut, utiliser un ID fixe, sinon générer automatiquement
    let group_id = if name == "default_group" {
        "default_group".to_string()
    } else {
        format!("group_{}", uuid::Uuid::new_v4().simple())
    };
    
    // Vérifier si le groupe existe déjà
    {
        let groups = state.groups.read().await;
        if groups.contains_key(&group_id) {
            tracing::info!("ℹ️ Group '{}' already exists with ID: {}", name, group_id);
            return groups.get(&group_id).cloned()
                .ok_or_else(|| "Group disappeared during check".to_string());
        }
    }
    
    // Créer le groupe manuellement avec l'ID souhaité
    let now = std::time::SystemTime::now();
    let group = DocumentGroup {
        id: group_id.clone(),
        name: name.clone(),
        active: true,
        chunk_config: crate::rag::ChunkConfig::default(),
        metadata_config: crate::rag::MetadataConfig::default(),
        documents: Vec::new(),
        qdrant_collection: format!("collection_{}", group_id),
        created_at: now,
        updated_at: now,
    };
    
    // Persister le groupe dans l'état RAG
    {
        let mut groups = state.groups.write().await;
        groups.insert(group_id.clone(), group.clone());
    }
    
    tracing::info!("✅ Created and persisted group '{}' with ID: {}", name, group_id);
    
    Ok(group)
}

#[tauri::command]
async fn rag_list_groups() -> Result<Vec<DocumentGroup>, String> {
    tracing::info!("Listing RAG groups");
    
    // TODO: Récupérer depuis la base de données (Phase 2)
    // Pour l'instant, retourner une liste vide pour tester l'intégration
    Ok(vec![])
}

#[tauri::command]
async fn rag_get_status() -> Result<String, String> {
    Ok("RAG Module Phase 1 - Ready".to_string())
}

#[tauri::command]
async fn list_documents() -> Result<Vec<serde_json::Value>, String> {
    use std::fs;
    use std::path::Path;
    use std::env;
    
    tracing::info!("🔍 list_documents called");
    
    // Obtenir le répertoire courant et naviguer vers le dossier exemple
    let current_dir = env::current_dir().map_err(|e| {
        tracing::error!("Failed to get current directory: {}", e);
        format!("Failed to get current directory: {}", e)
    })?;
    
    tracing::info!("📁 Current directory: {:?}", current_dir);
    
    let docs_path = current_dir.parent()
        .ok_or("Failed to get parent directory")?
        .join("exemple");
    let docs_dir = docs_path.as_path();
    
    tracing::info!("📂 Looking for documents in: {:?}", docs_dir);
    tracing::info!("📂 Directory exists: {}", docs_dir.exists());
    
    if !docs_dir.exists() {
        tracing::warn!("⚠️ Documents directory does not exist: {:?}", docs_dir);
        return Ok(vec![]);
    }
    
    let mut documents = Vec::new();
    
    match fs::read_dir(docs_dir) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                            let metadata = fs::metadata(&path).ok();
                            let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                            let size = if size_bytes > 1024 * 1024 {
                                format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
                            } else {
                                format!("{:.1} KB", size_bytes as f64 / 1024.0)
                            };
                            
                            let file_type = if filename.ends_with(".pdf") {
                                "PDF"
                            } else if filename.ends_with(".png") || filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                                "Image"
                            } else {
                                "Unknown"
                            };
                            
                            // Catégorisation automatique
                            let category = if filename.contains("unilever") || filename.contains("PV_AGE") {
                                "Business"
                            } else if filename.contains("2510") || filename.contains("research") {
                                "Academic"
                            } else if filename.contains("contrôle") || filename.contains("technique") {
                                "Legal"
                            } else if file_type == "Image" {
                                "Technical"
                            } else {
                                "Mixed"
                            };
                            
                            let doc = serde_json::json!({
                                "id": (index + 1).to_string(),
                                "name": filename,
                                "size": size,
                                "sizeBytes": size_bytes,
                                "type": file_type,
                                "status": "Ready",
                                "date": metadata.and_then(|m| m.modified().ok())
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| {
                                        let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                            .unwrap_or_else(|| chrono::Utc::now());
                                        datetime.format("%d/%m/%Y").to_string()
                                    })
                                    .unwrap_or_else(|| chrono::Utc::now().format("%d/%m/%Y").to_string()),
                                "category": category,
                                "pages": if file_type == "Image" { 1 } else { 10 }, // Estimation
                                "extracted": false,
                                "extractedAt": "",
                                "confidence": 0
                            });
                            
                            documents.push(doc);
                        }
                    }
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to read documents directory: {}", e);
            return Err(format!("Failed to read documents directory: {}", e));
        }
    }
    
    tracing::info!("📋 Found {} documents", documents.len());
    Ok(documents)
}

#[tauri::command]
async fn delete_document(filename: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    use std::env;
    
    // Obtenir le répertoire courant et naviguer vers le dossier exemple
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let docs_path = current_dir.parent()
        .ok_or("Failed to get parent directory")?
        .join("exemple");
    let file_path = docs_path.join(&filename);
    
    if !file_path.exists() {
        return Err(format!("File '{}' not found", filename));
    }
    
    match fs::remove_file(&file_path) {
        Ok(_) => {
            tracing::info!("Successfully deleted file: {}", filename);
            Ok(format!("File '{}' deleted successfully", filename))
        },
        Err(e) => {
            tracing::error!("Failed to delete file '{}': {}", filename, e);
            Err(format!("Failed to delete file '{}': {}", filename, e))
        }
    }
}

#[tauri::command]
async fn upload_document(file_path: String, target_name: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    use std::env;
    
    // Obtenir le répertoire courant et naviguer vers le dossier exemple
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let docs_path = current_dir.parent()
        .ok_or("Failed to get parent directory")?
        .join("exemple");
    let docs_dir = docs_path.as_path();
    
    // Créer le dossier exemple s'il n'existe pas
    if !docs_dir.exists() {
        fs::create_dir_all(docs_dir)
            .map_err(|e| format!("Failed to create documents directory: {}", e))?;
    }
    
    let source_path = Path::new(&file_path);
    let target_path = docs_dir.join(&target_name);
    
    if !source_path.exists() {
        return Err(format!("Source file '{}' not found", file_path));
    }
    
    match fs::copy(source_path, &target_path) {
        Ok(_) => {
            tracing::info!("Successfully uploaded file: {} -> {}", file_path, target_name);
            Ok(format!("File '{}' uploaded successfully", target_name))
        },
        Err(e) => {
            tracing::error!("Failed to upload file '{}': {}", target_name, e);
            Err(format!("Failed to upload file '{}': {}", target_name, e))
        }
    }
}

#[tauri::command]
async fn open_document_viewer(filename: String) -> Result<String, String> {
    use std::env;
    use std::path::Path;
    use tauri::AppHandle;
    
    // Obtenir le chemin du document
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let docs_path = current_dir.parent()
        .ok_or("Failed to get parent directory")?
        .join("exemple");
    let file_path = docs_path.join(&filename);
    
    if !file_path.exists() {
        return Err(format!("File '{}' not found", filename));
    }
    
    // Ouvrir le fichier avec l'application par défaut du système
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let result = Command::new("open")
            .arg(&file_path)
            .spawn();
        
        match result {
            Ok(_) => Ok(format!("Document '{}' opened successfully", filename)),
            Err(e) => Err(format!("Failed to open document '{}': {}", filename, e))
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let result = Command::new("cmd")
            .args(&["/C", "start", "", &file_path.to_string_lossy()])
            .spawn();
        
        match result {
            Ok(_) => Ok(format!("Document '{}' opened successfully", filename)),
            Err(e) => Err(format!("Failed to open document '{}': {}", filename, e))
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let result = Command::new("xdg-open")
            .arg(&file_path)
            .spawn();
        
        match result {
            Ok(_) => Ok(format!("Document '{}' opened successfully", filename)),
            Err(e) => Err(format!("Failed to open document '{}': {}", filename, e))
        }
    }
}

#[tauri::command]
async fn extract_document_content(filename: String) -> Result<serde_json::Value, String> {
    use std::env;
    use std::path::Path;
    
    tracing::info!("🔍 Starting extraction for document: {}", filename);
    
    // Obtenir le chemin du document
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let docs_path = current_dir.parent()
        .ok_or("Failed to get parent directory")?
        .join("exemple");
    let file_path = docs_path.join(&filename);
    
    if !file_path.exists() {
        return Err(format!("File '{}' not found", filename));
    }
    
    let start_time = std::time::Instant::now();
    
    // Déterminer le type de fichier et extraire le contenu
    let extraction_result = if filename.to_lowercase().ends_with(".pdf") {
        // Stratégie hybride : essayer extractous d'abord, puis fallback vers lopdf
        #[cfg(feature = "extractous")]
        {
            tracing::info!("🚀 Using extractous for PDF extraction (feature enabled)");
            match extract_pdf_content_extractous(&file_path).await {
                Ok(result) if !result.text.trim().is_empty() => {
                    tracing::info!("✅ Extractous extraction successful");
                    Ok(result)
                },
                Ok(_) => {
                    tracing::info!("⚠️ Extractous returned empty text, trying lopdf fallback...");
                    extract_pdf_content_lopdf(&file_path).await
                },
                Err(e) => {
                    tracing::warn!("⚠️ Extractous failed: {}, trying lopdf fallback...", e);
                    extract_pdf_content_lopdf(&file_path).await
                }
            }
        }
        #[cfg(not(feature = "extractous"))]
        {
            tracing::info!("📄 Using lopdf for PDF extraction (extractous feature disabled)");
            extract_pdf_content_lopdf(&file_path).await
        }
    } else if filename.to_lowercase().ends_with(".png") 
           || filename.to_lowercase().ends_with(".jpg") 
           || filename.to_lowercase().ends_with(".jpeg") {
        // Extraction OCR pour les images
        extract_image_content(&file_path).await
    } else {
        return Err(format!("Unsupported file type for '{}'", filename));
    };
    
    let processing_time = start_time.elapsed();
    
    match extraction_result {
        Ok(content) => {
            let result = serde_json::json!({
                "filename": filename,
                "content": content.text,
                "confidence": content.confidence,
                "method": content.method,
                "processing_time_ms": processing_time.as_millis(),
                "pages": content.pages,
                "metadata": content.metadata,
                "extracted_at": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            });
            
            tracing::info!("✅ Extraction completed for '{}' in {}ms", filename, processing_time.as_millis());
            Ok(result)
        },
        Err(e) => {
            tracing::error!("❌ Extraction failed for '{}': {}", filename, e);
            Err(format!("Extraction failed: {}", e))
        }
    }
}

#[tauri::command]
async fn get_document_extraction(filename: String) -> Result<serde_json::Value, String> {
    // Pour l'instant, on retourne un placeholder
    // Dans une vraie implémentation, on pourrait stocker les extractions en cache
    Ok(serde_json::json!({
        "filename": filename,
        "content": "Extraction not yet implemented - placeholder content",
        "confidence": 0.85,
        "method": "placeholder",
        "processing_time_ms": 0,
        "pages": 1,
        "extracted_at": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }))
}

// Structure pour les résultats d'extraction
struct ExtractionResult {
    text: String,
    confidence: f64,
    method: String,
    pages: u32,
    metadata: serde_json::Value,
}

// Extraction PDF avec lopdf
// Nouvelle fonction d'extraction PDF avec extractous (2025)
#[cfg(feature = "extractous")]
async fn extract_pdf_content_extractous(file_path: &Path) -> Result<ExtractionResult, String> {
    use extractous::{Extractor, TesseractOcrConfig, PdfParserConfig, PdfOcrStrategy};
    
    tracing::info!("🚀 Extracting PDF with extractous (OCR): {:?}", file_path);
    
    let extractor = Extractor::new()
        .set_ocr_config(
            TesseractOcrConfig::new()
                .set_language("eng+fra") // Support anglais et français
        )
        .set_pdf_config(
            PdfParserConfig::new()
                .set_ocr_strategy(PdfOcrStrategy::AUTO) // Essayer texte natif d'abord, puis OCR
        );
    
    match extractor.extract_file_to_string(file_path.to_str().unwrap()) {
        Ok((content, metadata)) => {
            let page_count = metadata.get("page_count")
                .and_then(|v| v.first())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            
            Ok(ExtractionResult {
                text: content,
                confidence: 0.85, // extractous ne fournit pas de score de confiance, utiliser une valeur par défaut
                method: "PDF (extractous OCR)".to_string(),
                pages: page_count,
                metadata: serde_json::json!({
                    "pages": page_count,
                    "source": "extractous",
                    "ocr_strategy": "AUTO",
                    "languages": "eng+fra"
                })
            })
        },
        Err(e) => {
            tracing::error!("❌ Extractous extraction failed: {}", e);
            Err(format!("Extractous extraction failed: {}", e))
        }
    }
}

// Fonction d'extraction PDF originale (backup)
async fn extract_pdf_content_lopdf(file_path: &Path) -> Result<ExtractionResult, String> {
    use crate::rag::ocr::pdf_lopdf::{LopdFProcessor, LopdFPipelineConfig};
    
    tracing::info!("📄 Extracting PDF content from: {:?}", file_path);
    
    let mut config = LopdFPipelineConfig::default();
    // Forcer l'OCR même avec peu de texte natif pour les PDFs scannés
    config.min_native_tokens = 10;
    let processor = LopdFProcessor::new(config).await
        .map_err(|e| format!("Failed to initialize PDF processor: {}", e))?;
    
    match processor.process_pdf(file_path).await {
        Ok(pages) => {
            let combined_text = pages.iter()
                .map(|page| {
                    // Prioriser le texte natif, mais utiliser OCR si vide
                    if !page.native_text.trim().is_empty() {
                        page.native_text.clone()
                    } else if let Some(ocr_result) = &page.ocr_result {
                        ocr_result.text.clone()
                    } else {
                        String::new()
                    }
                })
                .filter(|text| !text.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            
            let avg_confidence = if pages.is_empty() { 0.5 } else {
                let mut total_confidence = 0.0;
                let mut count = 0;
                
                for page in &pages {
                    if !page.native_text.trim().is_empty() {
                        // Texte natif = haute confiance
                        total_confidence += 0.95;
                        count += 1;
                    } else if let Some(ocr_result) = &page.ocr_result {
                        // Utiliser la confiance OCR
                        total_confidence += ocr_result.confidence;
                        count += 1;
                    }
                }
                
                if count > 0 { total_confidence / count as f32 } else { 0.5 }
            };
            
            Ok(ExtractionResult {
                text: combined_text,
                confidence: avg_confidence as f64,
                method: "PDF (lopdf)".to_string(),
                pages: pages.len() as u32,
                metadata: serde_json::json!({
                    "pages": pages.len(),
                    "source": "lopdf"
                })
            })
        },
        Err(e) => Err(format!("PDF extraction failed: {}", e))
    }
}

// Extraction OCR pour les images
async fn extract_image_content(file_path: &Path) -> Result<ExtractionResult, String> {
    use crate::rag::ocr::{TesseractProcessor, TesseractConfig};
    
    tracing::info!("🖼️ Extracting image content from: {:?}", file_path);
    
    let config = TesseractConfig::default();
    let processor = TesseractProcessor::new(config).await
        .map_err(|e| format!("Failed to initialize OCR: {}", e))?;
    
    match processor.process_image(file_path).await {
        Ok(result) => {
            Ok(ExtractionResult {
                text: result.text,
                confidence: result.confidence as f64 / 100.0,
                method: "OCR (Tesseract)".to_string(),
                pages: 1,
                metadata: serde_json::json!({
                    "language": result.language,
                    "processing_time": result.processing_time,
                    "engine": result.engine_used
                })
            })
        },
        Err(e) => Err(format!("OCR extraction failed: {}", e))
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging pour le debugging RAG + OCR (Phase 2)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("GRAVIS starting with RAG Module Phase 2 + OCR Integration + AWCS Phase 1 + Extension Server");

    // Créer l'état OCR global
    let ocr_state = OcrState::new();
    
    // Créer l'état RAG unifié avec classification Phase 3
    let rag_state = RagState::new().await.map_err(|e| {
        tracing::error!("Failed to initialize RagState: {}", e);
        e
    })?;
    
    // Créer l'état AWCS Phase 2 (incrémental)
    let awcs_state = AWCSState::new();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["Cmd+Shift+Control+L"])
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    use tauri::Emitter;

                    if event.state == ShortcutState::Pressed {
                        tracing::info!("AWCS Phase 4: Global shortcut triggered! {}", shortcut);

                        // Émettre l'événement pour le frontend
                        if let Err(e) = app.emit("awcs-shortcut-triggered", serde_json::json!({})) {
                            tracing::error!("Failed to emit shortcut event: {}", e);
                        }
                    }
                })
                .build()
        )
        .manage(ocr_state)
        .manage(rag_state)
        .manage(awcs_state);

    // Configurer le menu natif macOS et le system tray
    #[cfg(target_os = "macos")]
    {
        builder = builder.setup(|app| {
            let menu = menu::create_menu(&app.handle()).expect("Failed to create menu");
            menu::setup_menu_event_handler(&app.handle(), &menu);

            // Activer le menu pour toutes les fenêtres
            app.set_menu(menu).expect("Failed to set menu");

            // Créer l'icône système (system tray)
            tray::create_tray(&app.handle()).expect("Failed to create system tray");

            tracing::info!("✅ Menu bar natif macOS configuré");
            Ok(())
        });
    }

    // Configurer le system tray pour les autres OS
    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.setup(|app| {
            tray::create_tray(&app.handle()).expect("Failed to create system tray");
            Ok(())
        });
    }

    let app_handle = builder.invoke_handler(tauri::generate_handler![
            greet,
            // Window Management Commands
            open_rag_storage_window,
            open_settings_window,
            open_model_selector_window,
            open_conversations_window,
            close_specific_window,
            emit_model_changed,
            emit_parameters_changed,
            broadcast_to_window,
            get_active_windows,
            // RAG Commands Phase 1
            rag_create_group,
            rag_list_groups,
            rag_get_status,
            list_documents,
            delete_document,
            upload_document,
            open_document_viewer,
            extract_document_content,
            get_document_extraction,
            // OCR Commands Phase 2
            ocr_initialize,
            ocr_process_image,
            ocr_get_available_languages,
            ocr_get_version,
            ocr_get_cache_stats,
            ocr_clear_cache,
            ocr_get_config,
            // RAG Commands Phase 3 - Unified Intelligence
            add_document_intelligent,
            search_with_metadata,
            get_document_metadata,
            list_rag_documents,
            delete_rag_document,
            query_rag_with_context,
            // AWCS Commands Phase 1 - Core
            awcs_get_current_context,
            awcs_handle_query,
            awcs_check_permissions,
            awcs_request_permissions,
            awcs_setup_global_shortcut,
            awcs_get_state,
            awcs_set_state,
            awcs_cleanup,
            awcs_get_metrics,
            awcs_get_config,
            awcs_update_config,
            awcs_open_system_preferences,
            awcs_show_zone_selector,
            awcs_trigger_shortcut,
            awcs_test_extraction,
            awcs_get_context_ocr_direct,
            awcs_get_context_focused_ocr
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Démarrer le serveur extension après que l'app soit prête
    let app_handle_clone = app_handle.handle().clone();
    if let Err(e) = ext_server::start_extension_server(app_handle_clone).await {
        tracing::error!("Failed to start extension server: {}", e);
    }

    app_handle.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
    
    Ok(())
}
