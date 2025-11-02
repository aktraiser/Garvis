// GRAVIS AWCS - Context Extractor
// Extraction intelligente du contexte avec fallbacks hiérarchiques

use crate::awcs::types::*;
use crate::awcs::extractors::{
    window_detector::WindowDetector,
    dom_extractor::DOMExtractor,
    applescript_extractor::AppleScriptExtractor,
    accessibility_extractor::AccessibilityExtractor,
    ocr_extractor::OCRExtractor,
};
use std::time::Duration;
use tokio::time::timeout;

/// Extracteur de contexte avec stratégies multiples
#[derive(Debug)]
pub struct ContextExtractor {
    window_detector: WindowDetector,
    dom_extractor: DOMExtractor,
    applescript_extractor: AppleScriptExtractor,
    accessibility_extractor: AccessibilityExtractor,
    ocr_extractor: OCRExtractor,
    extraction_timeout: Duration,
}

impl ContextExtractor {
    /// Crée un nouveau extracteur de contexte
    pub fn new() -> Self {
        Self {
            window_detector: WindowDetector::new(),
            dom_extractor: DOMExtractor::new(),
            applescript_extractor: AppleScriptExtractor::new(),
            accessibility_extractor: AccessibilityExtractor::new(),
            ocr_extractor: OCRExtractor::new(),
            extraction_timeout: Duration::from_millis(800),
        }
    }
    
    /// Extrait le contexte de la fenêtre active avec fallbacks
    pub async fn extract_current_window_context(&mut self) -> Result<ContextEnvelope, AWCSError> {
        tracing::debug!("Starting context extraction with hierarchical fallbacks");
        
        // 1. Détection de la fenêtre active
        let window_info = self.window_detector.get_current_window().await?;
        
        tracing::info!(
            "AWCS Phase 3 - Window: app='{}', title='{}', pid={}, bundle_id={:?}",
            window_info.app,
            window_info.title,
            window_info.pid,
            window_info.bundle_id
        );
        
        // 2. Tentatives d'extraction avec fallbacks hiérarchiques
        
        // Extension Browser check (Phase 1 - Priority)
        if self.is_browser_app(&window_info.app) {
            tracing::info!("AWCS Phase 3 - Browser detected: {}, checking extension server", window_info.app);
            match self.try_extension_extraction(&window_info).await {
                Ok(context) => {
                    if context.confidence.text_completeness > 0.7 {
                        tracing::info!("Extraction successful with browser extension: {:.1}% completeness", context.confidence.text_completeness * 100.0);
                        return Ok(context);
                    }
                },
                Err(e) => tracing::info!("Extension extraction failed: {}", e),
            }
        }
        
        // DOM extraction
        tracing::info!("AWCS Phase 3 - Attempting extraction with method: DOM");
        match timeout(self.extraction_timeout, self.try_dom_extraction(&window_info)).await {
            Ok(Ok(context)) => {
                if context.confidence.text_completeness > 0.5 {
                    tracing::info!("Extraction successful with dom: {:.1}% completeness", context.confidence.text_completeness * 100.0);
                    return Ok(context);
                }
                tracing::debug!("DOM extraction had low confidence, trying next method");
            },
            Ok(Err(e)) => tracing::debug!("DOM extraction failed: {}", e),
            Err(_) => tracing::debug!("DOM extraction timed out"),
        }
        
        // AppleScript extraction
        tracing::info!("AWCS Phase 3 - Attempting extraction with method: AppleScript");
        match timeout(self.extraction_timeout, self.try_applescript_extraction(&window_info)).await {
            Ok(Ok(context)) => {
                if context.confidence.text_completeness > 0.5 {
                    tracing::info!("Extraction successful with applescript: {:.1}% completeness", context.confidence.text_completeness * 100.0);
                    return Ok(context);
                }
                tracing::debug!("AppleScript extraction had low confidence, trying next method");
            },
            Ok(Err(e)) => tracing::info!("AppleScript extraction failed: {}", e),
            Err(_) => tracing::warn!("AppleScript extraction timed out"),
        }
        
        // Accessibility extraction
        tracing::info!("AWCS Phase 3 - Attempting extraction with method: Accessibility");
        match timeout(self.extraction_timeout, self.try_accessibility_extraction(&window_info)).await {
            Ok(Ok(context)) => {
                if context.confidence.text_completeness > 0.5 {
                    tracing::info!("Extraction successful with accessibility: {:.1}% completeness", context.confidence.text_completeness * 100.0);
                    return Ok(context);
                }
                tracing::debug!("Accessibility extraction had low confidence, trying next method");
            },
            Ok(Err(e)) => tracing::info!("Accessibility extraction failed: {}", e),
            Err(_) => tracing::warn!("Accessibility extraction timed out"),
        }
        
        // OCR extraction (last resort) - Timeout plus long pour l'OCR
        tracing::info!("AWCS Phase 3 - Attempting extraction with method: OCR (last resort)");
        let ocr_timeout = Duration::from_secs(10); // 10 secondes pour l'OCR
        match timeout(ocr_timeout, self.try_ocr_extraction(&window_info)).await {
            Ok(Ok(context)) => {
                if context.confidence.text_completeness > 0.5 {
                    tracing::info!("Extraction successful with ocr: {:.1}% completeness", context.confidence.text_completeness * 100.0);
                    return Ok(context);
                } else {
                    tracing::debug!("OCR extraction had low confidence: {:.1}%", context.confidence.text_completeness * 100.0);
                }
            },
            Ok(Err(e)) => tracing::error!("OCR extraction failed: {}", e),
            Err(_) => tracing::error!("OCR extraction timed out after {} seconds", ocr_timeout.as_secs()),
        }
        
        // Si tous les fallbacks échouent, retourner une enveloppe minimale
        tracing::warn!("All extraction methods failed, returning minimal context");
        
        Ok(ContextEnvelope {
            source: window_info,
            document: None,
            content: ContentData {
                selection: None,
                fulltext: Some("Contenu non accessible".to_string()),
                metadata: None,
            },
            confidence: ExtractionConfidence {
                text_completeness: 0.1,
                source_reliability: 0.5,
                extraction_method: "fallback".to_string(),
            },
            timestamp: chrono::Utc::now(),
            security_flags: None,
        })
    }
    
