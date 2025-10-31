// GRAVIS AWCS - Utilities
// Fonctions utilitaires pour AWCS

use crate::awcs::types::*;

/// Validation des paramètres AWCS
pub struct AWCSValidator;

impl AWCSValidator {
    /// Valide qu'une chaîne de texte n'est pas vide
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), AWCSError> {
        if value.trim().is_empty() {
            Err(AWCSError::InvalidInput(format!("{} cannot be empty", field_name)))
        } else {
            Ok(())
        }
    }
    
    /// Valide qu'un PID est valide
    pub fn validate_pid(pid: u32) -> Result<(), AWCSError> {
        if pid == 0 {
            Err(AWCSError::InvalidInput("PID cannot be 0".to_string()))
        } else {
            Ok(())
        }
    }
}

/// Conversion sécurisée de types
pub struct SafeConverter;

impl SafeConverter {
    /// Convertit un niveau de confiance en pourcentage
    pub fn confidence_to_percentage(confidence: f64) -> u8 {
        ((confidence * 100.0).clamp(0.0, 100.0)) as u8
    }
    
    /// Tronque un texte à une longueur maximale
    pub fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        }
    }
}

/// Formatage et nettoyage de texte
pub struct TextCleaner;

impl TextCleaner {
    /// Nettoie le texte extrait (supprime les espaces multiples, etc.)
    pub fn clean_extracted_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Supprime les données sensibles potentielles (emails, numéros)
    pub fn redact_sensitive_data(text: &str) -> String {
        // Version basique - en production utiliser spaCy NER
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        let phone_regex = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap();
        
        let mut cleaned = email_regex.replace_all(text, "[EMAIL_REDACTED]").to_string();
        cleaned = phone_regex.replace_all(&cleaned, "[PHONE_REDACTED]").to_string();
        
        cleaned
    }
}