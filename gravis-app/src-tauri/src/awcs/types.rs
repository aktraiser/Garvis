// GRAVIS AWCS - Types de données
// Définitions des structures de données centrales pour AWCS

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// État d'activation AWCS avec progression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AWCSActivationState {
    Disabled,
    PermissionsPending,
    PermissionsGranted,
    Active,
    Error,
}

/// Permissions système requises pour AWCS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AWCSPermissions {
    pub accessibility: bool,
    pub automation: bool,
    pub screen_recording: bool,
}

/// Informations sur la fenêtre active
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app: String,
    pub title: String,
    pub pid: u32,
    pub bundle_id: Option<String>,     // macOS
    pub window_class: Option<String>,  // Windows/Linux
}

/// Informations sur le document/contenu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub doc_type: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

/// Données de contenu extraites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentData {
    pub selection: Option<String>,
    pub fulltext: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Confidence de l'extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionConfidence {
    pub text_completeness: f64,
    pub source_reliability: f64,
    pub extraction_method: String,
}

/// Drapeaux de sécurité
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFlags {
    pub pii_redacted: bool,
    pub full_text_blocked: bool,
    pub ocr_degraded: bool,
}

/// Enveloppe de contexte unifiée - Structure principale AWCS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextEnvelope {
    pub source: WindowInfo,
    pub document: Option<DocumentInfo>,
    pub content: ContentData,
    pub confidence: ExtractionConfidence,
    pub timestamp: DateTime<Utc>,
    pub security_flags: Option<SecurityFlags>,
}

/// Type d'intention utilisateur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IntentionType {
    Summary,
    Search,
    Recommendation,
    Translation,
    Explanation,
    General,
}

/// Classification d'intention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionClassification {
    pub intention_type: IntentionType,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

/// Résultat d'analyse d'intention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionResult {
    pub classification: IntentionClassification,
    pub relevant_content: String,
    pub strategy: ExecutionStrategy,
    pub suggested_actions: Vec<SuggestedAction>,
}

/// Stratégie d'exécution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStrategy {
    pub approach: String,
    pub estimated_duration: u64, // en millisecondes
    pub requires_web_search: bool,
    pub requires_llm: bool,
}

/// Action suggérée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: String,
    pub label: String,
    pub description: String,
}

/// Résultat de tâche exécutée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_type: String,
    pub result: String,
    pub suggested_actions: Vec<SuggestedAction>,
    pub execution_time: u64,
    pub success: bool,
}

/// Résultat de sélection utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub text: String,
    pub confidence: f64,
    pub coordinates: Option<SelectionCoordinates>,
    pub method: String,
}

/// Coordonnées de sélection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCoordinates {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Erreurs AWCS spécialisées
#[derive(Debug, thiserror::Error)]
pub enum AWCSError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Window detection failed: {0}")]
    WindowDetectionFailed(String),
    
    #[error("Context extraction failed: {0}")]
    ExtractionFailed(String),
    
    #[error("Unsupported application: {0}")]
    UnsupportedApp(String),
    
    #[error("Permissions insufficient: {0}")]
    PermissionsInsufficient(String),
    
    #[error("Script execution failed: {0}")]
    ScriptFailed(String),
    
    #[error("OCR processing failed: {0}")]
    OCRFailed(String),
    
    #[error("Screen capture failed: {0}")]
    ScreenCaptureError(String),
    
    #[error("Intent analysis failed: {0}")]
    IntentAnalysisFailed(String),
    
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
}

/// Configuration AWCS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AWCSConfig {
    pub enabled: bool,
    pub global_shortcut: String,
    pub extraction_timeout: u64,
    pub max_content_length: usize,
    pub pii_redaction_enabled: bool,
    pub allowed_apps: Vec<String>,
    pub blocked_apps: Vec<String>,
    pub security_mode: SecurityMode,
}

/// Mode de sécurité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMode {
    Permissive,
    Balanced,
    Strict,
}

impl Default for AWCSConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            global_shortcut: "Cmd+Shift+Control+L".to_string(),
            extraction_timeout: 5000, // 5 secondes
            max_content_length: 100_000, // 100k caractères
            pii_redaction_enabled: true,
            allowed_apps: vec![
                "Safari".to_string(),
                "Chrome".to_string(),
                "Microsoft Word".to_string(),
                "Code".to_string(),
            ],
            blocked_apps: vec![
                "Keychain Access".to_string(),
                "1Password".to_string(),
            ],
            security_mode: SecurityMode::Balanced,
        }
    }
}

/// Métriques AWCS pour télémétrie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AWCSMetrics {
    pub extractions_total: u64,
    pub extraction_success_rate: f64,
    pub avg_extraction_time: f64,
    pub method_distribution: HashMap<String, u64>,
    pub app_compatibility: HashMap<String, f64>,
    pub intention_accuracy: f64,
    pub user_satisfaction: Option<f64>,
}

impl Default for AWCSMetrics {
    fn default() -> Self {
        Self {
            extractions_total: 0,
            extraction_success_rate: 0.0,
            avg_extraction_time: 0.0,
            method_distribution: HashMap::new(),
            app_compatibility: HashMap::new(),
            intention_accuracy: 0.0,
            user_satisfaction: None,
        }
    }
}