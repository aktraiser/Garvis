// GRAVIS - Application principale avec module RAG intégré
// Phase 1: Intégration sécurisée du module RAG

// Module RAG (Phase 1 - structures de base)
pub mod rag;

use rag::DocumentGroup;

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
pub fn run() {
    // Initialiser le logging pour le debugging RAG (simplifié pour Phase 1)
    tracing_subscriber::fmt()
        .init();

    tracing::info!("GRAVIS starting with RAG Module Phase 1");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // RAG Commands Phase 1
            rag_create_group,
            rag_list_groups,
            rag_get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
