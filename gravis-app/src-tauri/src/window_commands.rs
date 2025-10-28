// Window management commands for GRAVIS
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Manager};

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