    // === Méthodes d'extraction spécialisées ===
    
    /// Tentative d'extraction DOM (navigateurs)
    async fn try_dom_extraction(&mut self, window: &WindowInfo) -> Result<ContextEnvelope, AWCSError> {
        if !self.is_web_browser(&window.app) {
            return Err(AWCSError::UnsupportedApp("Not a web browser".to_string()));
        }
        
        let content = self.dom_extractor.extract_from_browser(window).await?;
        
        Ok(ContextEnvelope {
            source: window.clone(),
            document: Some(DocumentInfo {
                doc_type: "web".to_string(),
                path: None,
                url: Some(content.url.clone()),
            }),
            content: ContentData {
                selection: content.selection,
                fulltext: Some(content.body_text),
                metadata: Some(serde_json::json!({
                    "url": content.url,
                    "title": content.title,
                    "word_count": content.word_count,
                    "has_frames": content.has_frames,
                    "has_shadow_dom": content.has_shadow_dom
                })),
            },
            confidence: ExtractionConfidence {
                text_completeness: if content.word_count > 100 { 0.95 } else { 0.7 },
                source_reliability: 0.9,
                extraction_method: "dom".to_string(),
            },
            timestamp: chrono::Utc::now(),
            security_flags: None,
        })
    }
    
    /// Tentative d'extraction AppleScript (macOS apps)
    async fn try_applescript_extraction(&mut self, window: &WindowInfo) -> Result<ContextEnvelope, AWCSError> {
        if cfg!(not(target_os = "macos")) {
            return Err(AWCSError::UnsupportedApp("AppleScript not available on this platform".to_string()));
        }
        
        let content = self.applescript_extractor.extract_from_app(window).await?;
        
        Ok(ContextEnvelope {
            source: window.clone(),
            document: Some(DocumentInfo {
                doc_type: content.doc_type,
                path: content.path,
                url: None,
            }),
            content: ContentData {
                selection: None,
                fulltext: Some(content.text),
                metadata: Some(serde_json::json!({
                    "word_count": content.word_count,
                    "is_protected": content.is_protected
                })),
            },
            confidence: ExtractionConfidence {
                text_completeness: if content.word_count > 50 { 0.9 } else { 0.6 },
                source_reliability: 0.95,
                extraction_method: "applescript".to_string(),
            },
            timestamp: chrono::Utc::now(),
            security_flags: None,
        })
    }
    
    /// Tentative d'extraction Accessibility (multi-platform)
    async fn try_accessibility_extraction(&mut self, window: &WindowInfo) -> Result<ContextEnvelope, AWCSError> {
        let content = self.accessibility_extractor.extract_text_elements(window).await?;
        
        Ok(ContextEnvelope {
            source: window.clone(),
            document: None,
            content: ContentData {
                selection: None,
                fulltext: Some(content.combined_text),
                metadata: Some(serde_json::json!({
                    "elements_found": content.elements_count,
                    "roles_detected": content.roles_detected
                })),
            },
            confidence: ExtractionConfidence {
                text_completeness: if content.elements_count > 5 { 0.8 } else { 0.5 },
                source_reliability: 0.7,
                extraction_method: "accessibility".to_string(),
            },
            timestamp: chrono::Utc::now(),
            security_flags: None,
        })
    }
    
