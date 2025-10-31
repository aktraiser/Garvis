// GRAVIS AWCS - Screen Capture Manager
// Capture d'écran native multi-plateforme - Phase 3

use crate::awcs::types::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Gestionnaire de capture d'écran natif
#[derive(Debug)]
pub struct ScreenCaptureManager {
    platform: Platform,
    temp_dir: PathBuf,
}

#[derive(Debug)]
enum Platform {
    MacOS,
    Windows,
    Linux,
}

/// Résultat de capture d'écran
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub image_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub capture_time_ms: u64,
    pub format: String,
}

/// Zone de capture
#[derive(Debug, Clone)]
pub struct CaptureZone {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ScreenCaptureManager {
    /// Crée un nouveau gestionnaire de capture d'écran
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        };

        let temp_dir = std::env::temp_dir().join("awcs_screenshots");
        
        // Créer le dossier temporaire s'il n'existe pas
        let _ = std::fs::create_dir_all(&temp_dir);
        
        tracing::debug!("Screen capture manager initialized for platform: {:?}", platform);
        
        Self { platform, temp_dir }
    }
    
    /// Capture l'écran entier
    pub async fn capture_full_screen(&self) -> Result<ScreenshotResult, AWCSError> {
        tracing::debug!("Capturing full screen");
        
        let start_time = Instant::now();
        
        let result = match self.platform {
            Platform::MacOS => self.capture_macos_full_screen().await,
            Platform::Windows => Err(AWCSError::ScreenCaptureError("Windows capture not implemented".to_string())),
            Platform::Linux => Err(AWCSError::ScreenCaptureError("Linux capture not implemented".to_string())),
        };
        
        result
        .map(|mut result| {
            result.capture_time_ms = start_time.elapsed().as_millis() as u64;
            result
        })
    }
    
    /// Capture une fenêtre spécifique
    pub async fn capture_window(&self, window: &WindowInfo) -> Result<ScreenshotResult, AWCSError> {
        tracing::debug!("Capturing window: {} (PID: {})", window.app, window.pid);
        
        let start_time = Instant::now();
        
        let result = match self.platform {
            Platform::MacOS => self.capture_macos_window(window).await,
            Platform::Windows => Err(AWCSError::ScreenCaptureError("Windows capture not implemented".to_string())),
            Platform::Linux => Err(AWCSError::ScreenCaptureError("Linux capture not implemented".to_string())),
        };
        
        result
        .map(|mut result| {
            result.capture_time_ms = start_time.elapsed().as_millis() as u64;
            result
        })
    }
    
    /// Capture une zone spécifique de l'écran
    pub async fn capture_zone(&self, zone: &CaptureZone) -> Result<ScreenshotResult, AWCSError> {
        tracing::debug!("Capturing zone: {}x{} at ({}, {})", zone.width, zone.height, zone.x, zone.y);
        
        let start_time = Instant::now();
        
        let result = match self.platform {
            Platform::MacOS => self.capture_macos_zone(zone).await,
            Platform::Windows => Err(AWCSError::ScreenCaptureError("Windows capture not implemented".to_string())),
            Platform::Linux => Err(AWCSError::ScreenCaptureError("Linux capture not implemented".to_string())),
        };
        
        result
        .map(|mut result| {
            result.capture_time_ms = start_time.elapsed().as_millis() as u64;
            result
        })
    }
    
    // === Implémentations macOS ===
    
    #[cfg(target_os = "macos")]
    async fn capture_macos_full_screen(&self) -> Result<ScreenshotResult, AWCSError> {
        let temp_file = self.temp_dir.join(format!("full_screen_{}.png", std::process::id()));
        
        let output = tokio::process::Command::new("screencapture")
            .arg("-x") // Pas de son
            .arg("-t")
            .arg("png")
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("macOS full screen capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError("Failed to capture full screen on macOS".to_string()));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    #[cfg(target_os = "macos")]
    async fn capture_macos_window(&self, window: &WindowInfo) -> Result<ScreenshotResult, AWCSError> {
        let temp_file = self.temp_dir.join(format!("window_{}_{}.png", window.pid, std::process::id()));
        
        let output = tokio::process::Command::new("screencapture")
            .arg("-x") // Pas de son
            .arg("-l")
            .arg(window.pid.to_string())
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("macOS window capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError(format!("Failed to capture window {} on macOS", window.app)));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    #[cfg(target_os = "macos")]
    async fn capture_macos_zone(&self, zone: &CaptureZone) -> Result<ScreenshotResult, AWCSError> {
        let temp_file = self.temp_dir.join(format!("zone_{}.png", std::process::id()));
        
        let output = tokio::process::Command::new("screencapture")
            .arg("-x") // Pas de son
            .arg("-R")
            .arg(format!("{},{},{},{}", zone.x, zone.y, zone.width, zone.height))
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("macOS zone capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError("Failed to capture zone on macOS".to_string()));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    // === Implémentations Windows ===
    
    async fn capture_windows_full_screen(&self) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "windows")]
        {
        let temp_file = self.temp_dir.join(format!("full_screen_{}.png", std::process::id()));
        
        let script = format!(r#"
        Add-Type -AssemblyName System.Drawing
        Add-Type -AssemblyName System.Windows.Forms
        
        $screen = [System.Windows.Forms.SystemInformation]::VirtualScreen
        $width = $screen.Width
        $height = $screen.Height
        $left = $screen.Left
        $top = $screen.Top
        
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($left, $top, 0, 0, $bitmap.Size)
        
        $bitmap.Save("{}", [System.Drawing.Imaging.ImageFormat]::Png)
        
        $graphics.Dispose()
        $bitmap.Dispose()
        
        Write-Output "Success"
        "#, temp_file.display());
        
        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Windows full screen capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError("Failed to capture full screen on Windows".to_string()));
        }
        
        self.load_screenshot_result(&temp_file).await
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AWCSError::ScreenCaptureError("Windows capture not available on this platform".to_string()))
        }
    }
    
    async fn capture_windows_window(&self, window: &WindowInfo) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "windows")]
        {
        let temp_file = self.temp_dir.join(format!("window_{}_{}.png", window.pid, std::process::id()));
        
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
        
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        
        $bitmap.Save("{}", [System.Drawing.Imaging.ImageFormat]::Png)
        
        $graphics.Dispose()
        $bitmap.Dispose()
        
        Write-Output "Success"
        "#, window.pid, temp_file.display());
        
        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Windows window capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError(format!("Failed to capture window {} on Windows", window.app)));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    async fn capture_windows_zone(&self, zone: &CaptureZone) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "windows")]
        {
        let temp_file = self.temp_dir.join(format!("zone_{}.png", std::process::id()));
        
        let script = format!(r#"
        Add-Type -AssemblyName System.Drawing
        Add-Type -AssemblyName System.Windows.Forms
        
        $left = {}
        $top = {}
        $width = {}
        $height = {}
        
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($left, $top, 0, 0, $bitmap.Size)
        
        $bitmap.Save("{}", [System.Drawing.Imaging.ImageFormat]::Png)
        
        $graphics.Dispose()
        $bitmap.Dispose()
        
        Write-Output "Success"
        "#, zone.x, zone.y, zone.width, zone.height, temp_file.display());
        
        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Windows zone capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError("Failed to capture zone on Windows".to_string()));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    // === Implémentations Linux ===
    
    async fn capture_linux_full_screen(&self) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "linux")]
        {
        let temp_file = self.temp_dir.join(format!("full_screen_{}.png", std::process::id()));
        
        // Essayer avec gnome-screenshot d'abord
        let mut output = tokio::process::Command::new("gnome-screenshot")
            .arg("-f")
            .arg(&temp_file)
            .output()
            .await;
        
        if output.is_err() || !output.as_ref().unwrap().status.success() {
            // Fallback avec import (ImageMagick)
            output = tokio::process::Command::new("import")
                .arg("-window")
                .arg("root")
                .arg(&temp_file)
                .output()
                .await;
        }
        
        match output {
            Ok(out) if out.status.success() => self.load_screenshot_result(&temp_file).await,
            Ok(_) => Err(AWCSError::ScreenCaptureError("Failed to capture full screen on Linux".to_string())),
            Err(e) => Err(AWCSError::ScreenCaptureError(format!("Linux full screen capture failed: {}", e))),
        }
    }
    
    async fn capture_linux_window(&self, window: &WindowInfo) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "linux")]
        {
        let temp_file = self.temp_dir.join(format!("window_{}_{}.png", window.pid, std::process::id()));
        
        // Essayer avec gnome-screenshot pour une fenêtre active
        let output = tokio::process::Command::new("gnome-screenshot")
            .arg("-w") // Fenêtre active
            .arg("-f")
            .arg(&temp_file)
            .output()
            .await;
        
        match output {
            Ok(out) if out.status.success() => self.load_screenshot_result(&temp_file).await,
            Ok(_) => Err(AWCSError::ScreenCaptureError(format!("Failed to capture window {} on Linux", window.app))),
            Err(e) => Err(AWCSError::ScreenCaptureError(format!("Linux window capture failed: {}", e))),
        }
    }
    
    async fn capture_linux_zone(&self, zone: &CaptureZone) -> Result<ScreenshotResult, AWCSError> {
        #[cfg(target_os = "linux")]
        {
        let temp_file = self.temp_dir.join(format!("zone_{}.png", std::process::id()));
        
        // Utiliser import avec une géométrie spécifique
        let geometry = format!("{}x{}+{}+{}", zone.width, zone.height, zone.x, zone.y);
        
        let output = tokio::process::Command::new("import")
            .arg("-window")
            .arg("root")
            .arg("-crop")
            .arg(&geometry)
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Linux zone capture failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScreenCaptureError("Failed to capture zone on Linux".to_string()));
        }
        
        self.load_screenshot_result(&temp_file).await
    }
    
    // === Méthodes utilitaires ===
    
    /// Charge le résultat d'une capture depuis un fichier
    async fn load_screenshot_result(&self, file_path: &PathBuf) -> Result<ScreenshotResult, AWCSError> {
        // Lire le fichier image
        let image_data = tokio::fs::read(file_path).await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Failed to read screenshot file: {}", e)))?;
        
        // Obtenir les dimensions de l'image (basique, pourrait être amélioré)
        let (width, height) = self.get_image_dimensions(&image_data)?;
        
        // Nettoyer le fichier temporaire
        let _ = tokio::fs::remove_file(file_path).await;
        
        Ok(ScreenshotResult {
            image_data,
            width,
            height,
            capture_time_ms: 0, // Sera défini par l'appelant
            format: "png".to_string(),
        })
    }
    
    /// Obtient les dimensions d'une image PNG (implémentation basique)
    fn get_image_dimensions(&self, image_data: &[u8]) -> Result<(u32, u32), AWCSError> {
        // Vérification basique pour PNG
        if image_data.len() < 24 || &image_data[0..8] != b"\x89PNG\r\n\x1a\n" {
            return Err(AWCSError::ScreenCaptureError("Invalid PNG format".to_string()));
        }
        
        // Extraire width et height du header IHDR (bytes 16-23)
        let width = u32::from_be_bytes([image_data[16], image_data[17], image_data[18], image_data[19]]);
        let height = u32::from_be_bytes([image_data[20], image_data[21], image_data[22], image_data[23]]);
        
        Ok((width, height))
    }
    
    /// Nettoie les fichiers temporaires anciens
    pub async fn cleanup_temp_files(&self) -> Result<(), AWCSError> {
        let mut entries = tokio::fs::read_dir(&self.temp_dir).await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Failed to read temp directory: {}", e)))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AWCSError::ScreenCaptureError(format!("Failed to iterate temp files: {}", e)))? {
            
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    // Supprimer les fichiers de plus d'une heure
                    if modified.elapsed().unwrap_or_default().as_secs() > 3600 {
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for ScreenCaptureManager {
    fn default() -> Self {
        Self::new()
    }
}