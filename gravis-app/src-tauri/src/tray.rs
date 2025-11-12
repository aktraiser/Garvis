// System tray / Menu bar icon pour GRAVIS
use tauri::{
    AppHandle, Manager, Emitter,
    tray::{TrayIconBuilder, MouseButton, MouseButtonState},
    menu::{Menu, MenuItem, PredefinedMenuItem},
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🔧 Creating system tray icon...");

    // Créer le menu contextuel du tray icon
    let menu = create_tray_menu(app)?;
    tracing::info!("✅ Tray menu created");

    // Charger l'icône du tray - utiliser l'icône de configuration ou celle par défaut
    // L'icône est spécifiée dans tauri.conf.json: "iconPath": "icons/trayIconTemplate.png"
    // Tauri chargera automatiquement cette icône pour le tray
    // On utilise l'icône par défaut de l'app comme fallback
    let icon = app.default_window_icon()
        .ok_or("No default window icon available")?
        .clone();

    tracing::info!("✅ Using tray icon from configuration");

    // Créer l'icône système avec le menu
    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(icon)
        .tooltip("GRAVIS - AI Assistant")
        .build(app)?;

    tracing::info!("✅ System tray icon built");

    // Attacher les gestionnaires d'événements APRÈS la création du tray
    let app_handle = app.clone();
    tray.on_menu_event(move |app, event| {
        tracing::info!("🎯 Tray menu event received: {}", event.id().as_ref());
        handle_tray_menu_event(app, event.id().as_ref());
    });

    tray.on_tray_icon_event(|tray, event| {
        // Gestion des clics sur l'icône (ne fonctionne pas sur macOS - bug Tauri 2)
        if let tauri::tray::TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            tracing::info!("🖱️ Tray icon clicked");
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    tracing::info!("Hiding main window");
                    let _ = window.hide();
                } else {
                    tracing::info!("Showing and focusing main window");
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.set_always_on_top(true);
                }
            } else {
                tracing::warn!("Main window not found");
            }
        }
    });

    tracing::info!("✅ System tray icon created successfully with event handlers");
    Ok(())
}

fn create_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    // Ouvrir GRAVIS
    let show_item = MenuItem::with_id(app, "show", "Ouvrir GRAVIS", true, None::<&str>)?;
    menu.append(&show_item)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Nouvelle conversation
    let new_conv = MenuItem::with_id(app, "new_conversation", "Nouvelle conversation", true, None::<&str>)?;
    menu.append(&new_conv)?;

    // Fenêtre RAG
    let rag = MenuItem::with_id(app, "open_rag", "Fenêtre RAG", true, None::<&str>)?;
    menu.append(&rag)?;

    // Sélecteur de modèle
    let model = MenuItem::with_id(app, "open_model_selector", "Sélecteur de modèle", true, None::<&str>)?;
    menu.append(&model)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Préférences
    let prefs = MenuItem::with_id(app, "preferences", "Préférences...", true, None::<&str>)?;
    menu.append(&prefs)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Quitter
    let quit = MenuItem::with_id(app, "quit", "Quitter GRAVIS", true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}

fn handle_tray_menu_event(app: &AppHandle, event_id: &str) {
    tracing::info!("🎯 Tray menu event received: {}", event_id);
    match event_id {
        "show" => {
            tracing::info!("📱 Tray: Showing main window");
            if let Some(window) = app.get_webview_window("main") {
                tracing::info!("✅ Main window found, showing...");
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.set_always_on_top(true);
                tracing::info!("✅ Window shown and focused");
            } else {
                tracing::error!("❌ Main window not found!");
            }
        }
        "new_conversation" => {
            tracing::info!("📝 Tray: Nouvelle conversation");
            if let Err(e) = app.emit("menu:new-conversation", ()) {
                tracing::error!("Failed to emit new-conversation event: {}", e);
            }
        }
        "open_rag" => {
            tracing::info!("🗄️ Tray: Ouvrir fenêtre RAG");
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::window_commands::open_rag_storage_window(app_clone).await {
                    tracing::error!("Failed to open RAG window: {}", e);
                }
            });
        }
        "open_model_selector" => {
            tracing::info!("🤖 Tray: Ouvrir sélecteur de modèle");
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::window_commands::open_model_selector_window(app_clone).await {
                    tracing::error!("Failed to open model selector: {}", e);
                }
            });
        }
        "preferences" => {
            tracing::info!("⚙️ Tray: Préférences");
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::window_commands::open_settings_window(app_clone).await {
                    tracing::error!("Failed to open settings: {}", e);
                }
            });
        }
        "quit" => {
            tracing::info!("👋 Tray: Quitter");
            app.exit(0);
        }
        _ => {
            tracing::debug!("Unhandled tray menu event: {}", event_id);
        }
    }
}
