// Menu bar natif macOS pour GRAVIS
use tauri::menu::{Menu, Submenu, MenuItem, PredefinedMenuItem, AboutMetadata, MenuEvent};
use tauri::{AppHandle, Manager, Emitter};

pub fn create_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    // Menu GRAVIS (Application)
    let app_menu = create_app_menu(app)?;
    menu.append(&app_menu)?;

    // Menu Fichier
    let file_menu = create_file_menu(app)?;
    menu.append(&file_menu)?;

    // Menu Édition
    let edit_menu = create_edit_menu(app)?;
    menu.append(&edit_menu)?;

    // Menu Affichage
    let view_menu = create_view_menu(app)?;
    menu.append(&view_menu)?;

    // Menu Fenêtre
    let window_menu = create_window_menu(app)?;
    menu.append(&window_menu)?;

    Ok(menu)
}

fn create_app_menu(app: &AppHandle) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let app_menu = Submenu::new(app, "GRAVIS", true)?;

    // À propos
    let about_metadata = AboutMetadata {
        name: Some("GRAVIS".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        copyright: Some("© 2025 Lucas Bometon".to_string()),
        authors: Some(vec!["Lucas Bometon".to_string()]),
        comments: Some("AI-powered voice assistant with RAG capabilities".to_string()),
        ..Default::default()
    };
    let about = PredefinedMenuItem::about(app, Some("À propos de GRAVIS"), Some(about_metadata))?;
    app_menu.append(&about)?;

    app_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Préférences
    let preferences = MenuItem::with_id(app, "preferences", "Préférences...", true, None::<&str>)?;
    app_menu.append(&preferences)?;

    app_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Hide, Hide Others, Show All
    app_menu.append(&PredefinedMenuItem::hide(app, Some("Masquer GRAVIS"))?)?;
    app_menu.append(&PredefinedMenuItem::hide_others(app, Some("Masquer les autres"))?)?;
    app_menu.append(&PredefinedMenuItem::show_all(app, Some("Tout afficher"))?)?;

    app_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Quitter
    app_menu.append(&PredefinedMenuItem::quit(app, Some("Quitter GRAVIS"))?)?;

    Ok(app_menu)
}

fn create_file_menu(app: &AppHandle) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let file_menu = Submenu::new(app, "Fichier", true)?;

    // Nouvelle conversation
    let new_conversation = MenuItem::with_id(app, "new_conversation", "Nouvelle conversation", true, None::<&str>)?;
    file_menu.append(&new_conversation)?;

    // Ouvrir document
    let open_document = MenuItem::with_id(app, "open_document", "Ouvrir document...", true, None::<&str>)?;
    file_menu.append(&open_document)?;

    file_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Fermer fenêtre
    let close_window = PredefinedMenuItem::close_window(app, Some("Fermer fenêtre"))?;
    file_menu.append(&close_window)?;

    Ok(file_menu)
}

fn create_edit_menu(app: &AppHandle) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let edit_menu = Submenu::new(app, "Édition", true)?;

    // Annuler / Rétablir
    edit_menu.append(&PredefinedMenuItem::undo(app, Some("Annuler"))?)?;
    edit_menu.append(&PredefinedMenuItem::redo(app, Some("Rétablir"))?)?;

    edit_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Cut, Copy, Paste, Select All
    edit_menu.append(&PredefinedMenuItem::cut(app, Some("Couper"))?)?;
    edit_menu.append(&PredefinedMenuItem::copy(app, Some("Copier"))?)?;
    edit_menu.append(&PredefinedMenuItem::paste(app, Some("Coller"))?)?;
    edit_menu.append(&PredefinedMenuItem::select_all(app, Some("Tout sélectionner"))?)?;

    Ok(edit_menu)
}

fn create_view_menu(app: &AppHandle) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let view_menu = Submenu::new(app, "Affichage", true)?;

    // Fenêtre RAG
    let rag_window = MenuItem::with_id(app, "open_rag", "Fenêtre RAG", true, None::<&str>)?;
    view_menu.append(&rag_window)?;

    // Sélecteur de modèle
    let model_selector = MenuItem::with_id(app, "open_model_selector", "Sélecteur de modèle", true, None::<&str>)?;
    view_menu.append(&model_selector)?;

    // Conversations
    let conversations = MenuItem::with_id(app, "open_conversations", "Conversations", true, None::<&str>)?;
    view_menu.append(&conversations)?;

    view_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Développeur
    let devtools = MenuItem::with_id(app, "toggle_devtools", "Outils de développement", true, None::<&str>)?;
    view_menu.append(&devtools)?;

    Ok(view_menu)
}

fn create_window_menu(app: &AppHandle) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let window_menu = Submenu::new(app, "Fenêtre", true)?;

    // Minimize
    window_menu.append(&PredefinedMenuItem::minimize(app, Some("Minimiser"))?)?;

    // Zoom
    window_menu.append(&PredefinedMenuItem::maximize(app, Some("Zoom"))?)?;

    Ok(window_menu)
}

pub fn setup_menu_event_handler(app: &AppHandle, _menu: &Menu<tauri::Wry>) {
    let app_handle = app.clone();

    // Les événements de menu dans Tauri 2 sont gérés via le système d'événements principal
    app.on_menu_event(move |app, event| {
        match event.id().as_ref() {
            // Menu Fichier
            "new_conversation" => {
                tracing::info!("📝 Menu: Nouvelle conversation");
                if let Err(e) = app.emit("menu:new-conversation", ()) {
                    tracing::error!("Failed to emit new-conversation event: {}", e);
                }
            }
            "open_document" => {
                tracing::info!("📄 Menu: Ouvrir document");
                if let Err(e) = app.emit("menu:open-document", ()) {
                    tracing::error!("Failed to emit open-document event: {}", e);
                }
            }

            // Menu Affichage
            "open_rag" => {
                tracing::info!("🗄️ Menu: Ouvrir fenêtre RAG");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::window_commands::open_rag_storage_window(app_clone).await {
                        tracing::error!("Failed to open RAG window: {}", e);
                    }
                });
            }
            "open_model_selector" => {
                tracing::info!("🤖 Menu: Ouvrir sélecteur de modèle");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::window_commands::open_model_selector_window(app_clone).await {
                        tracing::error!("Failed to open model selector: {}", e);
                    }
                });
            }
            "open_conversations" => {
                tracing::info!("💬 Menu: Ouvrir conversations");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::window_commands::open_conversations_window(app_clone).await {
                        tracing::error!("Failed to open conversations: {}", e);
                    }
                });
            }
            "toggle_devtools" => {
                tracing::info!("🔧 Menu: Toggle DevTools");
                if let Some(window) = app.get_webview_window("main") {
                    #[cfg(debug_assertions)]
                    {
                        if window.is_devtools_open() {
                            let _ = window.close_devtools();
                        } else {
                            let _ = window.open_devtools();
                        }
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        tracing::warn!("DevTools only available in development mode");
                    }
                }
            }

            // Menu Application
            "preferences" => {
                tracing::info!("⚙️ Menu: Préférences");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::window_commands::open_settings_window(app_clone).await {
                        tracing::error!("Failed to open settings: {}", e);
                    }
                });
            }

            _ => {
                tracing::debug!("Unhandled menu event: {:?}", event.id());
            }
        }
    });
}
