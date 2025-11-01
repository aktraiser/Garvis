// GRAVIS - Application principale avec module RAG intégré
// Phase 2: Intégration complète RAG + OCR Command-based

// Module RAG (Phase 2 - avec OCR intégré)
pub mod rag;
// Module AWCS (Phase 1 - Core)
pub mod awcs;
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
use awcs::AWCSState;
use awcs::commands::{
    awcs_get_current_context, awcs_handle_query, awcs_check_permissions, awcs_request_permissions,
    awcs_setup_global_shortcut, awcs_get_state, awcs_set_state, awcs_cleanup, awcs_get_metrics,
    awcs_get_config, awcs_update_config, awcs_open_system_preferences, awcs_show_zone_selector,
    awcs_trigger_shortcut, awcs_test_extraction, awcs_get_context_ocr_direct, awcs_get_context_focused_ocr
};
use window_commands::{open_rag_storage_window, open_settings_window, open_model_selector_window, open_conversations_window, emit_model_changed, emit_parameters_changed, broadcast_to_window, get_active_windows};


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

    tracing::info!("GRAVIS starting with RAG Module Phase 2 + OCR Integration + AWCS Phase 1");

    // Créer l'état OCR global
    let ocr_state = OcrState::new();
    
    // Créer l'état RAG unifié avec classification Phase 3
    let rag_state = RagState::new().await.map_err(|e| {
        tracing::error!("Failed to initialize RagState: {}", e);
        e
    })?;
    
    // Créer l'état AWCS Phase 2 (incrémental)
    let awcs_state = AWCSState::new();

    tauri::Builder::default()
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
        .manage(awcs_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Window Management Commands
            open_rag_storage_window,
            open_settings_window,
            open_model_selector_window,
            open_conversations_window,
            emit_model_changed,
            emit_parameters_changed,
            broadcast_to_window,
            get_active_windows,
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
            get_document_metadata,
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}
