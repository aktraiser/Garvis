// GRAVIS AWCS - Manager Principal
// Orchestre l'extraction de contexte et l'analyse d'intentions

use super::extractor::ContextExtractor;
use super::intention_analyzer::IntentionAnalyzer;
use super::permissions::PermissionsManager;
use super::global_shortcuts::GlobalShortcutManager; // Phase 4
use crate::awcs::types::*;
use std::time::Instant;
use std::collections::HashMap;

/// Gestionnaire principal AWCS
#[derive(Debug)]
pub struct AWCSManager {
    extractor: ContextExtractor,
    analyzer: IntentionAnalyzer,
    permissions: PermissionsManager,
    shortcuts: GlobalShortcutManager, // Phase 4: Raccourcis globaux
    config: AWCSConfig,
    metrics: AWCSMetrics,
}

impl AWCSManager {
    /// Crée un nouveau gestionnaire AWCS
    pub fn new() -> Self {
        tracing::info!("Initializing AWCS Manager - Phase 2 (Incremental)");
        
        Self {
            extractor: ContextExtractor::new(),
            analyzer: IntentionAnalyzer::new(),
            permissions: PermissionsManager::new(),
            shortcuts: GlobalShortcutManager::new(), // Phase 4
            config: AWCSConfig::default(),
            metrics: AWCSMetrics::default(),
        }
    }
    
