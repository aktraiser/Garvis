// GRAVIS AWCS - Accessibility Extractor
// Extraction via APIs d'accessibilité (AX/UIA/AT-SPI)

use crate::awcs::types::*;
use serde::{Deserialize, Serialize};

/// Extracteur d'accessibilité multi-plateforme
#[derive(Debug)]
pub struct AccessibilityExtractor {
    platform: Platform,
}

#[derive(Debug)]
enum Platform {
    MacOS,
    Windows,
    Linux,
}

/// Résultat d'extraction d'accessibilité
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessibilityResult {
    pub combined_text: String,
    pub elements_count: usize,
    pub roles_detected: Vec<String>,
}

impl AccessibilityExtractor {
    /// Crée un nouveau extracteur d'accessibilité
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        };
        
        tracing::debug!("Accessibility extractor initialized for platform: {:?}", platform);
        
        Self { platform }
    }
    
    /// Extrait les éléments texte via l'API d'accessibilité
    pub async fn extract_text_elements(&mut self, window: &WindowInfo) -> Result<AccessibilityResult, AWCSError> {
        match self.platform {
            Platform::MacOS => self.extract_macos_accessibility(window).await,
            Platform::Windows => self.extract_windows_accessibility(window).await,
            Platform::Linux => self.extract_linux_accessibility(window).await,
        }
    }
    
    // === Implémentation macOS (AX API) ===
    
    #[cfg(target_os = "macos")]
    async fn extract_macos_accessibility(&self, window: &WindowInfo) -> Result<AccessibilityResult, AWCSError> {
        tracing::debug!("Extracting accessibility content on macOS for PID: {}", window.pid);
        
        // Script AppleScript utilisant l'API d'accessibilité
        let script = format!(r#"
        set targetPID to {}
        set textElements to {{}}
        set roleList to {{}}
        
        try
            tell application "System Events"
                set targetProcess to first process whose unix id is targetPID
                
                # Fonction récursive pour parcourir les éléments
                on getTextFromElement(element)
                    set elementText to ""
                    set elementRole to ""
                    
                    try
                        set elementRole to role of element
                        set end of roleList to elementRole
                        
                        # Récupérer la valeur textuelle selon le rôle
                        if elementRole is in {{"AXStaticText", "AXTextField", "AXTextArea"}} then
                            try
                                set elementText to value of element
                                if elementText is not missing value and elementText is not "" then
                                    set end of textElements to elementText
                                end if
                            end try
                        end if
                        
                        # Récupérer le titre si disponible
                        try
                            set elementTitle to title of element
                            if elementTitle is not missing value and elementTitle is not "" then
                                set end of textElements to elementTitle
                            end if
                        end try
                        
                        # Parcourir les enfants
                        try
                            set childElements to entire contents of element
                            repeat with childElement in childElements
                                my getTextFromElement(childElement)
                            end repeat
                        end try
                        
                    on error
                        # Ignorer les erreurs d'accès
                    end try
                end getTextFromElement
                
                # Commencer l'extraction depuis la fenêtre principale
                try
                    set frontWindow to front window of targetProcess
                    my getTextFromElement(frontWindow)
                on error
                    # Si pas de fenêtre, essayer depuis le processus
                    my getTextFromElement(targetProcess)
                end try
            end tell
            
            # Combiner les résultats
            set combinedText to ""
            repeat with textItem in textElements
                set combinedText to combinedText & textItem & " "
            end repeat
            
            # Éliminer les doublons de rôles
            set uniqueRoles to {{}}
            repeat with roleItem in roleList
                if roleItem is not in uniqueRoles then
                    set end of uniqueRoles to roleItem
                end if
            end repeat
            
            return combinedText & "|SEPARATOR|" & (count of textElements) & "|SEPARATOR|" & (uniqueRoles as string)
            
        on error errMsg
            return "ERROR: " & errMsg & "|SEPARATOR|0|SEPARATOR|"
        end try
        "#, window.pid);
        
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("macOS accessibility script failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScriptFailed(
                format!("Accessibility script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        self.parse_macos_result(&result)
    }
    
    #[cfg(not(target_os = "macos"))]
    async fn extract_macos_accessibility(&self, _window: &WindowInfo) -> Result<AccessibilityResult, AWCSError> {
        Err(AWCSError::UnsupportedApp("macOS accessibility not available on this platform".to_string()))
    }
    
    fn parse_macos_result(&self, result: &str) -> Result<AccessibilityResult, AWCSError> {
        let parts: Vec<&str> = result.trim().split("|SEPARATOR|").collect();
        
        if parts.len() < 3 {
            return Err(AWCSError::ExtractionFailed("Invalid accessibility result format".to_string()));
        }
        
        if parts[0].starts_with("ERROR:") {
            return Err(AWCSError::ExtractionFailed(parts[0].to_string()));
        }
        
        let elements_count: usize = parts[1].parse().unwrap_or(0);
        let roles_detected: Vec<String> = if parts[2].is_empty() {
            vec![]
        } else {
            parts[2].split(", ").map(|s| s.to_string()).collect()
        };
        
        Ok(AccessibilityResult {
            combined_text: parts[0].to_string(),
            elements_count,
            roles_detected,
        })
    }
    
    // === Implémentation Windows (UIA) ===
    
    async fn extract_windows_accessibility(&self, window: &WindowInfo) -> Result<AccessibilityResult, AWCSError> {
        tracing::debug!("Extracting accessibility content on Windows for PID: {}", window.pid);
        
        // PowerShell script utilisant UI Automation
        let script = format!(r#"
        Add-Type -AssemblyName UIAutomationClient
        Add-Type -AssemblyName UIAutomationTypes
        
        try {{
            $automation = [System.Windows.Automation.AutomationElement]::RootElement
            $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, {})
            $targetWindow = $automation.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
            
            if ($targetWindow -eq $null) {{
                Write-Output "ERROR: Window not found|SEPARATOR|0|SEPARATOR|"
                exit
            }}
            
            $textElements = @()
            $roles = @()
            
            function Get-TextFromElement($element) {{
                try {{
                    $controlType = $element.Current.ControlType.LocalizedControlType
                    $roles += $controlType
                    
                    # Récupérer le nom de l'élément
                    $name = $element.Current.Name
                    if ($name -and $name.Trim() -ne "") {{
                        $textElements += $name
                    }}
                    
                    # Récupérer la valeur si c'est un contrôle de texte
                    try {{
                        $valuePattern = $element.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
                        if ($valuePattern -and $valuePattern.Current.Value) {{
                            $textElements += $valuePattern.Current.Value
                        }}
                    }} catch {{
                        # ValuePattern non supporté
                    }}
                    
                    # Parcourir les enfants
                    $children = $element.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
                    foreach ($child in $children) {{
                        Get-TextFromElement $child
                    }}
                    
                }} catch {{
                    # Ignorer les erreurs d'accès
                }}
            }}
            
            Get-TextFromElement $targetWindow
            
            $combinedText = $textElements -join " "
            $uniqueRoles = $roles | Sort-Object | Get-Unique
            
            Write-Output "$combinedText|SEPARATOR|$($textElements.Count)|SEPARATOR|$($uniqueRoles -join ', ')"
            
        }} catch {{
            Write-Output "ERROR: $($_.Exception.Message)|SEPARATOR|0|SEPARATOR|"
        }}
        "#, window.pid);
        
        let output = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("Windows accessibility script failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScriptFailed(
                format!("UIA script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        self.parse_windows_result(&result)
    }
    
    fn parse_windows_result(&self, result: &str) -> Result<AccessibilityResult, AWCSError> {
        let parts: Vec<&str> = result.trim().split("|SEPARATOR|").collect();
        
        if parts.len() < 3 {
            return Err(AWCSError::ExtractionFailed("Invalid Windows accessibility result".to_string()));
        }
        
        if parts[0].starts_with("ERROR:") {
            return Err(AWCSError::ExtractionFailed(parts[0].to_string()));
        }
        
        let elements_count: usize = parts[1].parse().unwrap_or(0);
        let roles_detected: Vec<String> = if parts[2].is_empty() {
            vec![]
        } else {
            parts[2].split(", ").map(|s| s.to_string()).collect()
        };
        
        Ok(AccessibilityResult {
            combined_text: parts[0].to_string(),
            elements_count,
            roles_detected,
        })
    }
    
    // === Implémentation Linux (AT-SPI) ===
    
    async fn extract_linux_accessibility(&self, window: &WindowInfo) -> Result<AccessibilityResult, AWCSError> {
        tracing::debug!("Extracting accessibility content on Linux for PID: {}", window.pid);
        
        // Vérifier si AT-SPI est disponible
        if !self.check_atspi_availability().await {
            return Err(AWCSError::ExtractionFailed("AT-SPI not available".to_string()));
        }
        
        // Script Python utilisant pyatspi (si disponible)
        let script = format!(r#"
import sys
try:
    import pyatspi
    import time
    
    def find_app_by_pid(target_pid):
        desktop = pyatspi.Registry.getDesktop(0)
        for app in desktop:
            try:
                if hasattr(app, 'get_process_id') and app.get_process_id() == target_pid:
                    return app
            except:
                continue
        return None
    
    def extract_text_recursive(obj, texts, roles):
        try:
            # Récupérer le rôle
            role = obj.get_role_name()
            if role not in roles:
                roles.append(role)
            
            # Récupérer le texte selon le rôle
            if role in ['text', 'label', 'static', 'entry']:
                try:
                    text = obj.get_text(0, -1) if hasattr(obj, 'get_text') else None
                    if text and text.strip():
                        texts.append(text.strip())
                except:
                    pass
            
            # Récupérer le nom
            try:
                name = obj.name
                if name and name.strip() and name not in texts:
                    texts.append(name.strip())
            except:
                pass
            
            # Parcourir les enfants
            try:
                for i in range(obj.get_child_count()):
                    child = obj.get_child_at_index(i)
                    extract_text_recursive(child, texts, roles)
            except:
                pass
                
        except Exception as e:
            pass
    
    target_pid = {}
    app = find_app_by_pid(target_pid)
    
    if app is None:
        print("ERROR: Application not found|SEPARATOR|0|SEPARATOR|")
        sys.exit(1)
    
    texts = []
    roles = []
    
    extract_text_recursive(app, texts, roles)
    
    combined_text = " ".join(texts)
    print(f"{{combined_text}}|SEPARATOR|{{len(texts)}}|SEPARATOR|{{', '.join(set(roles))}}")
    
except ImportError:
    print("ERROR: pyatspi not available|SEPARATOR|0|SEPARATOR|")
except Exception as e:
    print(f"ERROR: {{str(e)}}|SEPARATOR|0|SEPARATOR|")
        "#, window.pid);
        
        let output = tokio::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()
            .await
            .map_err(|e| AWCSError::ScriptFailed(format!("Linux accessibility script failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AWCSError::ScriptFailed(
                format!("AT-SPI script error: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        self.parse_linux_result(&result)
    }
    
    fn parse_linux_result(&self, result: &str) -> Result<AccessibilityResult, AWCSError> {
        let parts: Vec<&str> = result.trim().split("|SEPARATOR|").collect();
        
        if parts.len() < 3 {
            return Err(AWCSError::ExtractionFailed("Invalid Linux accessibility result".to_string()));
        }
        
        if parts[0].starts_with("ERROR:") {
            return Err(AWCSError::ExtractionFailed(parts[0].to_string()));
        }
        
        let elements_count: usize = parts[1].parse().unwrap_or(0);
        let roles_detected: Vec<String> = if parts[2].is_empty() {
            vec![]
        } else {
            parts[2].split(", ").map(|s| s.to_string()).collect()
        };
        
        Ok(AccessibilityResult {
            combined_text: parts[0].to_string(),
            elements_count,
            roles_detected,
        })
    }
    
    // === Méthodes utilitaires ===
    
    async fn check_atspi_availability(&self) -> bool {
        // Vérifier si le service AT-SPI est actif
        let output = tokio::process::Command::new("systemctl")
            .args(&["--user", "is-active", "at-spi-dbus-bus"])
            .output()
            .await;
        
        match output {
            Ok(out) => out.status.success(),
            Err(_) => {
                // Fallback : vérifier si pyatspi est installé
                let python_check = tokio::process::Command::new("python3")
                    .arg("-c")
                    .arg("import pyatspi; print('OK')")
                    .output()
                    .await;
                
                python_check.map(|out| out.status.success()).unwrap_or(false)
            }
        }
    }
    
    /// Teste l'extraction d'accessibilité
    pub async fn test_extraction(&mut self, window: &WindowInfo) -> Result<bool, AWCSError> {
        match self.extract_text_elements(&window.clone()).await {
            Ok(result) => {
                tracing::info!(
                    "Accessibility test successful: {} elements, {} roles from {}",
                    result.elements_count,
                    result.roles_detected.len(),
                    window.app
                );
                Ok(result.elements_count > 0)
            },
            Err(e) => {
                tracing::warn!("Accessibility test failed for {}: {}", window.app, e);
                Ok(false)
            }
        }
    }
}

impl Default for AccessibilityExtractor {
    fn default() -> Self {
        Self::new()
    }
}