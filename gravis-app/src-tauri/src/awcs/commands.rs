// GRAVIS AWCS - Tauri Commands
// Commandes exposées au frontend

use super::types::*;
use super::AWCSState;
use tauri::{AppHandle, State, Emitter};

/// Récupère le contexte de la fenêtre active
#[tauri::command]
pub async fn awcs_get_current_context(
    awcs_state: State<'_, AWCSState>,
) -> Result<ContextEnvelope, String> {
    tracing::info!("Command: awcs_get_current_context - Quick context extraction");
    
    // Délai réduit à 300ms pour réactivité optimale
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    manager.get_current_context().await
        .map_err(|e| e.to_string())
}

/// Traite une requête utilisateur avec le contexte
#[tauri::command]
pub async fn awcs_handle_query(
    query: String,
    context: ContextEnvelope,
    awcs_state: State<'_, AWCSState>,
) -> Result<TaskResult, String> {
    tracing::debug!("Command: awcs_handle_query with query: {}", query);
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    manager.handle_query(query, context).await
        .map_err(|e| e.to_string())
}

/// Vérifie les permissions système
#[tauri::command]
pub async fn awcs_check_permissions(
    awcs_state: State<'_, AWCSState>,
) -> Result<AWCSPermissions, String> {
    tracing::debug!("Command: awcs_check_permissions");
    
    let manager_arc = awcs_state.manager();
    let manager = manager_arc.read().await;
    manager.check_permissions().await
        .map_err(|e| e.to_string())
}

/// Demande les permissions manquantes
#[tauri::command]
pub async fn awcs_request_permissions(
    awcs_state: State<'_, AWCSState>,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_request_permissions");
    
    let manager_arc = awcs_state.manager();
    let manager = manager_arc.read().await;
    manager.request_permissions().await
        .map_err(|e| e.to_string())
}

/// Configure le raccourci global
#[tauri::command]
pub async fn awcs_setup_global_shortcut(
    awcs_state: State<'_, AWCSState>,
    app: AppHandle,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_setup_global_shortcut - Phase 4");
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    
    // Configurer le raccourci global avec app_handle
    manager.setup_global_shortcut(app).await
        .map_err(|e| e.to_string())?;
    
    tracing::info!("AWCS Phase 4: Global shortcut command completed successfully");
    Ok(())
}

/// Récupère l'état d'activation AWCS
#[tauri::command]
pub async fn awcs_get_state(
    awcs_state: State<'_, AWCSState>,
) -> Result<AWCSActivationState, String> {
    tracing::debug!("Command: awcs_get_state");
    
    let state = awcs_state.get_activation_state().await;
    Ok(state)
}

