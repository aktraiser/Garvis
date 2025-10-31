// GRAVIS AWCS - Active Window Context Service
// Phase 1: Core Infrastructure
// Architecture modulaire respectant l'organisation GRAVIS existante

pub mod commands;
pub mod core;
pub mod extractors;
pub mod types;
pub mod utils;

// Re-exports des éléments principaux pour l'API publique
pub use types::{
    ContextEnvelope, WindowInfo, ContentData, ExtractionConfidence,
    AWCSActivationState, AWCSPermissions, IntentionResult, TaskResult
};

pub use core::{
    AWCSManager, ContextExtractor, IntentionAnalyzer
};

pub use commands::{
    awcs_get_current_context, awcs_handle_query, awcs_check_permissions,
    awcs_setup_global_shortcut, awcs_get_state, awcs_cleanup
};

use std::sync::Arc;
use tokio::sync::RwLock;

/// État global AWCS partagé entre les commandes Tauri
#[derive(Debug)]
pub struct AWCSState {
    manager: Arc<RwLock<AWCSManager>>,
    activation_state: Arc<RwLock<AWCSActivationState>>,
}

impl AWCSState {
    /// Crée un nouvel état AWCS
    pub fn new() -> Self {
        tracing::info!("Initializing AWCS State - Phase 2 (Incremental)");
        
        Self {
            manager: Arc::new(RwLock::new(AWCSManager::new())),
            activation_state: Arc::new(RwLock::new(AWCSActivationState::Disabled)),
        }
    }
    
    /// Accès au manager AWCS
    pub fn manager(&self) -> Arc<RwLock<AWCSManager>> {
        self.manager.clone()
    }
    
    /// Accès à l'état d'activation
    pub fn activation_state(&self) -> Arc<RwLock<AWCSActivationState>> {
        self.activation_state.clone()
    }
    
    /// Mise à jour de l'état d'activation
    pub async fn set_activation_state(&self, state: AWCSActivationState) {
        let mut activation_state = self.activation_state.write().await;
        *activation_state = state.clone();
        tracing::info!("AWCS activation state updated to: {:?}", state);
    }
    
    /// Récupération de l'état d'activation actuel
    pub async fn get_activation_state(&self) -> AWCSActivationState {
        let activation_state = self.activation_state.read().await;
        activation_state.clone()
    }
}

impl Default for AWCSState {
    fn default() -> Self {
        Self::new()
    }
}