    /// Tentative d'extraction OCR (fallback universel)
    async fn try_ocr_extraction(&mut self, window: &WindowInfo) -> Result<ContextEnvelope, AWCSError> {
        let content = self.ocr_extractor.extract_from_window(window).await?;
        
        Ok(ContextEnvelope {
            source: window.clone(),
            document: None,
            content: ContentData {
                selection: None,
                fulltext: Some(content.text),
                metadata: Some(serde_json::json!({
                    "confidence": content.confidence,
                    "processing_time": content.processing_time_ms
                })),
            },
            confidence: ExtractionConfidence {
                text_completeness: content.confidence,
                source_reliability: 0.7,
                extraction_method: "ocr".to_string(),
            },
            timestamp: chrono::Utc::now(),
            security_flags: None,
        })
    }
    
    // === Méthodes utilitaires ===
    
    /// Vérifie si l'application est un navigateur web
    fn is_web_browser(&self, app_name: &str) -> bool {
        let browsers = ["Safari", "Google Chrome", "Chrome", "Firefox", "Microsoft Edge", "Edge", "Chromium", "Arc", "Brave"];
        let is_browser = browsers.iter().any(|&browser| app_name.contains(browser));
        tracing::info!("AWCS Phase 3 - Browser check: app='{}' => is_browser={}", app_name, is_browser);
        is_browser
    }
    
    /// Vérifie si l'application est un navigateur (alias pour compatibilité)
    fn is_browser_app(&self, app_name: &str) -> bool {
        self.is_web_browser(app_name)
    }
    
    /// Tente l'extraction via l'extension browser
    async fn try_extension_extraction(&self, window: &WindowInfo) -> Result<ContextEnvelope, AWCSError> {
        // Déclencher l'extraction via l'extension browser
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:8766/api/extension/trigger")
            .json(&serde_json::json!({
                "action": "extract_current_tab",
                "source": "awcs_shortcut"
            }))
            .timeout(std::time::Duration::from_millis(1500))
            .send()
            .await
            .map_err(|e| AWCSError::ExtractionFailed(format!("Extension server unreachable: {}", e)))?;
            
        if response.status().is_success() {
            // Attendre un peu que l'extension traite
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Créer une réponse contextuelle optimiste
            Ok(ContextEnvelope {
                source: window.clone(),
                document: None,
                content: ContentData {
                    selection: None,
                    fulltext: Some("Content extraction triggered via browser extension. Check GRAVIS chat for results.".to_string()),
                    metadata: None,
                },
                confidence: ExtractionConfidence {
                    text_completeness: 0.8, // Haute confiance pour l'extension
                    source_reliability: 0.9,
                    extraction_method: "browser_extension".to_string(),
                },
                timestamp: chrono::Utc::now(),
                security_flags: None,
            })
        } else {
            Err(AWCSError::ExtractionFailed("Extension server returned error".to_string()))
        }
    }
    
    /// Vérifie si l'application est supportée par AppleScript
    fn is_applescript_supported(&self, app_name: &str) -> bool {
        let supported_apps = ["Microsoft Word", "Microsoft Excel", "Microsoft PowerPoint", "Pages", "Numbers", "Keynote"];
        supported_apps.iter().any(|&app| app_name.contains(app))
    }
    
    /// Configure le timeout d'extraction
    pub fn set_extraction_timeout(&mut self, timeout: Duration) {
        self.extraction_timeout = timeout;
    }
    
    /// Force l'extraction OCR directement (mode universel)
    pub async fn extract_with_ocr_direct(&mut self) -> Result<ContextEnvelope, AWCSError> {
        tracing::info!("AWCS Phase 3 - Force OCR mode for universal extraction");
        
        // 1. Détection de la fenêtre active
        let window_info = self.window_detector.get_current_window().await?;
        
        tracing::info!(
            "AWCS Phase 3 - Direct OCR: app='{}', title='{}', pid={}",
            window_info.app,
            window_info.title,
            window_info.pid
        );
        
        // 2. Extraction OCR directe
        match self.try_ocr_extraction(&window_info).await {
            Ok(context) => {
                tracing::info!("Direct OCR extraction successful: {:.1}% confidence", context.confidence.text_completeness * 100.0);
                Ok(context)
            },
            Err(e) => {
                tracing::error!("Direct OCR extraction failed: {}", e);
                
                // Fallback minimal si même l'OCR échoue
                Ok(ContextEnvelope {
                    source: window_info,
                    document: None,
                    content: ContentData {
                        selection: None,
                        fulltext: Some("Extraction OCR échouée - contenu non accessible".to_string()),
                        metadata: None,
                    },
                    confidence: ExtractionConfidence {
                        text_completeness: 0.1,
                        source_reliability: 0.3,
                        extraction_method: "ocr_failed".to_string(),
                    },
                    timestamp: chrono::Utc::now(),
                    security_flags: None,
                })
            }
        }
    }
}

impl Default for ContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}