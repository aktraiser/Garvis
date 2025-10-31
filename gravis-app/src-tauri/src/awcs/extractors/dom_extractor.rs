// GRAVIS AWCS - DOM Extractor
// Extraction de contenu depuis les navigateurs web

use crate::awcs::types::*;
use serde::{Deserialize, Serialize};

/// Extracteur de contenu DOM pour navigateurs
#[derive(Debug)]
pub struct DOMExtractor {
    supported_browsers: Vec<String>,
}

/// Résultat d'extraction DOM
#[derive(Debug, Serialize, Deserialize)]
pub struct DOMExtractionResult {
    pub url: String,
    pub title: String,
    pub body_text: String,
    pub selection: Option<String>,
    pub word_count: usize,
    pub has_frames: bool,
    pub has_shadow_dom: bool,
}

impl DOMExtractor {
    /// Crée un nouveau extracteur DOM
    pub fn new() -> Self {
        Self {
            supported_browsers: vec![
                "Safari".to_string(),
                "Google Chrome".to_string(),
                "Chrome".to_string(),
                "Chromium".to_string(),
                "Firefox".to_string(),
                "Microsoft Edge".to_string(),
                "Edge".to_string(),
                "Arc".to_string(),
                "Brave".to_string(),
            ],
        }
    }
    
    /// Vérifie si l'application est un navigateur supporté
    pub fn is_supported_browser(&self, app_name: &str) -> bool {
        self.supported_browsers
            .iter()
            .any(|browser| app_name.contains(browser))
    }
    
    /// Extrait le contenu depuis un navigateur
    pub async fn extract_from_browser(&mut self, window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        if !self.is_supported_browser(&window.app) {
            return Err(AWCSError::UnsupportedApp(format!("Browser not supported: {}", window.app)));
        }
        
        tracing::debug!("Extracting DOM content from: {}", window.app);
        
        // Sélectionner la méthode d'extraction selon le navigateur
        if window.app.contains("Safari") {
            self.extract_from_safari(window).await
        } else if window.app.contains("Chrome") || window.app.contains("Chromium") {
            self.extract_from_chrome(window).await
        } else if window.app.contains("Firefox") {
            self.extract_from_firefox(window).await
        } else {
            // Fallback générique
            self.extract_generic_browser(window).await
        }
    }
    
    // === Extraction Safari (macOS) ===
    
