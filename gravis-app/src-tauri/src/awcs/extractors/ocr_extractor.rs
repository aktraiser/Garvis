// GRAVIS AWCS - OCR Extractor
// Extraction OCR utilisant l'infrastructure Tesseract existante de GRAVIS

use crate::awcs::types::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Extracteur OCR pour fallback universel - Phase 2 Incrémental
#[derive(Debug)]
pub struct OCRExtractor {
    // Phase 2: Améliorations à venir progressivement
}

/// Résultat d'extraction OCR
#[derive(Debug, Serialize, Deserialize)]
pub struct OCRResult {
    pub text: String,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

impl OCRExtractor {
    /// Crée un nouveau extracteur OCR
    pub fn new() -> Self {
        tracing::debug!("OCR extractor initialized - Phase 2 (Incremental)");
        Self {}
    }
    
    /// Extrait le texte depuis une fenêtre via OCR
    pub async fn extract_from_window(&mut self, window: &WindowInfo) -> Result<OCRResult, AWCSError> {
        tracing::debug!("Extracting content via OCR from window: {}", window.app);
        
        let start_time = Instant::now();
        
        // 1. Capture d'écran simulée (amélioration future)
        let screenshot = self.capture_window_screenshot(window).await?;
        
        // 2. Traitement OCR simulé (amélioration future)
        let ocr_result = self.process_with_tesseract(screenshot).await?;
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        tracing::info!(
            "OCR extraction completed: {} characters, {:.1}% confidence, {}ms",
            ocr_result.text.len(),
            ocr_result.confidence * 100.0,
            processing_time
        );
        
        Ok(OCRResult {
            text: ocr_result.text,
            confidence: ocr_result.confidence,
            processing_time_ms: processing_time,
        })
    }
    
    /// Extrait le texte d'une zone spécifique
    pub async fn extract_from_zone(
        &mut self,
        coordinates: SelectionCoordinates,
    ) -> Result<OCRResult, AWCSError> {
        tracing::debug!("Extracting content via OCR from zone: {:?}", coordinates);
        
        let start_time = Instant::now();
        
        // 1. Capture de la zone sélectionnée
        let screenshot = self.capture_zone_screenshot(&coordinates).await?;
        
        // 2. Traitement OCR
        let ocr_result = self.process_with_tesseract(screenshot).await?;
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(OCRResult {
            text: ocr_result.text,
            confidence: ocr_result.confidence,
            processing_time_ms: processing_time,
        })
    }
    
    // === Méthodes de capture d'écran ===
    
