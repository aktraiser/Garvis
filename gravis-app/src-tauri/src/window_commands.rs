// Window management commands for GRAVIS
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Manager, Emitter};

#[tauri::command]
pub async fn open_rag_storage_window(app: AppHandle) -> Result<(), String> {
    tracing::info!("Creating RAG storage window");
    
    // Avoid creating duplicates
    if app.get_webview_window("rag").is_some() {
        tracing::info!("RAG window already exists, focusing it");
        if let Some(window) = app.get_webview_window("rag") {
            let _ = window.set_focus();
        }
        return Ok(());
    }

    match WebviewWindowBuilder::new(
        &app,
        "rag",                               // window label
        WebviewUrl::App("index.html#rag".into()), // SPA with route /rag
    )
    .title("Storage RAG")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .build() {
        Ok(_) => {
            tracing::info!("RAG storage window created successfully");
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create RAG storage window: {}", e);
            Err(format!("Failed to create RAG storage window: {}", e))
        }
    }
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    tracing::info!("Creating Settings window");
    
    // Avoid creating duplicates
    if app.get_webview_window("settings").is_some() {
        tracing::info!("Settings window already exists, focusing it");
        if let Some(window) = app.get_webview_window("settings") {
            let _ = window.set_focus();
        }
        return Ok(());
    }

    match WebviewWindowBuilder::new(
        &app,
        "settings",                          // window label
        WebviewUrl::App("index.html#settings".into()), // SPA with route /settings
    )
    .title("GRAVIS Settings")
    .inner_size(900.0, 700.0)
    .min_inner_size(600.0, 500.0)
    .resizable(true)
    .center()
    .build() {
        Ok(_) => {
            tracing::info!("Settings window created successfully");
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create Settings window: {}", e);
            Err(format!("Failed to create Settings window: {}", e))
        }
    }
}

#[tauri::command]
pub async fn open_model_selector_window(app: AppHandle) -> Result<(), String> {
    tracing::info!("Creating Model Selector window");
    
    // Avoid creating duplicates
    if app.get_webview_window("model_selector").is_some() {
        tracing::info!("Model Selector window already exists, focusing it");
        if let Some(window) = app.get_webview_window("model_selector") {
            let _ = window.set_focus();
        }
        return Ok(());
    }

    match WebviewWindowBuilder::new(
        &app,
        "model_selector",                    // window label
        WebviewUrl::App("index.html#model_selector".into()), // SPA with route /model_selector
    )
    .title("Model Selection")
    .inner_size(800.0, 600.0)
    .min_inner_size(500.0, 400.0)
    .resizable(true)
    .center()
    .build() {
        Ok(_) => {
            tracing::info!("Model Selector window created successfully");
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create Model Selector window: {}", e);
            Err(format!("Failed to create Model Selector window: {}", e))
        }
    }
}

#[tauri::command]
pub async fn emit_model_changed(app: AppHandle, model: serde_json::Value) -> Result<(), String> {
    tracing::info!("Emitting model_changed event to all windows: {:?}", model);
    
    // Émettre l'événement globalement à toutes les fenêtres
    app.emit("model_changed", model.clone())
        .map_err(|e| format!("Failed to emit global model_changed event: {}", e))?;
    
    // Émettre aussi spécifiquement à chaque fenêtre connue pour plus de robustesse
    let known_windows = ["main", "model_selector", "settings", "rag"];
    for window_label in known_windows.iter() {
        if let Some(window) = app.get_webview_window(window_label) {
            let _ = window.emit("model_changed", model.clone());
            tracing::debug!("Emitted model_changed to window: {}", window_label);
        }
    }
    
    tracing::info!("Model change event broadcasted successfully");
    Ok(())
}

#[tauri::command]
pub async fn broadcast_to_window(
    app: AppHandle, 
    window_label: String, 
    event: String, 
    payload: serde_json::Value
) -> Result<(), String> {
    tracing::info!("Broadcasting {} to window {}", event, window_label);
    
    if let Some(window) = app.get_webview_window(&window_label) {
        window.emit(&event, payload)
            .map_err(|e| format!("Failed to emit {} to {}: {}", event, window_label, e))?;
        tracing::debug!("Successfully broadcasted {} to {}", event, window_label);
    } else {
        tracing::warn!("Window {} not found for broadcasting {}", window_label, event);
        return Err(format!("Window {} not found", window_label));
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_active_windows(app: AppHandle) -> Result<Vec<String>, String> {
    let mut active_windows = Vec::new();
    let known_windows = ["main", "model_selector", "settings", "rag"];
    
    for window_label in known_windows.iter() {
        if app.get_webview_window(window_label).is_some() {
            active_windows.push(window_label.to_string());
        }
    }
    
    tracing::info!("Active windows: {:?}", active_windows);
    Ok(active_windows)
}