// Window management commands for GRAVIS
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Manager, Emitter, PhysicalPosition, PhysicalSize};

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
    .always_on_top(true)
    .build() {
        Ok(window) => {
            tracing::info!("Settings window created successfully");
            // Donner le focus à la nouvelle fenêtre
            let _ = window.set_focus();
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
    .always_on_top(true)
    .build() {
        Ok(window) => {
            tracing::info!("Model Selector window created successfully");
            // Donner le focus à la nouvelle fenêtre
            let _ = window.set_focus();
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create Model Selector window: {}", e);
            Err(format!("Failed to create Model Selector window: {}", e))
        }
    }
}

#[tauri::command]
pub async fn open_conversations_window(app: AppHandle) -> Result<(), String> {
    tracing::info!("Creating Conversations window");
    
    // Avoid creating duplicates
    if app.get_webview_window("conversations").is_some() {
        tracing::info!("Conversations window already exists, focusing it");
        if let Some(window) = app.get_webview_window("conversations") {
            let _ = window.set_focus();
        }
        return Ok(());
    }

    match WebviewWindowBuilder::new(
        &app,
        "conversations",                         // window label
        WebviewUrl::App("index.html#conversations".into()), // SPA with route /conversations
    )
    .title("Historique des Conversations")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .always_on_top(true)
    .build() {
        Ok(window) => {
            tracing::info!("Conversations window created successfully");
            // Donner le focus à la nouvelle fenêtre
            let _ = window.set_focus();
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create Conversations window: {}", e);
            Err(format!("Failed to create Conversations window: {}", e))
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
    let known_windows = ["main", "model_selector", "settings", "rag", "conversations"];
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
pub async fn emit_parameters_changed(app: AppHandle, parameters: serde_json::Value) -> Result<(), String> {
    tracing::info!("Emitting parameters_changed event to all windows: {:?}", parameters);
    
    // Émettre l'événement globalement à toutes les fenêtres
    app.emit("parameters_changed", parameters.clone())
        .map_err(|e| format!("Failed to emit global parameters_changed event: {}", e))?;
    
    // Émettre aussi spécifiquement à chaque fenêtre connue pour plus de robustesse
    let known_windows = ["main", "model_selector", "settings", "rag", "conversations"];
    for window_label in known_windows.iter() {
        if let Some(window) = app.get_webview_window(window_label) {
            let _ = window.emit("parameters_changed", parameters.clone());
            tracing::debug!("Emitted parameters_changed to window: {}", window_label);
        }
    }
    
    tracing::info!("Parameters change event broadcasted successfully");
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
    let known_windows = ["main", "model_selector", "settings", "rag", "conversations"];
    
    for window_label in known_windows.iter() {
        if app.get_webview_window(window_label).is_some() {
            active_windows.push(window_label.to_string());
        }
    }
    
    tracing::info!("Active windows: {:?}", active_windows);
    Ok(active_windows)
}

#[tauri::command]
pub async fn close_specific_window(app: AppHandle, window_label: String) -> Result<(), String> {
    tracing::info!("Attempting to close window: {}", window_label);

    if let Some(window) = app.get_webview_window(&window_label) {
        window.close().map_err(|e| format!("Failed to close window '{}': {}", window_label, e))?;
        tracing::info!("Successfully closed window: {}", window_label);
        Ok(())
    } else {
        let error_msg = format!("Window '{}' not found", window_label);
        tracing::warn!("{}", error_msg);
        Err(error_msg)
    }
}

#[tauri::command]
pub async fn open_ocr_viewer_window(
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    tracing::info!("Creating OCR Viewer window for session: {}", session_id);

    // Close existing OCR viewer if any
    if let Some(existing) = app.get_webview_window("ocr_viewer") {
        tracing::info!("Closing existing OCR viewer window");
        let _ = existing.close();
    }

    // Get main window position and size to position OCR viewer next to it
    let main_window = app.get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let main_outer_position = main_window.outer_position()
        .map_err(|e| format!("Failed to get main window position: {}", e))?;
    let main_outer_size = main_window.outer_size()
        .map_err(|e| format!("Failed to get main window size: {}", e))?;
    let main_inner_size = main_window.inner_size()
        .map_err(|e| format!("Failed to get main window inner size: {}", e))?;

    // Calculate OCR viewer position (right next to main window, aligned at top)
    // Position X: right after main window
    let ocr_x = main_outer_position.x + main_outer_size.width as i32;
    // Position Y: same as main window (aligned at top)
    let ocr_y = main_outer_position.y;

    // OCR viewer dimensions (same total height as main window)
    let ocr_width = 600; // Width for OCR content
    // Use outer height to match main window's total height including decorations
    let ocr_height = main_outer_size.height;

    match WebviewWindowBuilder::new(
        &app,
        "ocr_viewer",
        WebviewUrl::App(format!("index.html#ocr-viewer?session={}", session_id).into()),
    )
    .title("OCR Document Viewer")
    .inner_size(ocr_width as f64, ocr_height as f64)
    .position(ocr_x as f64, ocr_y as f64)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .build() {
        Ok(ocr_window) => {
            tracing::info!("OCR Viewer window created successfully at position ({}, {})", ocr_x, ocr_y);

            // Setup bidirectional window move synchronization
            let ocr_window_clone = ocr_window.clone();
            let main_window_clone = main_window.clone();
            let ocr_window_clone2 = ocr_window.clone();
            let main_window_clone2 = main_window.clone();

            // Listen for main window move and resize events to sync OCR viewer
            let _ = main_window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Moved(position) => {
                        // Sync position when main window moves
                        if let Ok(main_size) = main_window_clone.outer_size() {
                            let new_ocr_x = position.x + main_size.width as i32;
                            let new_ocr_y = position.y;
                            let _ = ocr_window_clone.set_position(tauri::PhysicalPosition::new(new_ocr_x, new_ocr_y));
                        }
                    }
                    tauri::WindowEvent::Resized(size) => {
                        // Sync height when main window resizes
                        if let Ok(main_position) = main_window_clone.outer_position() {
                            if let Ok(current_size) = main_window_clone.outer_size() {
                                // Update OCR viewer height to match main window
                                let _ = ocr_window_clone.set_size(tauri::PhysicalSize::new(600, current_size.height));
                                // Also update position to stay aligned
                                let new_ocr_x = main_position.x + current_size.width as i32;
                                let _ = ocr_window_clone.set_position(tauri::PhysicalPosition::new(new_ocr_x, main_position.y));
                            }
                        }
                    }
                    _ => {}
                }
            });

            // Listen for OCR window move events and sync main window
            let _ = ocr_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Moved(position) = event {
                    if let Ok(main_size) = main_window_clone2.outer_size() {
                        let new_main_x = position.x - main_size.width as i32;
                        let new_main_y = position.y;
                        let _ = main_window_clone2.set_position(tauri::PhysicalPosition::new(new_main_x, new_main_y));
                    }
                }
            });

            // Focus main window to keep it active
            let _ = main_window.set_focus();
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to create OCR Viewer window: {}", e);
            Err(format!("Failed to create OCR Viewer window: {}", e))
        }
    }
}

#[tauri::command]
pub async fn close_ocr_viewer_window(app: AppHandle) -> Result<(), String> {
    tracing::info!("Closing OCR Viewer window");

    if let Some(window) = app.get_webview_window("ocr_viewer") {
        window.close().map_err(|e| format!("Failed to close OCR viewer: {}", e))?;
        tracing::info!("OCR Viewer window closed successfully");
        Ok(())
    } else {
        tracing::warn!("OCR Viewer window not found");
        Err("OCR Viewer window not found".to_string())
    }
}

#[tauri::command]
pub async fn update_ocr_viewer_highlights(
    app: AppHandle,
    spans: serde_json::Value,
) -> Result<(), String> {
    tracing::info!("Updating OCR viewer with highlighted spans");

    if let Some(window) = app.get_webview_window("ocr_viewer") {
        window.emit("update_highlights", spans)
            .map_err(|e| format!("Failed to emit highlights to OCR viewer: {}", e))?;
        tracing::debug!("Highlights updated successfully");
        Ok(())
    } else {
        tracing::warn!("OCR Viewer window not found for highlight update");
        Err("OCR Viewer window not found".to_string())
    }
}