/// Met à jour l'état d'activation AWCS
#[tauri::command]
pub async fn awcs_set_state(
    new_state: AWCSActivationState,
    awcs_state: State<'_, AWCSState>,
    app: AppHandle,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_set_state to: {:?}", new_state);
    
    awcs_state.set_activation_state(new_state.clone()).await;
    
    // Émettre l'événement de changement d'état
    app.emit("awcs-state-changed", serde_json::json!({ "state": new_state }))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Nettoie les ressources AWCS
#[tauri::command]
pub async fn awcs_cleanup(
    awcs_state: State<'_, AWCSState>,
    app: AppHandle,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_cleanup");
    
    let manager_arc = awcs_state.manager();
    {
        let mut manager = manager_arc.write().await;
        manager.cleanup(app.clone()).await
            .map_err(|e| e.to_string())?;
    }
    
    // Mettre à jour l'état
    awcs_state.set_activation_state(AWCSActivationState::Disabled).await;
    
    // Émettre l'événement
    app.emit("awcs-state-changed", serde_json::json!({ 
        "state": AWCSActivationState::Disabled 
    })).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Récupère les métriques AWCS
#[tauri::command]
pub async fn awcs_get_metrics(
    awcs_state: State<'_, AWCSState>,
) -> Result<AWCSMetrics, String> {
    tracing::debug!("Command: awcs_get_metrics");
    
    let manager_arc = awcs_state.manager();
    let manager = manager_arc.read().await;
    let metrics = manager.get_metrics().clone();
    Ok(metrics)
}

/// Récupère la configuration AWCS
#[tauri::command]
pub async fn awcs_get_config(
    awcs_state: State<'_, AWCSState>,
) -> Result<AWCSConfig, String> {
    tracing::debug!("Command: awcs_get_config");
    
    let manager_arc = awcs_state.manager();
    let manager = manager_arc.read().await;
    let config = manager.get_config().clone();
    Ok(config)
}

/// Met à jour la configuration AWCS
#[tauri::command]
pub async fn awcs_update_config(
    config: AWCSConfig,
    awcs_state: State<'_, AWCSState>,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_update_config");
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    manager.update_config(config);
    
    Ok(())
}

/// Ouvre les préférences système pour les permissions
#[tauri::command]
pub async fn awcs_open_system_preferences(
    awcs_state: State<'_, AWCSState>,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_open_system_preferences");
    
    let _manager_arc = awcs_state.manager();
    
    // Utiliser le PermissionsManager pour ouvrir les préférences
    // TODO: Accéder au PermissionsManager depuis le manager
    // Pour l'instant, on implémente directement
    
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| format!("Failed to open preferences: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("ms-settings:privacy-speechtyping")
            .spawn()
            .map_err(|e| format!("Failed to open settings: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Essayer quelques options communes
        let commands = vec!["gnome-control-center", "systemsettings5", "unity-control-center"];
        
        for cmd in commands {
            if let Ok(_) = tokio::process::Command::new(cmd).spawn() {
                break;
            }
        }
    }
    
    Ok(())
}

/// Affiche le sélecteur de zone pour OCR
#[tauri::command]
pub async fn awcs_show_zone_selector(
    _app: AppHandle,
) -> Result<SelectionResult, String> {
    tracing::debug!("Command: awcs_show_zone_selector");
    
    // TODO: Implémentation de l'overlay de sélection
    // Pour l'instant, on retourne un résultat simulé
    
    Ok(SelectionResult {
        text: "Texte sélectionné via zone OCR (simulation)".to_string(),
        confidence: 0.85,
        coordinates: Some(SelectionCoordinates {
            x: 100,
            y: 100,
            width: 300,
            height: 200,
        }),
        method: "zone_ocr".to_string(),
    })
}

/// Émule l'événement de raccourci global (pour les tests)
#[tauri::command]
pub async fn awcs_trigger_shortcut(
    app: AppHandle,
) -> Result<(), String> {
    tracing::debug!("Command: awcs_trigger_shortcut (test)");
    
    // Émettre l'événement de raccourci global
    app.emit("awcs-shortcut-triggered", serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Teste l'extraction sur une application spécifique
#[tauri::command]
pub async fn awcs_test_extraction(
    app_name: String,
    awcs_state: State<'_, AWCSState>,
) -> Result<bool, String> {
    tracing::debug!("Command: awcs_test_extraction for: {}", app_name);
    
    // Créer une WindowInfo de test
    let _test_window = WindowInfo {
        app: app_name,
        title: "Test Window".to_string(),
        pid: std::process::id(),
        bundle_id: None,
        window_class: None,
    };
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    
    match manager.get_current_context().await {
        Ok(context) => {
            tracing::info!(
                "Test extraction successful: {} characters extracted",
                context.content.fulltext.as_ref().map(|t| t.len()).unwrap_or(0)
            );
            Ok(true)
        },
        Err(e) => {
            tracing::warn!("Test extraction failed: {}", e);
            Ok(false)
        }
    }
}

/// Force l'extraction OCR pour toute application (mode universel)
#[tauri::command]
pub async fn awcs_get_context_ocr_direct(
    awcs_state: State<'_, AWCSState>,
) -> Result<ContextEnvelope, String> {
    tracing::info!("Command: awcs_get_context_ocr_direct - Force OCR mode");
    
    // Délai de 2 secondes pour permettre de changer de fenêtre
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let manager_arc = awcs_state.manager();
    let mut manager = manager_arc.write().await;
    
    // Utiliser l'extraction OCR directe (méthode existante préservée)
    manager.get_current_context().await
        .map_err(|e| e.to_string())
}

/// NOUVEAU : Extraction OCR avec capture de fenêtre focalisée (amélioration Phase 3)
#[tauri::command]
pub async fn awcs_get_context_focused_ocr(
    awcs_state: State<'_, AWCSState>,
) -> Result<ContextEnvelope, String> {
    tracing::info!("Command: awcs_get_context_focused_ocr - Quick focused OCR");
    
    // Délai réduit à 300ms pour réactivité optimale  
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // 1. Détection de la fenêtre active
    use crate::awcs::extractors::window_detector::WindowDetector;
    let mut window_detector = WindowDetector::new();
    let window_info = window_detector.get_current_window().await
        .map_err(|e| e.to_string())?;
    
    // 2. Extraction OCR focalisée (nouvelle méthode)
    use crate::awcs::extractors::ocr_extractor::OCRExtractor;
    let mut ocr_extractor = OCRExtractor::new();
    let ocr_result = ocr_extractor.extract_from_focused_window(&window_info).await
        .map_err(|e| e.to_string())?;
    
    // 3. Construction de l'enveloppe de contexte
    Ok(ContextEnvelope {
        source: window_info,
        document: None,
        content: ContentData {
            selection: None,
            fulltext: Some(ocr_result.text),
            metadata: Some(serde_json::json!({
                "processing_time_ms": ocr_result.processing_time_ms,
                "extraction_method": "focused_ocr"
            })),
        },
        confidence: ExtractionConfidence {
            text_completeness: ocr_result.confidence,
            source_reliability: 0.9, // Méthode focalisée plus fiable
            extraction_method: "focused_ocr".to_string(),
        },
        timestamp: chrono::Utc::now(),
        security_flags: None,
    })
}

/// Commandes pour les événements personnalisés
pub mod events {
    use tauri::{AppHandle, Emitter};
    use serde_json::json;
    
    /// Émet un événement d'erreur AWCS
    pub fn emit_awcs_error(app: &AppHandle, error: &str) {
        let _ = app.emit("awcs-error", json!({ "error": error }));
    }
    
    /// Émet un événement de changement de permissions
    pub fn emit_permissions_changed(app: &AppHandle, permissions: &super::AWCSPermissions) {
        let _ = app.emit("awcs-permissions-changed", json!({ "permissions": permissions }));
    }
    
    /// Émet un événement d'activation AWCS
    pub fn emit_awcs_activated(app: &AppHandle, context: &super::ContextEnvelope) {
        let _ = app.emit("awcs-activated", json!({ "context": context }));
    }
    
    /// Émet un événement de traitement de requête
    pub fn emit_query_processed(app: &AppHandle, query: &str, result: &super::TaskResult) {
        let _ = app.emit("awcs-query-processed", json!({ 
            "query": query, 
            "result": result 
        }));
    }
}