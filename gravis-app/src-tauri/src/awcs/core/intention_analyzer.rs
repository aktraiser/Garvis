// GRAVIS AWCS - Intention Analyzer
// Analyse les intentions utilisateur et planifie l'exécution

use crate::awcs::types::*;
use regex::Regex;
use std::collections::HashMap;

/// Analyseur d'intentions utilisateur
#[derive(Debug)]
pub struct IntentionAnalyzer {
    patterns: HashMap<IntentionType, Vec<Regex>>,
}

impl IntentionAnalyzer {
    /// Crée un nouvel analyseur d'intentions
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
        };
        
        analyzer.initialize_patterns();
        analyzer
    }
    
    /// Analyse l'intention d'une requête utilisateur
    pub async fn analyze_intention(
        &self,
        query: &str,
        context: &ContextEnvelope,
    ) -> Result<IntentionResult, AWCSError> {
        tracing::debug!("Analyzing user intention for query: {}", query);
        
        // 1. Classification de l'intention
        let classification = self.classify_intention(query);
        
        // 2. Sélection du contenu pertinent
        let relevant_content = self.select_relevant_content(context, &classification);
        
        // 3. Planification de la stratégie d'exécution
        let strategy = self.plan_execution_strategy(&classification, context);
        
        // 4. Génération des actions suggérées
        let suggested_actions = self.generate_suggested_actions(&classification, context);
        
        tracing::info!(
            "Intention analyzed: type={:?}, confidence={:.2}, content_length={}",
            classification.intention_type,
            classification.confidence,
            relevant_content.len()
        );
        
        Ok(IntentionResult {
            classification,
            relevant_content,
            strategy,
            suggested_actions,
        })
    }
    
    // === Méthodes privées ===
    
    /// Initialise les patterns de reconnaissance d'intentions
    fn initialize_patterns(&mut self) {
        // Patterns pour résumé
        let summary_patterns = vec![
            Regex::new(r"(?i)(résume|résumé|summary|synthèse|points? clés?)").unwrap(),
            Regex::new(r"(?i)(principales? idées?|essentiel|en bref)").unwrap(),
        ];
        self.patterns.insert(IntentionType::Summary, summary_patterns);
        
        // Patterns pour recherche/vérification
        let search_patterns = vec![
            Regex::new(r"(?i)(recherche|vérifie|fact.?check|trouve|cherche)").unwrap(),
            Regex::new(r"(?i)(informations?|données?|sources?)").unwrap(),
        ];
        self.patterns.insert(IntentionType::Search, search_patterns);
        
        // Patterns pour recommandations
        let recommendation_patterns = vec![
            Regex::new(r"(?i)(recommande|propose|suggère|conseille)").unwrap(),
            Regex::new(r"(?i)(actions?|étapes?|que faire)").unwrap(),
        ];
        self.patterns.insert(IntentionType::Recommendation, recommendation_patterns);
        
        // Patterns pour traduction
        let translation_patterns = vec![
            Regex::new(r"(?i)(traduis|translate|en (anglais|français|espagnol))").unwrap(),
            Regex::new(r"(?i)(translation|traduction)").unwrap(),
        ];
        self.patterns.insert(IntentionType::Translation, translation_patterns);
        
        // Patterns pour explication
        let explanation_patterns = vec![
            Regex::new(r"(?i)(explique|qu.est.?ce|comment|pourquoi)").unwrap(),
            Regex::new(r"(?i)(définition|signification|sens)").unwrap(),
        ];
        self.patterns.insert(IntentionType::Explanation, explanation_patterns);
    }
    
    /// Classifie l'intention d'une requête
    fn classify_intention(&self, query: &str) -> IntentionClassification {
        let mut best_match: Option<(IntentionType, f64, Vec<String>)> = None;
        
        for (intention_type, patterns) in &self.patterns {
            let mut total_score = 0.0;
            let mut matched_keywords = Vec::new();
            
            for pattern in patterns {
                if let Some(mat) = pattern.find(query) {
                    total_score += 1.0;
                    matched_keywords.push(mat.as_str().to_string());
                }
            }
            
            if total_score > 0.0 {
                // Calcul de la confidence basé sur le nombre de matches et la longueur de la requête
                let confidence = (total_score / patterns.len() as f64) * 0.8 + 
                                (matched_keywords.len() as f64 / query.split_whitespace().count() as f64) * 0.2;
                
                if let Some((_, current_confidence, _)) = &best_match {
                    if confidence > *current_confidence {
                        best_match = Some((intention_type.clone(), confidence, matched_keywords));
                    }
                } else {
                    best_match = Some((intention_type.clone(), confidence, matched_keywords));
                }
            }
        }
        
        match best_match {
            Some((intention_type, confidence, keywords)) => {
                IntentionClassification {
                    intention_type,
                    confidence: confidence.min(1.0),
                    keywords,
                }
            },
            None => {
                // Intention générale par défaut
                IntentionClassification {
                    intention_type: IntentionType::General,
                    confidence: 0.5,
                    keywords: vec![],
                }
            }
        }
    }
    
    /// Sélectionne le contenu pertinent basé sur l'intention et le contexte
    fn select_relevant_content(&self, context: &ContextEnvelope, classification: &IntentionClassification) -> String {
        // Priorité à la sélection utilisateur si disponible et suffisamment longue
        if let Some(ref selection) = context.content.selection {
            if selection.trim().len() > 50 {
                tracing::debug!("Using user selection as primary content ({} chars)", selection.len());
                return selection.clone();
            }
        }
        
        // Sinon, utiliser le texte complet si la confidence d'extraction est élevée
        if let Some(ref fulltext) = context.content.fulltext {
            if context.confidence.text_completeness > 0.8 {
                tracing::debug!("Using full document text (high confidence: {:.1}%)", 
                               context.confidence.text_completeness * 100.0);
                
                // Limiter la taille pour certaines intentions
                match classification.intention_type {
                    IntentionType::Summary | IntentionType::Translation => {
                        if fulltext.len() > 2000 {
                            return format!("{}...", &fulltext[..2000]);
                        }
                    },
                    _ => {}
                }
                
                return fulltext.clone();
            }
        }
        
        // Fallback sur la sélection même si courte
        if let Some(ref selection) = context.content.selection {
            if !selection.trim().is_empty() {
                return selection.clone();
            }
        }
        
        // Dernier recours : extrait du texte complet
        if let Some(ref fulltext) = context.content.fulltext {
            if fulltext.len() > 500 {
                return format!("{}...", &fulltext[..500]);
            } else {
                return fulltext.clone();
            }
        }
        
        "Aucun contenu textuel disponible".to_string()
    }
    
    /// Planifie la stratégie d'exécution
    fn plan_execution_strategy(&self, classification: &IntentionClassification, context: &ContextEnvelope) -> ExecutionStrategy {
        match classification.intention_type {
            IntentionType::Summary => ExecutionStrategy {
                approach: "Local LLM summarization".to_string(),
                estimated_duration: 2000,
                requires_web_search: false,
                requires_llm: true,
            },
            IntentionType::Search => ExecutionStrategy {
                approach: "Web search + fact checking".to_string(),
                estimated_duration: 5000,
                requires_web_search: true,
                requires_llm: true,
            },
            IntentionType::Recommendation => ExecutionStrategy {
                approach: "Context analysis + action planning".to_string(),
                estimated_duration: 3000,
                requires_web_search: false,
                requires_llm: true,
            },
            IntentionType::Translation => ExecutionStrategy {
                approach: "Language detection + translation".to_string(),
                estimated_duration: 1500,
                requires_web_search: false,
                requires_llm: true,
            },
            IntentionType::Explanation => ExecutionStrategy {
                approach: "Knowledge retrieval + explanation".to_string(),
                estimated_duration: 4000,
                requires_web_search: true,
                requires_llm: true,
            },
            IntentionType::General => ExecutionStrategy {
                approach: "General analysis".to_string(),
                estimated_duration: 2500,
                requires_web_search: false,
                requires_llm: true,
            },
        }
    }
    
    /// Génère les actions suggérées
    fn generate_suggested_actions(&self, classification: &IntentionClassification, context: &ContextEnvelope) -> Vec<SuggestedAction> {
        let mut actions = Vec::new();
        
        // Actions communes
        actions.push(SuggestedAction {
            action_type: "copy".to_string(),
            label: "Copier le résultat".to_string(),
            description: "Copier la réponse dans le presse-papier".to_string(),
        });
        
        // Actions spécifiques à l'intention
        match classification.intention_type {
            IntentionType::Summary => {
                actions.push(SuggestedAction {
                    action_type: "export_note".to_string(),
                    label: "Exporter en note".to_string(),
                    description: "Sauvegarder le résumé comme note".to_string(),
                });
            },
            IntentionType::Search => {
                actions.push(SuggestedAction {
                    action_type: "open_sources".to_string(),
                    label: "Ouvrir les sources".to_string(),
                    description: "Ouvrir les liens de vérification".to_string(),
                });
            },
            IntentionType::Recommendation => {
                actions.push(SuggestedAction {
                    action_type: "create_tasks".to_string(),
                    label: "Créer des tâches".to_string(),
                    description: "Convertir en tâches à faire".to_string(),
                });
            },
            _ => {}
        }
        
        // Actions contextuelles selon l'application source
        if context.source.app.contains("Word") || context.source.app.contains("Pages") {
            actions.push(SuggestedAction {
                action_type: "insert_document".to_string(),
                label: "Insérer dans le document".to_string(),
                description: "Ajouter le résultat au document actif".to_string(),
            });
        }
        
        if let Some(ref doc) = context.document {
            if doc.doc_type == "web" {
                actions.push(SuggestedAction {
                    action_type: "bookmark".to_string(),
                    label: "Marquer la page".to_string(),
                    description: "Sauvegarder cette page avec l'analyse".to_string(),
                });
            }
        }
        
        actions
    }
}

impl Default for IntentionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}