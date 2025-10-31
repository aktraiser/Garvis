// GRAVIS AWCS - AppleScript Extractor
// Extraction de contenu via AppleScript (macOS uniquement)

use crate::awcs::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extracteur AppleScript pour applications macOS
#[derive(Debug)]
pub struct AppleScriptExtractor {
    scripts: HashMap<String, String>,
    supported_apps: Vec<String>,
}

/// Résultat d'extraction AppleScript
#[derive(Debug, Serialize, Deserialize)]
pub struct AppleScriptResult {
    pub text: String,
    pub doc_type: String,
    pub path: Option<String>,
    pub word_count: usize,
    pub is_protected: bool,
}

impl AppleScriptExtractor {
    /// Crée un nouveau extracteur AppleScript
    pub fn new() -> Self {
        let mut extractor = Self {
            scripts: HashMap::new(),
            supported_apps: vec![
                "Microsoft Word".to_string(),
                "Microsoft Excel".to_string(),
                "Microsoft PowerPoint".to_string(),
                "Pages".to_string(),
                "Numbers".to_string(),
                "Keynote".to_string(),
                "TextEdit".to_string(),
                "Preview".to_string(),
            ],
        };
        
        extractor.initialize_scripts();
        extractor
    }
    
    /// Vérifie si l'application est supportée
    pub fn is_supported_app(&self, app_name: &str) -> bool {
        self.supported_apps
            .iter()
            .any(|app| app_name.contains(app))
    }
    