    /// Configure AWCS avec des paramètres personnalisés
    pub fn with_config(mut self, config: AWCSConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Extrait le contexte de la fenêtre active
    pub async fn get_current_context(&mut self) -> Result<ContextEnvelope, AWCSError> {
        let start_time = Instant::now();
        
        tracing::debug!("Starting context extraction");
        
        // 1. Vérification des permissions
        if !self.permissions.check_required_permissions().await? {
            return Err(AWCSError::PermissionsInsufficient(
                "Required permissions not granted".to_string()
            ));
        }
        
        // 2. Extraction du contexte
        let context = self.extractor.extract_current_window_context().await?;
        
        // 3. Validation de sécurité
        let validated_context = self.apply_security_filters(context).await?;
        
        // 4. Mise à jour des métriques
        let extraction_time = start_time.elapsed().as_millis() as f64;
        self.update_extraction_metrics(&validated_context, extraction_time);
        
        tracing::info!(
            "Context extracted successfully: app={}, method={}, completeness={:.2}%",
            validated_context.source.app,
            validated_context.confidence.extraction_method,
            validated_context.confidence.text_completeness * 100.0
        );
        
        Ok(validated_context)
    }
    
    /// Traite une requête utilisateur avec le contexte
    pub async fn handle_query(
        &mut self,
        query: String,
        context: ContextEnvelope,
    ) -> Result<TaskResult, AWCSError> {
        let start_time = Instant::now();
        
        tracing::debug!("Processing user query: {}", query);
        
        // 1. Analyse de l'intention
        let intention = self.analyzer.analyze_intention(&query, &context).await?;
        
        // 2. Exécution de la tâche selon l'intention
        let result = self.execute_task(intention, context).await?;
        
        // 3. Mise à jour des métriques
        let execution_time = start_time.elapsed().as_millis() as u64;
        self.update_query_metrics(&result, execution_time);
        
        Ok(result)
    }
    
    /// Vérifie les permissions système
    pub async fn check_permissions(&self) -> Result<AWCSPermissions, AWCSError> {
        self.permissions.get_current_permissions().await
    }
    
    /// Demande les permissions manquantes
    pub async fn request_permissions(&self) -> Result<(), AWCSError> {
        self.permissions.request_missing_permissions().await
    }
    
    /// Configure le raccourci global
    pub async fn setup_global_shortcut(&mut self, app_handle: tauri::AppHandle) -> Result<(), AWCSError> {
        tracing::info!("AWCS Phase 4: Setting up global shortcut: {}", self.config.global_shortcut);
        
        // Enregistrer le raccourci avec le gestionnaire
        self.shortcuts.register_shortcut(&self.config.global_shortcut, app_handle).await?;
        
        tracing::info!("AWCS Phase 4: Global shortcut setup completed");
        Ok(())
    }
    
    /// Nettoie les ressources AWCS
    pub async fn cleanup(&mut self, app_handle: tauri::AppHandle) -> Result<(), AWCSError> {
        tracing::info!("AWCS Phase 4: Cleaning up AWCS resources");
        
        // Désactiver les raccourcis globaux
        self.shortcuts.unregister_all(app_handle).await?;
        
        // Réinitialiser les métriques
        self.metrics = AWCSMetrics::default();
        
        tracing::info!("AWCS Phase 4: Cleanup completed");
        Ok(())
    }
    
    /// Récupère les métriques actuelles
    pub fn get_metrics(&self) -> &AWCSMetrics {
        &self.metrics
    }
    
    /// Récupère la configuration actuelle
    pub fn get_config(&self) -> &AWCSConfig {
        &self.config
    }
    
    /// Met à jour la configuration
    pub fn update_config(&mut self, config: AWCSConfig) {
        self.config = config;
        tracing::info!("AWCS configuration updated");
    }
    
    // === Méthodes privées ===
    
    /// Applique les filtres de sécurité au contexte
    async fn apply_security_filters(
        &self,
        mut context: ContextEnvelope,
    ) -> Result<ContextEnvelope, AWCSError> {
        // 1. Vérification application bloquée
        if self.config.blocked_apps.iter().any(|app| context.source.app.contains(app)) {
            // Mode sécurisé pour applications sensibles
            context.content.fulltext = None;
            context.security_flags = Some(SecurityFlags {
                pii_redacted: true,
                full_text_blocked: true,
                ocr_degraded: true,
            });
        }
        
        // 2. Redaction PII si activée
        if self.config.pii_redaction_enabled {
            context = self.apply_pii_redaction(context).await?;
        }
        
        // 3. Limitation de taille
        if let Some(ref mut fulltext) = context.content.fulltext {
            if fulltext.len() > self.config.max_content_length {
                fulltext.truncate(self.config.max_content_length);
                tracing::warn!("Content truncated to {} characters", self.config.max_content_length);
            }
        }
        
        Ok(context)
    }
    
    /// Applique la redaction PII
    async fn apply_pii_redaction(
        &self,
        mut context: ContextEnvelope,
    ) -> Result<ContextEnvelope, AWCSError> {
        // TODO: Implémentation de la redaction PII avec regex/NER
        // Pour l'instant, on marque simplement comme traité
        if let Some(ref mut flags) = context.security_flags {
            flags.pii_redacted = true;
        } else {
            context.security_flags = Some(SecurityFlags {
                pii_redacted: true,
                full_text_blocked: false,
                ocr_degraded: false,
            });
        }
        
        Ok(context)
    }
    
    /// Exécute une tâche selon l'intention
    async fn execute_task(
        &self,
        intention: IntentionResult,
        context: ContextEnvelope,
    ) -> Result<TaskResult, AWCSError> {
        let start_time = Instant::now();
        
        // Sélection du contenu à traiter
        let content = intention.relevant_content;
        
        // Simulation d'exécution selon le type d'intention
        let result = match intention.classification.intention_type {
            IntentionType::Summary => {
                format!("Résumé du contenu de {} :\n\n• Point clé 1\n• Point clé 2\n• Point clé 3", context.source.app)
            },
            IntentionType::Search => {
                format!("Recherche d'informations liées au contenu de {}", context.source.app)
            },
            IntentionType::Recommendation => {
                format!("Recommandations basées sur le contenu de {} :\n\n1. Action suggérée 1\n2. Action suggérée 2\n3. Action suggérée 3", context.source.app)
            },
            IntentionType::Translation => {
                format!("Traduction du contenu sélectionné depuis {}", context.source.app)
            },
            IntentionType::Explanation => {
                format!("Explication du contenu affiché dans {}", context.source.app)
            },
            IntentionType::General => {
                format!("Analyse générale du contenu de {}", context.source.app)
            },
        };
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(TaskResult {
            task_type: format!("{:?}", intention.classification.intention_type),
            result,
            suggested_actions: intention.suggested_actions,
            execution_time,
            success: true,
        })
    }
    
    /// Met à jour les métriques d'extraction
    fn update_extraction_metrics(&mut self, context: &ContextEnvelope, extraction_time: f64) {
        self.metrics.extractions_total += 1;
        
        // Mise à jour du temps moyen
        let total_extractions = self.metrics.extractions_total as f64;
        self.metrics.avg_extraction_time = 
            (self.metrics.avg_extraction_time * (total_extractions - 1.0) + extraction_time) / total_extractions;
        
        // Distribution des méthodes
        *self.metrics.method_distribution
            .entry(context.confidence.extraction_method.clone())
            .or_insert(0) += 1;
        
        // Compatibilité par application
        let app_name = &context.source.app;
        let current_compat = self.metrics.app_compatibility
            .get(app_name)
            .unwrap_or(&0.0);
        
        // Calcul simple de compatibilité basé sur la complétude
        let new_compat = (current_compat + context.confidence.text_completeness) / 2.0;
        self.metrics.app_compatibility.insert(app_name.clone(), new_compat);
        
        // Taux de succès global
        if context.confidence.text_completeness > 0.5 {
            let successes = (self.metrics.extraction_success_rate * (total_extractions - 1.0)) + 1.0;
            self.metrics.extraction_success_rate = successes / total_extractions;
        } else {
            let successes = self.metrics.extraction_success_rate * (total_extractions - 1.0);
            self.metrics.extraction_success_rate = successes / total_extractions;
        }
    }
    
    /// Met à jour les métriques de requête
    fn update_query_metrics(&mut self, result: &TaskResult, _execution_time: u64) {
        // Mise à jour de la précision d'intention basée sur le succès
        if result.success {
            self.metrics.intention_accuracy = 
                (self.metrics.intention_accuracy * 0.9) + (1.0 * 0.1);
        } else {
            self.metrics.intention_accuracy = 
                self.metrics.intention_accuracy * 0.9;
        }
    }
}

impl Default for AWCSManager {
    fn default() -> Self {
        Self::new()
    }
}