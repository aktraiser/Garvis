// GRAVIS AWCS - Core Logic
// Gestionnaire principal et logique métier centrale

pub mod manager;
pub mod extractor;
pub mod intention_analyzer;
pub mod permissions;
pub mod screen_capture; // Phase 3: Module de capture d'écran natif
pub mod global_shortcuts; // Phase 4: Module de raccourcis globaux

// Re-exports
pub use manager::AWCSManager;
pub use extractor::ContextExtractor;
pub use intention_analyzer::IntentionAnalyzer;
pub use permissions::PermissionsManager;
pub use screen_capture::ScreenCaptureManager; // Phase 3: Capture d'écran native
pub use global_shortcuts::GlobalShortcutManager; // Phase 4: Raccourcis globaux