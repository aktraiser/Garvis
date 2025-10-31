// GRAVIS AWCS - Window Detector
// Détection cross-platform de la fenêtre active

use crate::awcs::types::*;

/// Détecteur de fenêtre active multi-plateforme
#[derive(Debug)]
pub struct WindowDetector {
    platform: Platform,
}

#[derive(Debug)]
enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl WindowDetector {
    /// Crée un nouveau détecteur de fenêtre
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        };
        
        tracing::debug!("Window detector initialized for platform: {:?}", platform);
        
        Self { platform }
    }
    
    /// Récupère les informations de la fenêtre active
    pub async fn get_current_window(&self) -> Result<WindowInfo, AWCSError> {
        match self.platform {
            Platform::MacOS => self.get_macos_active_window().await,
            Platform::Windows => self.get_windows_active_window().await,
            Platform::Linux => self.get_linux_active_window().await,
        }
    }
    
    // === Implémentation macOS ===
    
    #[cfg(target_os = "macos")]
    async fn get_macos_active_window(&self) -> Result<WindowInfo, AWCSError> {
        use std::process::Command;
        
        tracing::debug!("Detecting active window on macOS");
        
        // Script AppleScript pour récupérer les informations de la fenêtre active
        let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            set windowTitle to ""
            set appPID to 0
            set bundleID to ""
            
            try
                tell application process frontApp
                    set windowTitle to name of front window
                    set appPID to unix id
                end tell
                
                # Récupérer le bundle ID via l'application
                tell application "System Events"
                    set bundleID to bundle identifier of application frontApp
                end tell
            on error
                # Si erreur, utiliser des valeurs par défaut
            end try
            
            return frontApp & "|" & windowTitle & "|" & appPID & "|" & bundleID
        end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| AWCSError::WindowDetectionFailed(format!("AppleScript execution failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::WindowDetectionFailed(
                format!("AppleScript failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();
        
        if parts.len() < 4 {
            return Err(AWCSError::WindowDetectionFailed("Invalid AppleScript output".to_string()));
        }
        
        let pid: u32 = parts[2].parse()
            .map_err(|_| AWCSError::WindowDetectionFailed("Invalid PID".to_string()))?;
        
        Ok(WindowInfo {
            app: parts[0].to_string(),
            title: parts[1].to_string(),
            pid,
            bundle_id: if parts[3].is_empty() { None } else { Some(parts[3].to_string()) },
            window_class: None,
        })
    }
    
    #[cfg(not(target_os = "macos"))]
    async fn get_macos_active_window(&self) -> Result<WindowInfo, AWCSError> {
        Err(AWCSError::WindowDetectionFailed("macOS detection not available on this platform".to_string()))
    }
    
    // === Implémentation Windows ===
    
    #[cfg(target_os = "windows")]
    async fn get_windows_active_window(&self) -> Result<WindowInfo, AWCSError> {
        use std::process::Command;
        
        tracing::debug!("Detecting active window on Windows");
        
        // PowerShell script pour récupérer la fenêtre active
        let script = r#"
        Add-Type @"
            using System;
            using System.Runtime.InteropServices;
            using System.Text;
            public class Win32 {
                [DllImport("user32.dll")]
                public static extern IntPtr GetForegroundWindow();
                [DllImport("user32.dll")]
                public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
                [DllImport("user32.dll", SetLastError=true)]
                public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
                [DllImport("user32.dll")]
                public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
            }
"@

        $hwnd = [Win32]::GetForegroundWindow()
        $title = New-Object System.Text.StringBuilder 256
        [Win32]::GetWindowText($hwnd, $title, $title.Capacity) | Out-Null

        $className = New-Object System.Text.StringBuilder 256
        [Win32]::GetClassName($hwnd, $className, $className.Capacity) | Out-Null

        $processId = 0
        [Win32]::GetWindowThreadProcessId($hwnd, [ref]$processId) | Out-Null
        $process = Get-Process -Id $processId -ErrorAction SilentlyContinue

        $appName = if ($process) { $process.ProcessName } else { "Unknown" }

        Write-Output "$appName|$($title.ToString())|$processId|$($className.ToString())"
        "#;
        
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()
            .map_err(|e| AWCSError::WindowDetectionFailed(format!("PowerShell execution failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::WindowDetectionFailed(
                format!("PowerShell failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();
        
        if parts.len() < 4 {
            return Err(AWCSError::WindowDetectionFailed("Invalid PowerShell output".to_string()));
        }
        
        let pid: u32 = parts[2].parse()
            .map_err(|_| AWCSError::WindowDetectionFailed("Invalid PID".to_string()))?;
        
        Ok(WindowInfo {
            app: parts[0].to_string(),
            title: parts[1].to_string(),
            pid,
            bundle_id: None,
            window_class: if parts[3].is_empty() { None } else { Some(parts[3].to_string()) },
        })
    }
    
    #[cfg(not(target_os = "windows"))]
    async fn get_windows_active_window(&self) -> Result<WindowInfo, AWCSError> {
        Err(AWCSError::WindowDetectionFailed("Windows detection not available on this platform".to_string()))
    }
    
    // === Implémentation Linux ===
    
    async fn get_linux_active_window(&self) -> Result<WindowInfo, AWCSError> {
        use std::process::Command;
        
        tracing::debug!("Detecting active window on Linux");
        
        // Détecter l'environnement (X11 ou Wayland)
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        
        match session_type.as_str() {
            "x11" => self.get_x11_active_window().await,
            "wayland" => self.get_wayland_active_window().await,
            _ => {
                // Essayer X11 par défaut
                tracing::warn!("Unknown session type, trying X11");
                self.get_x11_active_window().await
            }
        }
    }
    
    async fn get_x11_active_window(&self) -> Result<WindowInfo, AWCSError> {
        use std::process::Command;
        
        // Récupérer l'ID de la fenêtre active via xprop
        let output = Command::new("xprop")
            .arg("-root")
            .arg("_NET_ACTIVE_WINDOW")
            .output()
            .map_err(|e| AWCSError::WindowDetectionFailed(format!("xprop failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::WindowDetectionFailed("xprop command failed".to_string()));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        let window_id = result
            .split_whitespace()
            .last()
            .ok_or_else(|| AWCSError::WindowDetectionFailed("Invalid xprop output".to_string()))?;
        
        // Récupérer les détails de la fenêtre
        let output = Command::new("xprop")
            .arg("-id")
            .arg(window_id)
            .arg("_NET_WM_NAME")
            .arg("WM_CLASS")
            .arg("_NET_WM_PID")
            .output()
            .map_err(|e| AWCSError::WindowDetectionFailed(format!("xprop window details failed: {}", e)))?;
        
        let props = String::from_utf8_lossy(&output.stdout);
        
        // Parser les propriétés
        let mut title = "Unknown".to_string();
        let mut app = "Unknown".to_string();
        let mut pid = 0u32;
        
        for line in props.lines() {
            if line.contains("_NET_WM_NAME") {
                if let Some(name) = line.split('=').nth(1) {
                    title = name.trim().trim_matches('"').to_string();
                }
            } else if line.contains("WM_CLASS") {
                if let Some(class) = line.split('=').nth(1) {
                    // WM_CLASS retourne "instance", "class"
                    let parts: Vec<&str> = class.trim().trim_matches('"').split("\", \"").collect();
                    if let Some(class_name) = parts.last() {
                        app = class_name.trim_matches('"').to_string();
                    }
                }
            } else if line.contains("_NET_WM_PID") {
                if let Some(pid_str) = line.split('=').nth(1) {
                    pid = pid_str.trim().parse().unwrap_or(0);
                }
            }
        }
        
        Ok(WindowInfo {
            app,
            title,
            pid,
            bundle_id: None,
            window_class: None,
        })
    }
    
    async fn get_wayland_active_window(&self) -> Result<WindowInfo, AWCSError> {
        // Wayland est plus restrictif, utiliser les informations disponibles
        tracing::warn!("Wayland active window detection limited");
        
        // Essayer quelques méthodes communes
        if let Ok(output) = std::process::Command::new("swaymsg")
            .arg("-t")
            .arg("get_tree")
            .output()
        {
            // Parser la sortie JSON de sway (si disponible)
            let json_str = String::from_utf8_lossy(&output.stdout);
            // TODO: Parser JSON pour trouver la fenêtre focalisée
        }
        
        // Fallback vers des informations génériques
        Ok(WindowInfo {
            app: "Unknown (Wayland)".to_string(),
            title: "Active Window".to_string(),
            pid: std::process::id(),
            bundle_id: None,
            window_class: None,
        })
    }
}

impl Default for WindowDetector {
    fn default() -> Self {
        Self::new()
    }
}