// GRAVIS - Application principale avec module RAG intégré
// Phase 2: Intégration complète RAG + OCR Command-based

// Module RAG (Phase 2 - avec OCR intégré)
pub mod rag;
// Window management commands
mod window_commands;

use rag::{DocumentGroup, OcrState, RagState};
use rag::ocr::commands::{
    ocr_initialize, ocr_process_image, ocr_get_available_languages,
    ocr_get_version, ocr_get_cache_stats, ocr_clear_cache, ocr_get_config
};
use rag::commands::{
    add_document_intelligent, search_with_metadata, get_document_metadata
};
use window_commands::{open_rag_storage_window, open_settings_window, open_model_selector_window};


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


// === RAG Commands (Phase 1 - Basic) ===

#[tauri::command]
async fn rag_create_group(name: String) -> Result<DocumentGroup, String> {
    tracing::info!("Creating RAG group: {}", name);
    
    let group = DocumentGroup::new(name);
    
    // TODO: Persister en base de données (Phase 2)
    tracing::info!("Created group with ID: {}", group.id);
    
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


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging pour le debugging RAG + OCR (Phase 2)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("GRAVIS starting with RAG Module Phase 2 + OCR Integration");

    // Créer l'état OCR global
    let ocr_state = OcrState::new();
    
    // Créer l'état RAG unifié avec classification Phase 3
    let rag_state = RagState::new().await.map_err(|e| {
        tracing::error!("Failed to initialize RagState: {}", e);
        e
    })?;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ocr_state)
        .manage(rag_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Window Management Commands
            open_rag_storage_window,
            open_settings_window,
            open_model_selector_window,
            // RAG Commands Phase 1
            rag_create_group,
            rag_list_groups,
            rag_get_status,
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
            get_document_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}