    /// Extrait le contenu depuis une application
    pub async fn extract_from_app(&self, window: &WindowInfo) -> Result<AppleScriptResult, AWCSError> {
        if cfg!(not(target_os = "macos")) {
            return Err(AWCSError::UnsupportedApp("AppleScript only available on macOS".to_string()));
        }
        
        if !self.is_supported_app(&window.app) {
            return Err(AWCSError::UnsupportedApp(format!("App not supported: {}", window.app)));
        }
        
        tracing::debug!("Extracting content from {} using AppleScript", window.app);
        
        // Sélectionner le script approprié
        let script = self.get_script_for_app(&window.app)?;
        
        // Exécuter le script
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("AppleScript execution failed: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AWCSError::ScriptFailed(
                format!("AppleScript error: {}", error_msg)
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        self.parse_script_result(&result, &window.app)
    }
    
    // === Initialisation des scripts ===
    
    fn initialize_scripts(&mut self) {
        // Script pour Microsoft Word
        self.scripts.insert("Microsoft Word".to_string(), r#"
        tell application "Microsoft Word"
            if exists active document then
                try
                    set docContent to content of text object of active document as string
                    set docPath to full name of active document
                    set docStats to {
                        wordCount: count words of active document,
                        pageCount: count pages of active document,
                        isProtected: protection type of active document is not no protection
                    }
                    return docContent & "|SEPARATOR|" & docPath & "|SEPARATOR|" & (item 1 of docStats) & "|SEPARATOR|" & (item 3 of docStats)
                on error errMsg
                    return "ERROR|SEPARATOR||SEPARATOR|0|SEPARATOR|true|SEPARATOR|" & errMsg
                end try
            else
                return "NO_DOCUMENT|SEPARATOR||SEPARATOR|0|SEPARATOR|false|SEPARATOR|"
            end if
        end tell
        "#.to_string());
        
        // Script pour Microsoft Excel
        self.scripts.insert("Microsoft Excel".to_string(), r#"
        tell application "Microsoft Excel"
            if exists active workbook then
                try
                    set sheetName to name of active sheet
                    set cellText to ""
                    tell active sheet
                        # Récupérer le contenu des cellules visibles
                        set usedRange to get used range
                        if usedRange is not missing value then
                            set cellText to value of usedRange as string
                        end if
                    end tell
                    set workbookPath to full name of active workbook
                    return cellText & "|SEPARATOR|" & workbookPath & "|SEPARATOR|100|SEPARATOR|false"
                on error errMsg
                    return "ERROR|SEPARATOR||SEPARATOR|0|SEPARATOR|true|SEPARATOR|" & errMsg
                end try
            else
                return "NO_DOCUMENT|SEPARATOR||SEPARATOR|0|SEPARATOR|false|SEPARATOR|"
            end if
        end tell
        "#.to_string());
        
        // Script pour Pages
        self.scripts.insert("Pages".to_string(), r#"
        tell application "Pages"
            if exists front document then
                try
                    set docContent to body text of front document
                    set docPath to ""
                    try
                        set docPath to file of front document as string
                    end try
                    set wordCount to count words of front document
                    return docContent & "|SEPARATOR|" & docPath & "|SEPARATOR|" & wordCount & "|SEPARATOR|false"
                on error errMsg
                    return "ERROR|SEPARATOR||SEPARATOR|0|SEPARATOR|true|SEPARATOR|" & errMsg
                end try
            else
                return "NO_DOCUMENT|SEPARATOR||SEPARATOR|0|SEPARATOR|false|SEPARATOR|"
            end if
        end tell
        "#.to_string());
        
        // Script pour TextEdit
        self.scripts.insert("TextEdit".to_string(), r#"
        tell application "TextEdit"
            if exists front document then
                try
                    set docContent to text of front document
                    set docPath to ""
                    try
                        set docPath to path of front document
                    end try
                    set wordCount to count words of front document
                    return docContent & "|SEPARATOR|" & docPath & "|SEPARATOR|" & wordCount & "|SEPARATOR|false"
                on error errMsg
                    return "ERROR|SEPARATOR||SEPARATOR|0|SEPARATOR|true|SEPARATOR|" & errMsg
                end try
            else
                return "NO_DOCUMENT|SEPARATOR||SEPARATOR|0|SEPARATOR|false|SEPARATOR|"
            end if
        end tell
        "#.to_string());
        
        // Script pour Preview
        self.scripts.insert("Preview".to_string(), r#"
        tell application "Preview"
            if exists front document then
                try
                    set docPath to path of front document
                    # Preview ne permet pas d'extraire directement le texte via AppleScript
                    # On retourne des informations de base
                    return "PDF/Image content from Preview|SEPARATOR|" & docPath & "|SEPARATOR|0|SEPARATOR|false"
                on error errMsg
                    return "ERROR|SEPARATOR||SEPARATOR|0|SEPARATOR|true|SEPARATOR|" & errMsg
                end try
            else
                return "NO_DOCUMENT|SEPARATOR||SEPARATOR|0|SEPARATOR|false|SEPARATOR|"
            end if
        end tell
        "#.to_string());
    }
    
    // === Méthodes utilitaires ===
    
    fn get_script_for_app(&self, app_name: &str) -> Result<String, AWCSError> {
        // Trouver le script correspondant
        for (script_app, script) in &self.scripts {
            if app_name.contains(script_app) {
                return Ok(script.clone());
            }
        }
        
        Err(AWCSError::UnsupportedApp(format!("No script available for: {}", app_name)))
    }
    
    fn parse_script_result(&self, result: &str, app_name: &str) -> Result<AppleScriptResult, AWCSError> {
        let parts: Vec<&str> = result.trim().split("|SEPARATOR|").collect();
        
        if parts.len() < 4 {
            return Err(AWCSError::ExtractionFailed("Invalid AppleScript result format".to_string()));
        }
        
        // Gestion des cas d'erreur
        if parts[0] == "ERROR" {
            let error_msg = if parts.len() > 4 { parts[4] } else { "Unknown error" };
            return Err(AWCSError::ScriptFailed(format!("AppleScript error: {}", error_msg)));
        }
        
        if parts[0] == "NO_DOCUMENT" {
            return Err(AWCSError::ExtractionFailed("No active document found".to_string()));
        }
        
        let word_count: usize = parts[2].parse().unwrap_or(0);
        let is_protected: bool = parts[3] == "true";
        
        // Déterminer le type de document
        let doc_type = if app_name.contains("Word") {
            "docx".to_string()
        } else if app_name.contains("Excel") {
            "xlsx".to_string()
        } else if app_name.contains("PowerPoint") {
            "pptx".to_string()
        } else if app_name.contains("Pages") {
            "pages".to_string()
        } else if app_name.contains("Numbers") {
            "numbers".to_string()
        } else if app_name.contains("Preview") {
            "pdf".to_string()
        } else {
            "text".to_string()
        };
        
        Ok(AppleScriptResult {
            text: parts[0].to_string(),
            doc_type,
            path: if parts[1].is_empty() { None } else { Some(parts[1].to_string()) },
            word_count,
            is_protected,
        })
    }
    
    /// Teste l'extraction pour une application
    pub async fn test_extraction(&self, window: &WindowInfo) -> Result<bool, AWCSError> {
        match self.extract_from_app(window).await {
            Ok(result) => {
                tracing::info!(
                    "Test extraction successful: {} words from {} ({})",
                    result.word_count,
                    window.app,
                    result.doc_type
                );
                Ok(!result.text.is_empty() && result.word_count > 0)
            },
            Err(e) => {
                tracing::warn!("Test extraction failed for {}: {}", window.app, e);
                Ok(false)
            }
        }
    }
    
    /// Ajoute un script personnalisé pour une application
    pub fn add_custom_script(&mut self, app_name: String, script: String) {
        self.scripts.insert(app_name.clone(), script);
        if !self.supported_apps.contains(&app_name) {
            self.supported_apps.push(app_name);
        }
    }
    
    /// Récupère la liste des applications supportées
    pub fn get_supported_apps(&self) -> &[String] {
        &self.supported_apps
    }
    
    /// Vérifie si AppleScript est disponible
    pub async fn check_applescript_availability(&self) -> Result<bool, AWCSError> {
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg("return \"AppleScript available\"")
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("AppleScript check failed: {}", e)))?;
        
        Ok(output.status.success())
    }
}

impl Default for AppleScriptExtractor {
    fn default() -> Self {
        Self::new()
    }
}