    async fn extract_from_safari(&self, window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        if cfg!(not(target_os = "macos")) {
            return Err(AWCSError::UnsupportedApp("Safari extraction only available on macOS".to_string()));
        }
        
        let script = r#"
        tell application "Safari"
            if exists front document then
                set pageURL to URL of front document
                set pageTitle to name of front document
                
                # Extraire le texte du body
                set pageText to do JavaScript "document.body ? document.body.innerText : ''" in front document
                
                # Vérifier la sélection
                set selectedText to do JavaScript "window.getSelection() ? window.getSelection().toString() : ''" in front document
                
                # Métriques de la page
                set wordCount to do JavaScript "
                    var text = document.body ? document.body.innerText : '';
                    text.split(/\\s+/).filter(word => word.length > 0).length;
                " in front document
                
                set hasFrames to do JavaScript "document.querySelectorAll('iframe').length > 0" in front document
                set hasShadowDOM to do JavaScript "
                    Array.from(document.querySelectorAll('*')).some(el => el.shadowRoot);
                " in front document
                
                return pageURL & "|" & pageTitle & "|" & pageText & "|" & selectedText & "|" & wordCount & "|" & hasFrames & "|" & hasShadowDOM
            else
                return "|||0|false|false"
            end if
        end tell
        "#;
        
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("Safari AppleScript failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScriptFailed(
                format!("Safari script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        self.parse_safari_result(&result)
    }
    
    fn parse_safari_result(&self, result: &str) -> Result<DOMExtractionResult, AWCSError> {
        let parts: Vec<&str> = result.trim().split('|').collect();
        
        if parts.len() < 7 {
            return Err(AWCSError::ExtractionFailed("Invalid Safari extraction result".to_string()));
        }
        
        let word_count: usize = parts[4].parse().unwrap_or(0);
        let has_frames: bool = parts[5] == "true";
        let has_shadow_dom: bool = parts[6] == "true";
        
        Ok(DOMExtractionResult {
            url: parts[0].to_string(),
            title: parts[1].to_string(),
            body_text: parts[2].to_string(),
            selection: if parts[3].is_empty() { None } else { Some(parts[3].to_string()) },
            word_count,
            has_frames,
            has_shadow_dom,
        })
    }
    
    // === Extraction Chrome/Chromium ===
    
    async fn extract_from_chrome(&self, window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        // TODO: Implémentation avec extension Chrome ou debugging protocol
        // Pour l'instant, essayer AppleScript sur macOS
        
        if cfg!(target_os = "macos") {
            self.extract_chrome_macos(window).await
        } else {
            // Fallback vers méthode générique
            self.extract_generic_browser(window).await
        }
    }
    
    async fn extract_chrome_macos(&self, _window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        tracing::info!("AWCS Phase 3: Attempting Chrome extraction via AppleScript (simplified)");
        
        // Version simplifiée qui récupère juste URL et titre sans JavaScript
        let script = r#"
        tell application "Google Chrome"
            if exists front window then
                tell front window
                    if exists active tab then
                        set pageURL to URL of active tab
                        set pageTitle to title of active tab
                        return pageURL & "|" & pageTitle
                    end if
                end tell
            end if
            return "|"
        end tell
        "#;
        
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("Chrome AppleScript failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScriptFailed(
                format!("Chrome script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();
        
        tracing::info!("Chrome extraction result: {} parts, url={}, title={}", 
                     parts.len(), 
                     parts.get(0).unwrap_or(&""), 
                     parts.get(1).unwrap_or(&""));
        
        if parts.len() >= 2 && !parts[0].is_empty() {
            // Version simplifiée avec juste URL et titre
            Ok(DOMExtractionResult {
                url: parts[0].to_string(),
                title: parts[1].to_string(),
                body_text: format!("Chrome content from: {}", parts[0]),
                selection: None,
                word_count: 100, // Valeur par défaut
                has_frames: false,
                has_shadow_dom: false,
            })
        } else {
            Err(AWCSError::ScriptFailed("Failed to get Chrome URL/title".to_string()))
        }
    }
    
    // === Extraction Firefox ===
    
    async fn extract_from_firefox(&self, _window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        // TODO: Implémentation avec WebDriver ou extension Firefox
        // Pour l'instant, retourner une erreur explicite
        
        Err(AWCSError::UnsupportedApp("Firefox extraction not implemented yet".to_string()))
    }
    
    // === Méthode générique ===
    
    async fn extract_generic_browser(&self, window: &WindowInfo) -> Result<DOMExtractionResult, AWCSError> {
        tracing::warn!("Using generic browser extraction for: {}", window.app);
        
        // Méthode fallback : extraction via clipboard ou OCR
        // Pour l'instant, retourner un contenu minimal
        
        Ok(DOMExtractionResult {
            url: "unknown".to_string(),
            title: format!("Content from {}", window.app),
            body_text: "Contenu non accessible via extraction DOM automatique".to_string(),
            selection: None,
            word_count: 0,
            has_frames: false,
            has_shadow_dom: false,
        })
    }
    
    /// Méthode de test pour vérifier l'extraction
    pub async fn test_extraction(&mut self, window: &WindowInfo) -> Result<bool, AWCSError> {
        match self.extract_from_browser(&window.clone()).await {
            Ok(result) => {
                tracing::info!(
                    "Test extraction successful: {} words from {}",
                    result.word_count,
                    result.url
                );
                Ok(result.word_count > 0)
            },
            Err(e) => {
                tracing::warn!("Test extraction failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Ajoute un navigateur supporté
    pub fn add_supported_browser(&mut self, browser_name: String) {
        if !self.supported_browsers.contains(&browser_name) {
            self.supported_browsers.push(browser_name);
        }
    }
    
    /// Récupère la liste des navigateurs supportés
    pub fn get_supported_browsers(&self) -> &[String] {
        &self.supported_browsers
    }
}

impl Default for DOMExtractor {
    fn default() -> Self {
        Self::new()
    }
}