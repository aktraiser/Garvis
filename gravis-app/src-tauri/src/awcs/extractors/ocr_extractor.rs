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
        
        // Essayer Tesseract d'abord (méthode principale)
        match self.real_ocr_processing(&image_data).await {
            Ok(text) => {
                let confidence = self.calculate_text_confidence(&text);
                
                // Si la confiance est faible, essayer le transformer OCR en fallback
                if confidence < 0.6 {
                    tracing::info!("AWCS Phase 3: Tesseract confidence low ({:.2}), trying transformer OCR fallback", confidence);
                    match self.try_transformer_ocr(&image_data).await {
                        Ok(transformer_result) => {
                            if transformer_result.confidence > confidence {
                                tracing::info!("AWCS Phase 3: Transformer OCR performed better ({:.2} vs {:.2})", 
                                              transformer_result.confidence, confidence);
                                return Ok(transformer_result);
                            }
                        },
                        Err(e) => {
                            tracing::debug!("AWCS Phase 3: Transformer OCR fallback failed: {}", e);
                        }
                    }
                }
                
                Ok(OCRSimpleResult {
                    text,
                    confidence,
                })
            },
            Err(e) => {
                tracing::warn!("AWCS Phase 3: Tesseract failed, trying transformer OCR: {}", e);
                // Fallback complet vers transformer
                self.try_transformer_ocr(&image_data).await
            }
        }
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
    
    /// NOUVELLE MÉTHODE : Extraction OCR avec capture de fenêtre focalisée (amélioration Phase 3)
    pub async fn extract_from_focused_window(&mut self, window: &WindowInfo) -> Result<OCRResult, AWCSError> {
        tracing::info!("AWCS Phase 3: Extracting via focused window OCR for: {}", window.app);
        
        let start_time = Instant::now();
        
        // 1. Capture de la fenêtre focalisée uniquement (nouvelle méthode)
        let screenshot = self.capture_focused_window_screenshot(window).await?;
        
        // 2. Traitement OCR avec l'infrastructure Tesseract existante (préservée)
        let ocr_result = self.process_with_tesseract(screenshot).await?;
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 3. Filtrage intelligent (nouveau)
        let filtered_text = self.filter_gravis_ui(&ocr_result.text);
        
        tracing::info!("AWCS Phase 3: Focused OCR completed: {} -> {} characters, {:.1}% confidence, {}ms", 
                      ocr_result.text.len(), filtered_text.len(), ocr_result.confidence * 100.0, processing_time);
        
        Ok(OCRResult {
            text: filtered_text,
            confidence: ocr_result.confidence,
            processing_time_ms: processing_time,
        })
    }
    
    /// Capture de fenêtre focalisée via API d'accessibilité (nouvelle méthode)
    async fn capture_focused_window_screenshot(&self, window: &WindowInfo) -> Result<Vec<u8>, AWCSError> {
        tracing::info!("AWCS Phase 3: Attempting focused window capture for: {}", window.app);
        
        // 1. Essayer la capture de fenêtre focalisée via API d'accessibilité
        match self.capture_focused_window_bounds().await {
            Ok((x, y, width, height)) => {
                tracing::info!("AWCS Phase 3: Got window bounds: x={}, y={}, w={}, h={}", x, y, width, height);
                
                // Capture d'écran complète puis crop
                let full_screenshot = self.capture_full_screen_for_crop().await?;
                
                // Crop vers la région de la fenêtre active
                let cropped_image = self.crop_image_to_bounds(full_screenshot, x, y, width, height)?;
                
                tracing::info!("AWCS Phase 3: Focused window capture successful: {}x{} pixels", width, height);
                Ok(cropped_image)
            },
            Err(bounds_err) => {
                tracing::warn!("AWCS Phase 3: Window bounds detection failed ({}), falling back to existing method", bounds_err);
                
                // Fallback vers la méthode existante (préservée)
                self.capture_window_screenshot(window).await
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn capture_focused_window_bounds(&self) -> Result<(i32, i32, i32, i32), AWCSError> {
        use std::process::Command;
        
        tracing::debug!("AWCS Phase 3: Getting focused window bounds via Accessibility API");
        
        let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            try
                tell application process frontApp
                    set frontWindow to front window
                    set {x, y} to position of frontWindow
                    set {w, h} to size of frontWindow
                    return x & "," & y & "," & w & "," & h
                end tell
            on error
                return "error"
            end try
        end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AWCSError::OCRFailed(format!("AppleScript bounds failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::OCRFailed(
                format!("Bounds script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result_raw = String::from_utf8_lossy(&output.stdout);
        let result = result_raw.trim();
        
        if result == "error" {
            return Err(AWCSError::OCRFailed("Could not get window bounds".to_string()));
        }
        
        let coords: Vec<&str> = result.split(',').collect();
        if coords.len() != 4 {
            return Err(AWCSError::OCRFailed("Invalid bounds format".to_string()));
        }
        
        let x: i32 = coords[0].parse().map_err(|_| AWCSError::OCRFailed("Invalid x coordinate".to_string()))?;
        let y: i32 = coords[1].parse().map_err(|_| AWCSError::OCRFailed("Invalid y coordinate".to_string()))?;
        let w: i32 = coords[2].parse().map_err(|_| AWCSError::OCRFailed("Invalid width".to_string()))?;
        let h: i32 = coords[3].parse().map_err(|_| AWCSError::OCRFailed("Invalid height".to_string()))?;
        
        Ok((x, y, w, h))
    }
    
    #[cfg(not(target_os = "macos"))]
    async fn capture_focused_window_bounds(&self) -> Result<(i32, i32, i32, i32), AWCSError> {
        Err(AWCSError::OCRFailed("Focused window bounds only available on macOS".to_string()))
    }
    
    async fn capture_full_screen_for_crop(&self) -> Result<Vec<u8>, AWCSError> {
        use crate::awcs::core::ScreenCaptureManager;
        
        let screen_capture = ScreenCaptureManager::new();
        let result = screen_capture.capture_full_screen().await
            .map_err(|e| AWCSError::OCRFailed(format!("Full screen capture failed: {}", e)))?;
        
        Ok(result.image_data)
    }
    
    fn crop_image_to_bounds(&self, image_data: Vec<u8>, x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>, AWCSError> {
        use image::ImageFormat;
        
        // Charger l'image complète
        let img = image::load_from_memory(&image_data)
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to load image: {}", e)))?;
        
        // Valider les bounds
        let img_width = img.width() as i32;
        let img_height = img.height() as i32;
        
        let crop_x = std::cmp::max(0, x) as u32;
        let crop_y = std::cmp::max(0, y) as u32;
        let crop_width = std::cmp::min(width, img_width - x) as u32;
        let crop_height = std::cmp::min(height, img_height - y) as u32;
        
        if crop_width == 0 || crop_height == 0 {
            return Err(AWCSError::OCRFailed("Invalid crop dimensions".to_string()));
        }
        
        // Crop l'image
        let mut cropped = img.crop_imm(crop_x, crop_y, crop_width, crop_height);
        
        // Phase 3: Détection et crop vers la zone de contenu principal
        cropped = self.detect_content_area(cropped)?;
        
        // Phase 3: Preprocessing intelligent pour améliorer l'OCR
        cropped = self.preprocess_for_ocr(cropped)?;
        
        // Convertir en bytes PNG
        let mut output = Vec::new();
        cropped.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)
            .map_err(|e| AWCSError::OCRFailed(format!("Failed to encode cropped image: {}", e)))?;
        
        tracing::info!("AWCS Phase 3: Image cropped from {}x{} to {}x{}", 
                      img_width, img_height, cropped.width(), cropped.height());
        
        Ok(output)
    }
    
    /// Détecte et crop vers la zone de contenu principale (éviter menubars, sidebars)
    fn detect_content_area(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        let width = image.width();
        let height = image.height();
        
        // Stratégie conservative : exclure les zones typiques des UI
        // Menubar macOS : généralement 25-30px en haut
        let menubar_height = std::cmp::min(30, height / 20); // Maximum 5% de la hauteur
        
        // Sidebars : généralement 200-300px sur les côtés pour les apps modernes
        let sidebar_width = std::cmp::min(250, width / 6); // Maximum 16% de la largeur
        
        // Dock/taskbar : généralement 60-80px en bas
        let taskbar_height = std::cmp::min(80, height / 15); // Maximum 6% de la hauteur
        
        // Calculer la zone de contenu central
        let content_x = sidebar_width;
        let content_y = menubar_height;
        let content_width = width.saturating_sub(sidebar_width * 2); // Éviter les deux côtés
        let content_height = height.saturating_sub(menubar_height + taskbar_height);
        
        // Validation : s'assurer qu'on garde au moins 50% de l'image
        if content_width < width / 2 || content_height < height / 2 {
            tracing::debug!("AWCS Phase 3: Content area too small, keeping full window");
            return Ok(image);
        }
        
        let content_area = image.crop_imm(content_x, content_y, content_width, content_height);
        
        tracing::info!("AWCS Phase 3: Content area detected: {}x{} -> {}x{} (excluded menubar:{}, sidebar:{}, taskbar:{})",
                      width, height, content_width, content_height, menubar_height, sidebar_width, taskbar_height);
        
        Ok(content_area)
    }
    
    /// Preprocessing intelligent d'image pour optimiser l'OCR
    fn preprocess_for_ocr(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        
        tracing::debug!("AWCS Phase 3: Applying intelligent preprocessing for OCR");
        
        // 1. Conversion en niveaux de gris pour améliorer la qualité OCR
        let mut processed = image.grayscale();
        
        // 2. Augmentation du contraste adaptatif
        processed = self.enhance_contrast(processed)?;
        
        // 3. Réduction du bruit via filtre médian (simulation simple)
        processed = self.reduce_noise(processed)?;
        
        // 4. Amélioration de la netteté pour le texte
        processed = self.sharpen_text(processed)?;
        
        // 5. Redimensionnement intelligent si l'image est trop petite
        processed = self.upscale_if_needed(processed)?;
        
        tracing::info!("AWCS Phase 3: Image preprocessing completed: {}x{}", 
                      processed.width(), processed.height());
        
        Ok(processed)
    }
    
    /// Amélioration adaptative du contraste
    fn enhance_contrast(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        // Calcul de l'histogramme pour déterminer le contraste optimal
        let luma = image.to_luma8();
        let pixels = luma.pixels().map(|p| p[0] as f32).collect::<Vec<_>>();
        
        if pixels.is_empty() {
            return Ok(image);
        }
        
        // Calcul des percentiles pour l'ajustement automatique
        let mut sorted_pixels = pixels.clone();
        sorted_pixels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = sorted_pixels.len();
        let p5 = sorted_pixels[len * 5 / 100] as u8;  // 5e percentile
        let p95 = sorted_pixels[len * 95 / 100] as u8; // 95e percentile
        
        // Étirement de contraste adaptatif
        let contrast_range = p95.saturating_sub(p5) as f32;
        if contrast_range < 50.0 {
            // Image low-contrast : étirement agressif
            let enhanced = image.adjust_contrast(30.0);
            tracing::debug!("AWCS Phase 3: Applied aggressive contrast enhancement (low-contrast image)");
            Ok(enhanced)
        } else {
            // Image normale : étirement modéré
            let enhanced = image.adjust_contrast(10.0);
            tracing::debug!("AWCS Phase 3: Applied moderate contrast enhancement");
            Ok(enhanced)
        }
    }
    
    /// Réduction de bruit intelligent
    fn reduce_noise(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        // Pour un vrai filtre médian, on utiliserait imageproc::filter::median_filter
        // Ici on simule avec un léger blur suivi d'un sharpen
        let denoised = image.blur(0.5); // Très léger blur pour réduire le bruit
        tracing::debug!("AWCS Phase 3: Applied noise reduction");
        Ok(denoised)
    }
    
    /// Amélioration de la netteté pour le texte
    fn sharpen_text(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        // Filtre de netteté optimisé pour le texte
        let sharpened = image.unsharpen(1.0, 2); // Sigma=1.0, threshold=2
        tracing::debug!("AWCS Phase 3: Applied text sharpening");
        Ok(sharpened)
    }
    
    /// Redimensionnement intelligent si nécessaire
    fn upscale_if_needed(&self, image: image::DynamicImage) -> Result<image::DynamicImage, AWCSError> {
        let width = image.width();
        let height = image.height();
        
        // Si l'image est trop petite (moins de 800px de largeur), on l'agrandit
        if width < 800 {
            let scale_factor = 800.0 / width as f32;
            let new_width = (width as f32 * scale_factor) as u32;
            let new_height = (height as f32 * scale_factor) as u32;
            
            let upscaled = image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            tracing::info!("AWCS Phase 3: Upscaled image from {}x{} to {}x{} (factor: {:.2})", 
                          width, height, new_width, new_height, scale_factor);
            Ok(upscaled)
        } else {
            tracing::debug!("AWCS Phase 3: Image size adequate, no upscaling needed");
            Ok(image)
        }
    }
    
    /// Calcule la confiance du texte extrait basée sur des heuristiques
    fn calculate_text_confidence(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }
        
        let mut confidence = 0.5; // Base confidence
        
        // Facteur 1: Longueur du texte (plus de texte = plus fiable)
        let length_factor = (text.len() as f64 / 100.0).min(0.3);
        confidence += length_factor;
        
        // Facteur 2: Ratio caractères alphanumériques vs spéciaux
        let alphanumeric_count = text.chars().filter(|c| c.is_alphanumeric()).count();
        let total_chars = text.chars().count();
        if total_chars > 0 {
            let alphanumeric_ratio = alphanumeric_count as f64 / total_chars as f64;
            confidence += alphanumeric_ratio * 0.2;
        }
        
        // Facteur 3: Présence de mots reconnaissables
        let word_count = text.split_whitespace().filter(|word| word.len() > 2).count();
        if word_count > 0 {
            confidence += (word_count as f64 / 10.0).min(0.2);
        }
        
        // Facteur 4: Pénalité pour trop de caractères de bruit
        let noise_chars = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && !".,!?()-:;\"'".contains(*c)).count();
        let noise_ratio = noise_chars as f64 / total_chars as f64;
        confidence -= noise_ratio * 0.3;
        
        confidence.max(0.0).min(1.0)
    }
    
    /// Essayer l'OCR via transformer (TrOCR/PaddleOCR) comme fallback
    async fn try_transformer_ocr(&self, image_data: &[u8]) -> Result<OCRSimpleResult, AWCSError> {
        tracing::info!("AWCS Phase 3: Attempting transformer OCR (experimental)");
        
        // Pour l'instant, on retourne une erreur car les transformers ne sont pas encore intégrés
        // Dans une future version, on pourrait intégrer TrOCR via Candle ou onnxruntime
        
        // Simulation de ce que serait le résultat
        // TODO: Intégrer TrOCR/PaddleOCR via Candle transformers
        Err(AWCSError::OCRFailed(
            "Transformer OCR not yet implemented. Available as fallback when Candle TrOCR integration is complete.".to_string()
        ))
        
        /* Version future avec TrOCR:
        
        use crate::rag::ocr::transformer::TrOCRProcessor;
        
        let processor = TrOCRProcessor::new().await?;
        let result = processor.process_image(image_data).await?;
        
        Ok(OCRSimpleResult {
            text: result.text,
            confidence: result.confidence,
        })
        */
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