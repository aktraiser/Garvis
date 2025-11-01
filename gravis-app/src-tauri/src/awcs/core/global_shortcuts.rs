// GRAVIS AWCS - Global Shortcuts Manager
// Phase 4: Système de raccourcis globaux pour activation AWCS

use crate::awcs::types::*;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri::Emitter;

/// Gestionnaire des raccourcis globaux AWCS
#[derive(Debug)]
pub struct GlobalShortcutManager {
    shortcuts: Vec<String>,
    is_active: bool,
}

impl GlobalShortcutManager {
    /// Crée un nouveau gestionnaire de raccourcis
    pub fn new() -> Self {
        Self {
            shortcuts: vec![],
            is_active: false,
        }
    }
    
    /// Configure et enregistre un raccourci global
    pub async fn register_shortcut(
        &mut self, 
        shortcut: &str,
        _app_handle: tauri::AppHandle,
    ) -> Result<(), AWCSError> {
        tracing::info!("AWCS Phase 4: Shortcut is managed by plugin initialization: {}", shortcut);
        
        // Le raccourci est maintenant géré directement par le plugin lors de l'initialisation
        // Pas besoin d'enregistrement manuel supplémentaire
        self.shortcuts.push(shortcut.to_string());
        self.is_active = true;
        
        tracing::info!("AWCS Phase 4: Global shortcut setup completed: {}", shortcut);
        tracing::info!("AWCS Phase 4: Shortcut events will be handled by with_handler");
        Ok(())
    }
    
    /// Désactive tous les raccourcis
    pub async fn unregister_all(&mut self, app_handle: tauri::AppHandle) -> Result<(), AWCSError> {
        tracing::info!("AWCS Phase 4: Unregistering all global shortcuts");
        
        let shortcut_manager = app_handle.global_shortcut();
        
        for shortcut in &self.shortcuts {
            if let Err(e) = shortcut_manager.unregister(shortcut.as_str()) {
                tracing::warn!("Failed to unregister shortcut {}: {}", shortcut, e);
            }
        }
        
        self.shortcuts.clear();
        self.is_active = false;
        
        tracing::info!("AWCS Phase 4: All global shortcuts unregistered");
        Ok(())
    }
    
    /// Vérifie si les raccourcis sont actifs
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Récupère la liste des raccourcis enregistrés
    pub fn get_registered_shortcuts(&self) -> &Vec<String> {
        &self.shortcuts
    }
}

impl Default for GlobalShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}