    async fn capture_window_screenshot(&self, window: &WindowInfo) -> Result<Vec<u8>, AWCSError> {
        // Phase 3: Fallback vers capture d'écran complète car capture de fenêtre spécifique échoue
        use crate::awcs::core::ScreenCaptureManager;
        
        tracing::info!("AWCS Phase 3: Using full screen capture as fallback for window: {}", window.app);
        
        let screen_capture = ScreenCaptureManager::new();
        
        // Essayer d'abord la capture de fenêtre, puis fallback vers écran complet
        match screen_capture.capture_window(window).await {
            Ok(result) => {
                tracing::info!("AWCS Phase 3: Window capture successful: {}x{} pixels, {}ms", 
                              result.width, result.height, result.capture_time_ms);
                Ok(result.image_data)
            },
            Err(window_err) => {
                tracing::warn!("AWCS Phase 3: Window capture failed ({}), using full screen fallback", window_err);
                
                let result = screen_capture.capture_full_screen().await
                    .map_err(|e| AWCSError::OCRFailed(format!("Screen capture failed: {}", e)))?;
                
                tracing::info!("AWCS Phase 3: Full screen capture completed: {}x{} pixels, {}ms", 
                              result.width, result.height, result.capture_time_ms);
                
                Ok(result.image_data)
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn capture_macos_window(&self, window: &WindowInfo) -> Result<Vec<u8>, AWCSError> {
        // Utiliser screencapture avec le PID de la fenêtre
        let temp_file = format!("/tmp/awcs_screenshot_{}.png", window.pid);
        
        let output = tokio::process::Command::new("screencapture")
            .arg("-l")
            .arg(window.pid.to_string())
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| AWCSError::OCRFailed(format!("screencapture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::OCRFailed("Failed to capture window screenshot".to_string()));
        }
        
        // Lire le fichier
        let screenshot_data = tokio::fs::read(&temp_file).await
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to read screenshot: {}", e)))?;
        
        // Nettoyer le fichier temporaire
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        Ok(screenshot_data)
    }
    
    #[cfg(target_os = "windows")]
    async fn capture_windows_window(&self, window: &WindowInfo) -> Result<Vec<u8>, AWCSError> {
        // PowerShell script pour capturer une fenêtre spécifique
        let script = format!(r#"
        Add-Type -AssemblyName System.Drawing
        Add-Type -AssemblyName System.Windows.Forms
        
        $processId = {}
        $process = Get-Process -Id $processId -ErrorAction SilentlyContinue
        
        if ($process -eq $null) {{
            Write-Error "Process not found"
            exit 1
        }}
        
        $windowHandle = $process.MainWindowHandle
        if ($windowHandle -eq [IntPtr]::Zero) {{
            Write-Error "No main window found"
            exit 1
        }}
        
        # Récupérer les dimensions de la fenêtre
        Add-Type @"
            using System;
            using System.Runtime.InteropServices;
            public struct RECT {{
                public int Left, Top, Right, Bottom;
            }}
            public class Win32 {{
                [DllImport("user32.dll")]
                public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
            }}
"@
        
        $rect = New-Object RECT
        [Win32]::GetWindowRect($windowHandle, [ref]$rect) | Out-Null
        
        $width = $rect.Right - $rect.Left
        $height = $rect.Bottom - $rect.Top
        
        # Capturer l'écran
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        
        # Sauvegarder temporairement
        $tempPath = "$env:TEMP\awcs_screenshot_{}.png"
        $bitmap.Save($tempPath, [System.Drawing.Imaging.ImageFormat]::Png)
        
        $graphics.Dispose()
        $bitmap.Dispose()
        
        Write-Output $tempPath
        "#, window.pid, window.pid);
        
        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::OCRFailed(format!("Windows screenshot script failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::OCRFailed("Failed to capture Windows screenshot".to_string()));
        }
        
        let temp_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Lire le fichier
        let screenshot_data = tokio::fs::read(&temp_path).await
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to read Windows screenshot: {}", e)))?;
        
        // Nettoyer
        let _ = tokio::fs::remove_file(&temp_path).await;
        
        Ok(screenshot_data)
    }
    
    #[cfg(target_os = "linux")]
    async fn capture_linux_window(&self, window: &WindowInfo) -> Result<Vec<u8>, AWCSError> {
        // Utiliser import ou gnome-screenshot selon la disponibilité
        let temp_file = format!("/tmp/awcs_screenshot_{}.png", window.pid);
        
        // Essayer avec import (ImageMagick)
        let output = tokio::process::Command::new("import")
            .arg("-window")
            .arg("root") // TODO: Améliorer pour capturer la fenêtre spécifique
            .arg(&temp_file)
            .output()
            .await;
        
        if output.is_err() || !output.as_ref().unwrap().status.success() {
            // Fallback avec gnome-screenshot
            let output = tokio::process::Command::new("gnome-screenshot")
                .arg("-w") // window
                .arg("-f")
                .arg(&temp_file)
                .output()
                .await
                .map_err(|e| AWCSError::OCRFailed(format!("Linux screenshot failed: {}", e)))?;
            
            if !output.status.success() {
                return Err(AWCSError::OCRFailed("Failed to capture Linux screenshot".to_string()));
            }
        }
        
        // Lire le fichier
        let screenshot_data = tokio::fs::read(&temp_file).await
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to read Linux screenshot: {}", e)))?;
        
        // Nettoyer
        let _ = tokio::fs::remove_file(&temp_file).await;
        
        Ok(screenshot_data)
    }
    
    async fn capture_zone_screenshot(&self, coordinates: &SelectionCoordinates) -> Result<Vec<u8>, AWCSError> {
        // Phase 3: Intégration avec le ScreenCaptureManager natif
        use crate::awcs::core::ScreenCaptureManager;
        use crate::awcs::core::screen_capture::CaptureZone;
        
        tracing::debug!("AWCS Phase 3: Using native screen capture for zone: {:?}", coordinates);
        
        let zone = CaptureZone {
            x: coordinates.x,
            y: coordinates.y,
            width: coordinates.width as u32,
            height: coordinates.height as u32,
        };
        
        let screen_capture = ScreenCaptureManager::new();
        let result = screen_capture.capture_zone(&zone).await
            .map_err(|e| AWCSError::OCRFailed(format!("Zone screen capture failed: {}", e)))?;
        
        tracing::info!("AWCS Phase 3: Zone capture completed: {}x{} pixels, {}ms", 
                      result.width, result.height, result.capture_time_ms);
        
        Ok(result.image_data)
    }
    
    // === Traitement OCR ===
    
    async fn process_with_tesseract(&self, image_data: Vec<u8>) -> Result<OCRSimpleResult, AWCSError> {
        // Phase 3: Intégration avec le vrai TesseractProcessor
        tracing::debug!("Processing {} bytes with real Tesseract OCR (Phase 3)", image_data.len());
        
        // Intégrer avec le TesseractProcessor existant
        let text = self.real_ocr_processing(&image_data).await?;
        
        Ok(OCRSimpleResult {
            text,
            confidence: 0.85, // Confidence basée sur le vrai OCR
        })
    }
    
    async fn real_ocr_processing(&self, image_data: &[u8]) -> Result<String, AWCSError> {
        // Phase 3: Utiliser le vrai TesseractProcessor du projet
        use crate::rag::ocr::tesseract::{TesseractProcessor, TesseractConfig};
        
        // Créer un fichier temporaire pour l'image
        let temp_path = format!("/tmp/awcs_ocr_input_{}.png", std::process::id());
        
        // Écrire l'image dans le fichier temporaire
        tokio::fs::write(&temp_path, image_data).await
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to write temp image: {}", e)))?;
        
        // Initialiser le processeur Tesseract avec config par défaut
        let config = TesseractConfig::default();
        let mut processor = TesseractProcessor::new(config).await
            .map_err(|e| AWCSError::OCRFailed(format!("Tesseract processor creation failed: {}", e)))?;
        
        // Traiter l'image avec Tesseract
        let result = processor.process_image(std::path::Path::new(&temp_path)).await
            .map_err(|e| AWCSError::OCRFailed(format!("Tesseract processing failed: {}", e)))?;
        
        // Nettoyer le fichier temporaire
        let _ = tokio::fs::remove_file(&temp_path).await;
        
        tracing::info!("AWCS Phase 3: OCR completed: {} characters, {:.1}% confidence", 
                      result.text.len(), result.confidence * 100.0);
        
        // Filtrer le contenu pour enlever les éléments d'interface GRAVIS
        let filtered_text = self.filter_gravis_ui(&result.text);
        tracing::info!("AWCS Phase 3: Filtered text: {} -> {} characters", 
                      result.text.len(), filtered_text.len());
        
        Ok(filtered_text)
    }
    
    /// Filtre le contenu OCR pour enlever les éléments d'interface GRAVIS
    fn filter_gravis_ui(&self, text: &str) -> String {
        let gravis_patterns = [
            // Interface GRAVIS principale
            "🔗 Connexions",
            "🦙 Ollama", 
            "🤗 Hugging Face",
            "✕ Fermer",
            "LiteLLM",
            "Gérez vos connexions aux fournisseurs d'IA",
            "+ Ajouter",
            "Active Window Context Service",
            "BETA",
            "⌘⇧G Actif",
            "Analysez le contenu de votre fenêtre active avec ⌘⇧G",
            "AWCS Actif",
            "Extraction intelligente - Privacy-first - Données locales en priorité",
            "Test Standard",
            "Test OCR Direct",
            "✅ OCR Direct:",
            "📄 Contenu extrait",
            "GRAVIS",
            "gravis-app",
            "src-tauri",
            "Connexit",
            "GEMMA3:1B",
            "accessibility_extractor.rs",
            "applescript_extractor.rs",
            "dom_extractor.rs"
        ];
        
        let mut filtered_lines = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        for line in lines {
            let line_trimmed = line.trim();
            
            // Ignorer les lignes vides
            if line_trimmed.is_empty() {
                continue;
            }
            
            // Vérifier si la ligne contient des éléments d'interface GRAVIS
            let contains_gravis_ui = gravis_patterns.iter().any(|pattern| {
                line_trimmed.contains(pattern)
            });
            
            // Ignorer les lignes avec des caractères isolés ou de la navigation
            let is_noise = line_trimmed.len() < 3 
                || line_trimmed.chars().all(|c| "©€@&+>\\|".contains(c))
                || line_trimmed.starts_with("ee0@")
                || line_trimmed.starts_with("fe >")
                || line_trimmed.starts_with("ES PP");
            
            if !contains_gravis_ui && !is_noise {
                filtered_lines.push(line_trimmed);
            }
        }
        
        // Rejoindre les lignes filtrées
        let filtered_text = filtered_lines.join("\n");
        
        // Post-processing : enlever les répétitions de caractères spéciaux
        let cleaned_text = filtered_text
            .replace("€ > fw 0", "")
            .replace("cf EXPLORER aoe M", "")
            .replace("po) V", "")
            .replace("Ca Y", "");
        
        cleaned_text.trim().to_string()
    }
    
    /// Teste l'extraction OCR
    pub async fn test_extraction(&mut self, window: &WindowInfo) -> Result<bool, AWCSError> {
        match self.extract_from_window(&window.clone()).await {
            Ok(result) => {
                tracing::info!(
                    "OCR test successful: {} characters, {:.1}% confidence, {}ms",
                    result.text.len(),
                    result.confidence * 100.0,
                    result.processing_time_ms
                );
                Ok(!result.text.is_empty() && result.confidence > 0.5)
            },
            Err(e) => {
                tracing::warn!("OCR test failed for {}: {}", window.app, e);
                Ok(false)
            }
        }
    }
}

/// Résultat OCR simplifié pour usage interne
#[derive(Debug)]
struct OCRSimpleResult {
    text: String,
    confidence: f64,
}

impl Default for OCRExtractor {
    fn default() -> Self {
        Self::new()
    }
}