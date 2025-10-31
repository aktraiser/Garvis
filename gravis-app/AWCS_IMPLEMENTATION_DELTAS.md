# GRAVIS AWCS - Deltas d'Implémentation Critiques
## Complément Technique pour Passer de "Très Bon" à "Imbattable"

📅 **Date**: 30 Octobre 2025  
🎯 **Objectif**: Deltas concrets pour optimiser l'implémentation AWCS  
⚡ **Statut**: Spécifications techniques prêtes pour exécution  

---

## 🏗️ Matrice d'Extraction par Famille d'Applications

### 🌐 Navigateurs (Safari/Chrome/Firefox/Edge)

#### Stratégie d'Extraction Hiérarchique
```typescript
// Ordre de priorité et limites par méthode
const BrowserExtractionStrategy = {
  priority: [
    {
      method: 'extension',
      apis: ['document.body.innerText', 'document.title', 'location.href'],
      successRate: 97,
      latencyP95: 150,
      limits: ['iframes cross-origin', 'shadow DOM', 'canvas apps']
    },
    {
      method: 'applescript_automation', 
      targets: ['Safari', 'Chrome'],
      successRate: 85,
      latencyP95: 250,
      limits: ['permissions required', 'sandbox restrictions']
    },
    {
      method: 'ocr_window',
      successRate: 80,
      latencyP95: 600,
      limits: ['text density', 'font rendering']
    }
  ],
  
  deepReadTriggers: {
    textDensity: 'words < 300 || ratio(text/HTML) < 0.1',
    contentScripts: ['notion', 'confluence', 'jira', 'linear'],
    fallback: 'OCR zone ciblée'
  }
};
```

#### Extension Contract Minimal
```javascript
// content-script.js
const payload = {
  type: 'GRAVIS_PAGE_CONTEXT',
  url: location.href,
  title: document.title,
  selection: window.getSelection()?.toString() || '',
  bodyText: document.body.innerText.slice(0, 500_000), // garde-fou
  metadata: {
    wordCount: document.body.innerText.split(/\s+/).length,
    hasFrames: document.querySelectorAll('iframe').length > 0,
    hasShadowDOM: document.querySelectorAll('*').some(el => el.shadowRoot)
  }
};

chrome.runtime.sendMessage(payload);
```

### 📄 Office Suite (Word/Excel/PowerPoint)

#### macOS - AppleScript
```applescript
-- Extraction robuste avec gestion d'erreurs
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
      return {content:docContent, path:docPath, stats:docStats}
    on error
      return {content:"", path:"", error:"Document protégé ou inaccessible"}
    end try
  end if
end tell
```

#### Windows - COM
```csharp
// Extraction via COM avec gestion sécurisée
public class OfficeExtractor {
    public async Task<ContextEnvelope> ExtractWordContent() {
        try {
            var word = Marshal.GetActiveObject("Word.Application") as Microsoft.Office.Interop.Word.Application;
            if (word?.ActiveDocument != null) {
                var doc = word.ActiveDocument;
                
                // Vérification protection
                if (doc.ProtectionType != WdProtectionType.wdNoProtection) {
                    return CreateLimitedEnvelope("Document protégé");
                }
                
                // Segmentation pour gros documents
                var content = doc.Content.Text;
                if (content.Length > 1_000_000) {
                    content = ExtractByParagraphs(doc, maxWords: 50_000);
                }
                
                return CreateEnvelope(content, doc.FullName, doc.Words.Count);
            }
        } catch (Exception ex) {
            return CreateErrorEnvelope($"COM error: {ex.Message}");
        }
    }
}
```

### 📱 Applications Electron (VS Code/Slack/Notion/Obsidian)

#### Accessibilité + Heuristiques par Rôles
```rust
// src-tauri/src/awcs/electron_extractor.rs
use accessibility_sys::{AXUIElement, AXValue};

pub struct ElectronExtractor {
    role_extractors: HashMap<String, Box<dyn RoleExtractor>>,
}

impl ElectronExtractor {
    pub async fn extract_by_roles(&self, window: &WindowInfo) -> Result<String, ExtractionError> {
        let ax_element = AXUIElement::from_pid(window.pid)?;
        let mut extracted_text = String::new();
        
        // Extraction par rôles AX prioritaires
        let priority_roles = ["AXStaticText", "AXTextArea", "AXTextField", "AXDocument"];
        
        for role in priority_roles {
            if let Ok(elements) = ax_element.children_with_role(role) {
                for element in elements {
                    if let Ok(text) = element.attribute_value("AXValue") {
                        extracted_text.push_str(&text);
                        extracted_text.push('\n');
                    }
                }
            }
        }
        
        // Option "Electron assisté" via IPC bridge si autorisé
        if self.has_electron_bridge(window) {
            let bridge_content = self.query_electron_bridge(window).await?;
            extracted_text = format!("{}\n--- Bridge Content ---\n{}", extracted_text, bridge_content);
        }
        
        Ok(extracted_text)
    }
}
```

### 🏦 Applications Sandbox/Privées (Banque, SSO)

#### Mode Réduit Sécurisé
```typescript
// Détection et restriction automatique
export class SecureAppHandler {
  private sensitiveApps = [
    'com.apple.keychainaccess',
    'com.microsoft.authenticator',
    /.*banking.*/i,
    /.*bank.*/i,
    /.*finance.*/i
  ];
  
  async handleSecureApp(window: WindowInfo): Promise<ContextEnvelope> {
    if (this.isSensitiveApp(window.bundleId)) {
      return {
        source: window,
        content: {
          selection: await this.getSelectionOnly(), // Sélection utilisateur uniquement
          fulltext: null, // Jamais de texte complet
          metadata: { securityMode: 'restricted' }
        },
        confidence: {
          textCompleteness: 0.1, // Très faible par design
          sourceReliability: 0.9,
          extractionMethod: 'selection-only'
        },
        timestamp: new Date(),
        securityFlags: {
          piiRedacted: true,
          fullTextBlocked: true,
          ocrDegraded: true
        }
      };
    }
  }
}
```

---

## 🎯 Sélection Utilisateur Enrichie

### Gestuelle Lasso/Zone
```rust
// src-tauri/src/awcs/selection_overlay.rs
use tauri::Window;

#[tauri::command]
pub async fn show_zone_selector(window: Window) -> Result<SelectionResult, String> {
    // Overlay transparent Tauri pour sélection zone
    let overlay = create_selection_overlay(&window).await?;
    
    let zone = overlay.wait_for_user_selection().await?;
    
    // OCR ciblé sur la zone sélectionnée = bien plus fiable
    let screenshot = capture_zone(&zone).await?;
    let ocr_result = process_zone_ocr(screenshot, &zone).await?;
    
    Ok(SelectionResult {
        text: ocr_result.text,
        confidence: ocr_result.confidence,
        coordinates: zone,
        method: "zone_ocr"
    })
}

struct SelectionOverlay {
    start_point: (i32, i32),
    end_point: (i32, i32),
    is_selecting: bool,
}
```

### Interception Presse-Papier Intelligente
```typescript
// Détection sélection utilisateur via ⌘C
export class ClipboardInterceptor {
  private lastClipboard: string = '';
  private interceptTimeout: number = 500; // ms
  
  async interceptSelection(): Promise<string | null> {
    const beforeClipboard = await this.getClipboard();
    
    // Attendre changement presse-papier dans les 500ms
    return new Promise((resolve) => {
      const checkInterval = setInterval(async () => {
        const currentClipboard = await this.getClipboard();
        
        if (currentClipboard !== beforeClipboard && currentClipboard.length > 10) {
          clearInterval(checkInterval);
          resolve(currentClipboard);
        }
      }, 50);
      
      setTimeout(() => {
        clearInterval(checkInterval);
        resolve(null);
      }, this.interceptTimeout);
    });
  }
}
```

### Heuristique Focus Element
```rust
// Lecture uniquement de l'élément focalisé
#[tauri::command]
pub async fn extract_focused_element() -> Result<ContextEnvelope, String> {
    let focused_element = get_system_focused_element().await?;
    
    match focused_element.role {
        "TextField" | "TextArea" | "Document" => {
            let content = focused_element.get_text_content()?;
            Ok(create_focused_envelope(content, focused_element))
        },
        _ => Err("No text-focused element found".to_string())
    }
}
```

---

## 🔒 Privacy/PII Renforcée

### Redaction On-Device
```python
# awcs/pii_redactor.py
import spacy
import re
from typing import Dict, List, Tuple

class LocalPIIRedactor:
    def __init__(self):
        # Modèles légers spaCy fr/en
        self.nlp_fr = spacy.load("fr_core_news_sm")
        self.nlp_en = spacy.load("en_core_web_sm")
        
        self.patterns = {
            'email': re.compile(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'),
            'phone_fr': re.compile(r'(?:(?:\+33|0)[1-9](?:[0-9]{8}))'),
            'iban': re.compile(r'\b[A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}([A-Z0-9]?){0,16}\b'),
            'card': re.compile(r'\b(?:\d{4}[-\s]?){3}\d{4}\b'),
            'token': re.compile(r'\b[A-Za-z0-9]{32,}\b')
        }
    
    def redact_text(self, text: str, lang: str = 'auto') -> Dict:
        """Redaction locale avec conservation hash pour audit"""
        
        # Détection langue si auto
        if lang == 'auto':
            lang = self.detect_language(text[:200])
        
        # NER avec spaCy
        nlp = self.nlp_fr if lang == 'fr' else self.nlp_en
        doc = nlp(text)
        
        redacted_text = text
        redactions = []
        
        # Entités nommées
        for ent in doc.ents:
            if ent.label_ in ['PERSON', 'ORG', 'GPE']:
                redacted_text = redacted_text.replace(ent.text, f"[{ent.label_}_REDACTED]")
                redactions.append({
                    'type': ent.label_,
                    'original_hash': hashlib.sha256(ent.text.encode()).hexdigest()[:8],
                    'position': (ent.start_char, ent.end_char)
                })
        
        # Patterns regex
        for pattern_name, pattern in self.patterns.items():
            for match in pattern.finditer(text):
                redacted_text = redacted_text.replace(match.group(), f"[{pattern_name.upper()}_REDACTED]")
                redactions.append({
                    'type': pattern_name,
                    'original_hash': hashlib.sha256(match.group().encode()).hexdigest()[:8],
                    'position': (match.start(), match.end())
                })
        
        return {
            'redacted_text': redacted_text,
            'redactions': redactions,
            'pii_detected': len(redactions) > 0,
            'redaction_timestamp': datetime.utcnow().isoformat()
        }
```

---

## 🖥️ Multi-Plateforme Robuste

### Linux & Wayland
```rust
// src-tauri/src/awcs/linux_extractor.rs
use wayland_client::{Connection, Dispatch};

pub struct LinuxExtractor {
    session_type: SessionType,
}

#[derive(Debug)]
enum SessionType {
    X11,
    Wayland,
    Unknown,
}

impl LinuxExtractor {
    pub fn new() -> Self {
        let session_type = match std::env::var("XDG_SESSION_TYPE").as_deref() {
            Ok("wayland") => SessionType::Wayland,
            Ok("x11") => SessionType::X11,
            _ => SessionType::Unknown,
        };
        
        Self { session_type }
    }
    
    pub async fn extract_active_window(&self) -> Result<ContextEnvelope, ExtractionError> {
        match self.session_type {
            SessionType::X11 => self.extract_x11().await,
            SessionType::Wayland => self.extract_wayland().await,
            SessionType::Unknown => self.extract_clipboard_fallback().await,
        }
    }
    
    async fn extract_wayland(&self) -> Result<ContextEnvelope, ExtractionError> {
        // Wayland bloque capture & AX sur certaines compos
        warn!("Wayland detected - using degraded mode");
        
        // Plan B : Clipboard-first mode + OCR zone uniquement
        let clipboard_content = self.get_clipboard_content().await?;
        
        if clipboard_content.len() > 50 {
            return Ok(create_envelope_from_clipboard(clipboard_content));
        }
        
        // Fallback OCR zone avec overlay
        self.extract_ocr_zone_only().await
    }
    
    async fn extract_x11(&self) -> Result<ContextEnvelope, ExtractionError> {
        // EWMH pour fenêtre active
        let active_window = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()
            .await?;
            
        let window_id = parse_window_id(&active_window.stdout)?;
        
        // Extraction via AT-SPI
        self.extract_via_atspi(window_id).await
    }
}
```

### Detection Fenêtre Active Cross-Platform
```rust
// src-tauri/src/awcs/window_detector.rs
#[cfg(target_os = "macos")]
use core_foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;

pub async fn get_current_window() -> Result<WindowInfo, DetectionError> {
    #[cfg(target_os = "macos")]
    {
        get_macos_active_window().await
    }
    
    #[cfg(target_os = "windows")]
    {
        get_windows_active_window().await
    }
    
    #[cfg(target_os = "linux")]
    {
        get_linux_active_window().await
    }
}

#[cfg(target_os = "macos")]
async fn get_macos_active_window() -> Result<WindowInfo, DetectionError> {
    // AXUIElementCopyAttributeValue(frontmost) + bundleId + title
    let frontmost_app = AXUIElementCreateSystemWide();
    let frontmost_pid = frontmost_app.attribute_value("AXFocusedApplication")?;
    
    let bundle_id = get_bundle_id_from_pid(frontmost_pid)?;
    let window_title = get_window_title_from_pid(frontmost_pid)?;
    
    Ok(WindowInfo {
        app: get_app_name_from_bundle(&bundle_id)?,
        title: window_title,
        pid: frontmost_pid,
        bundle_id: Some(bundle_id),
        window_class: None,
    })
}

#[cfg(target_os = "windows")]
async fn get_windows_active_window() -> Result<WindowInfo, DetectionError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        
        let mut title_buffer = [0u16; 256];
        let title_len = GetWindowTextW(hwnd, &mut title_buffer);
        let title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);
        
        let process = OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id)?;
        let exe_name = get_process_name(process)?;
        
        Ok(WindowInfo {
            app: exe_name,
            title,
            pid: process_id,
            bundle_id: None,
            window_class: Some(get_window_class(hwnd)?),
        })
    }
}
```

---

## 📊 Critères d'Acceptation Mesurables

### Success Rates par Méthode
```rust
// src-tauri/src/awcs/metrics.rs
#[derive(Debug, Serialize)]
pub struct ExtractionMetrics {
    // Success rates cibles
    pub dom_success_rate: f64,        // ≥ 97% pages standard
    pub office_api_success_rate: f64, // ≥ 95% docs non protégés  
    pub accessibility_success_rate: f64, // ≥ 90% pour Electron/desktop
    pub ocr_zone_success_rate: f64,   // ≥ 85% avec WER < 15%
    
    // Budget latence P95
    pub dom_latency_p95: Duration,    // < 250ms
    pub applescript_latency_p95: Duration, // < 250ms
    pub com_latency_p95: Duration,    // < 250ms
    pub accessibility_latency_p95: Duration, // < 400ms
    pub ocr_zone_latency_p95: Duration, // < 700ms
    
    // Sécurité
    pub pii_leak_count: u64,          // = 0 (objectif)
    pub redaction_accuracy: f64,      // ≥ 99%
}

impl ExtractionMetrics {
    pub fn meets_acceptance_criteria(&self) -> bool {
        self.dom_success_rate >= 0.97
            && self.office_api_success_rate >= 0.95
            && self.accessibility_success_rate >= 0.90
            && self.ocr_zone_success_rate >= 0.85
            && self.dom_latency_p95 < Duration::from_millis(250)
            && self.accessibility_latency_p95 < Duration::from_millis(400)
            && self.ocr_zone_latency_p95 < Duration::from_millis(700)
            && self.pii_leak_count == 0
    }
}
```

### Observabilité Production-Ready
```rust
// Métriques Prometheus-compatible
lazy_static! {
    static ref AWCS_EXTRACTION_TOTAL: IntCounterVec = register_int_counter_vec!(
        "awcs_extraction_method_total",
        "Total extractions by method and app",
        &["method", "app"]
    ).unwrap();
    
    static ref AWCS_TEXT_COMPLETENESS: HistogramVec = register_histogram_vec!(
        "awcs_text_completeness_histogram", 
        "Text completeness distribution",
        &["method"],
        vec![0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 0.95, 0.99, 1.0]
    ).unwrap();
    
    static ref AWCS_LLM_TOKENS: IntCounterVec = register_int_counter_vec!(
        "awcs_llm_tokens_total",
        "LLM tokens used by provider", 
        &["provider"]
    ).unwrap();
    
    static ref AWCS_REDACTIONS: IntCounterVec = register_int_counter_vec!(
        "awcs_redactions_total",
        "PII redactions by pattern",
        &["pattern"]
    ).unwrap();
}

pub fn record_extraction(method: &str, app: &str, completeness: f64, tokens: u64) {
    AWCS_EXTRACTION_TOTAL.with_label_values(&[method, app]).inc();
    AWCS_TEXT_COMPLETENESS.with_label_values(&[method]).observe(completeness);
    
    if tokens > 0 {
        AWCS_LLM_TOKENS.with_label_values(&["local"]).inc_by(tokens);
    }
}
```

---

## 🔗 Intégration MCP - Skills Déclaratifs

### AWCS comme MCP Tool
```typescript
// src/lib/mcp-awcs-server.ts
export class AWCSMCPServer implements MCPServer {
  tools = [
    {
      name: "awcs.get_context",
      description: "Get current active window context",
      inputSchema: {
        type: "object",
        properties: {
          includeSelection: { type: "boolean", default: true },
          maxChars: { type: "number", default: 20000 },
          redactPII: { type: "boolean", default: true }
        }
      }
    }
  ];
  
  async callTool(name: string, args: any): Promise<MCPResult> {
    switch (name) {
      case "awcs.get_context":
        const context = await invoke<ContextEnvelope>('awcs_get_current_context');
        
        if (args.redactPII) {
          context.content = await this.redactPII(context.content);
        }
        
        if (args.maxChars && context.content.fulltext) {
          context.content.fulltext = context.content.fulltext.slice(0, args.maxChars);
        }
        
        return { content: [{ type: "text", text: JSON.stringify(context) }] };
        
      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  }
}
```

### Router d'Intentions Stateless
```typescript
// src/lib/awcs-intention-router.ts
export class AWCSIntentionRouter {
  private mcpClients: Map<string, MCPClient> = new Map();
  
  async routeIntention(intention: IntentionResult, context: ContextEnvelope): Promise<TaskResult> {
    const routingTable = {
      'summary': 'mcp.rag.summarize',
      'verify': 'mcp.web.fact_check', 
      'translate': 'mcp.translate.run',
      'search': 'mcp.web.search',
      'explain': 'mcp.rag.explain'
    };
    
    const mcpTool = routingTable[intention.type];
    if (!mcpTool) {
      throw new Error(`No MCP tool for intention: ${intention.type}`);
    }
    
    const [server, tool] = mcpTool.split('.');
    const mcpClient = this.mcpClients.get(server);
    
    if (!mcpClient) {
      throw new Error(`MCP server not available: ${server}`);
    }
    
    // Appel MCP stateless avec contexte
    return await mcpClient.callTool(tool, {
      text: context.content.selection || context.content.fulltext,
      origin: context.source.app,
      type: context.document?.type,
      url: context.document?.url,
      metadata: context.content.metadata
    });
  }
}
```

---

## 🚀 PoC Hyper-Ciblé (2-4 jours)

### Milestone 1: Active Window + DOM
```bash
# Jour 1-2: Foundation
- Extension Chromium basique (manifest v3)
- Message contract Tauri ↔ Extension
- DOM extraction (Safari/Chrome)
- Sélection > fulltext prioritization
- Budget P95 < 250ms

# Critères d'acceptation:
✅ Extension injecte content script
✅ Message contract fonctionne
✅ DOM extraction réussit sur 10 sites test
✅ Latence P95 < 250ms mesurée
```

### Milestone 2: Office Integration  
```bash
# Jour 2-3: Cross-platform Office
- AppleScript Word/Safari (macOS)
- COM Word (Windows)
- Gestion document vide/protégé
- Récupération content + path

# Critères d'acceptation:
✅ Word content extraction (mac/win)
✅ Gestion erreurs documents protégés
✅ Metadata enrichies (wordCount, path)
✅ Tests sur 5 documents différents
```

### Milestone 3: OCR Zone
```bash
# Jour 3-4: Interactive Selection
- Overlay Tauri pour sélection rectangle
- OCR FR/EN sur zone sélectionnée
- WER < 15% sur fixtures
- Retour texte + bbox

# Critères d'acceptation:
✅ Overlay sélection fonctionnel
✅ OCR zone précis (WER < 15%)
✅ Tests sur 5 fixtures visuelles
✅ Coordonnées bbox correctes
```

### Milestone 4: PII Redaction
```bash
# Jour 4: Security Layer
- spaCy NER local (fr/en)
- Patterns regex (email/tel/token)
- Benchmark < 20ms / 1000 tokens
- Hash conservation audit

# Critères d'acceptation:
✅ NER détecte 95% des entités test
✅ Regex patterns fonctionnels
✅ Performance < 20ms mesurée
✅ Hashes correctement conservés
```

---

## 🎨 UX Micro-Détails

### Bandeau Contexte Informatif
```tsx
// src/components/AWCSContextBanner.tsx
export const AWCSContextBanner: React.FC<{ context: ContextEnvelope }> = ({ context }) => {
  const getMethodIcon = (method: string) => {
    const icons = {
      'dom': '🌐',
      'applescript': '🍎', 
      'com': '🪟',
      'accessibility': '♿',
      'ocr': '📷'
    };
    return icons[method] || '🔧';
  };
  
  const getConfidenceColor = (confidence: number) => {
    if (confidence > 0.9) return 'text-green-600';
    if (confidence > 0.7) return 'text-yellow-600';
    return 'text-red-600';
  };
  
  return (
    <div className="awcs-context-banner bg-gray-50 p-3 rounded-lg mb-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <FileText size={16} className="text-blue-600" />
          <span className="font-medium">{context.source.app}</span>
          <span className="text-gray-600 text-sm">{context.source.title}</span>
          {context.document?.url && (
            <a href={context.document.url} className="text-blue-500 text-sm hover:underline">
              🔗 {new URL(context.document.url).hostname}
            </a>
          )}
        </div>
        
        <div className="flex items-center space-x-4 text-sm">
          <span className="flex items-center space-x-1">
            <span>{getMethodIcon(context.confidence.extraction_method)}</span>
            <span>{context.confidence.extraction_method}</span>
          </span>
          
          <span className={`font-semibold ${getConfidenceColor(context.confidence.text_completeness)}`}>
            {Math.round(context.confidence.text_completeness * 100)}% fiable
          </span>
          
          {context.content.selection && (
            <span className="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs">
              📝 Sélection active
            </span>
          )}
        </div>
      </div>
    </div>
  );
};
```

### Boutons Rapides Contextuels
```tsx
// Actions rapides adaptées au type de contenu
export const QuickActions: React.FC<{ context: ContextEnvelope }> = ({ context }) => {
  const getContextualActions = () => {
    const baseActions = [
      { query: "Résume ce contenu en 5 points", icon: "📝", label: "Résumé" },
      { query: "Propose 3 actions à partir de ce contenu", icon: "💡", label: "Actions" }
    ];
    
    // Actions contextuelles par type d'app
    if (context.source.app.includes('Safari') || context.source.app.includes('Chrome')) {
      baseActions.push(
        { query: "Vérifie les informations de cette page", icon: "🔍", label: "Vérifier" },
        { query: "Trouve les liens importants", icon: "🔗", label: "Liens" }
      );
    }
    
    if (context.source.app.includes('Word') || context.source.app.includes('Pages')) {
      baseActions.push(
        { query: "Corrige le style et la grammaire", icon: "✏️", label: "Corriger" },
        { query: "Génère un plan pour ce document", icon: "📋", label: "Plan" }
      );
    }
    
    return baseActions;
  };
  
  return (
    <div className="quick-actions grid grid-cols-2 md:grid-cols-4 gap-2 mb-4">
      {getContextualActions().map((action, index) => (
        <button
          key={index}
          onClick={() => handleQuickAction(action.query)}
          className="p-3 bg-white border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors text-left"
        >
          <div className="text-lg mb-1">{action.icon}</div>
          <div className="text-sm font-medium">{action.label}</div>
        </button>
      ))}
    </div>
  );
};
```

### Privacy Toast
```tsx
// Premier lancement - transparence privacy
export const PrivacyToast: React.FC = () => {
  return (
    <div className="privacy-toast bg-blue-50 border-l-4 border-blue-400 p-4 mb-4">
      <div className="flex">
        <div className="flex-shrink-0">
          <Shield className="h-5 w-5 text-blue-400" />
        </div>
        <div className="ml-3">
          <p className="text-sm text-blue-700">
            <strong>Protection vie privée :</strong> GRAVIS lit uniquement le texte de la fenêtre active. 
            Aucun mot de passe. Données sensibles masquées localement avant tout traitement.
          </p>
        </div>
      </div>
    </div>
  );
};
```

---

## ✅ Checklist Exécutable "Prête à Coder"

### Infrastructure de Base
```bash
# Backend Rust
□ Extension Chromium (manifest v3) + message contract
□ AppleScript extractor (Word/Safari) 
□ COM extractor (Word Windows)
□ AX/UIA extractor minimal (role-based text harvest)
□ Overlay OCR zone + Tesseract (FR/EN)
□ NER local PII redaction (spaCy sm fr/en)

# API Contracts  
□ awcs_get_current_context() -> ContextEnvelope
□ awcs_handle_query(query) -> (intention → execution → response)
□ awcs_show_zone_selector() -> SelectionResult
□ awcs_redact_pii(text) -> RedactionResult

# Observabilité
□ Prometheus counters/histograms (method, app, completeness)
□ JSON structured logs (filebeat compatible)
□ Performance monitoring (P95 latencies)
□ Error tracking (sentry-compatible)
```

### Interface Utilisateur
```bash
# UX Components
□ Palette ⌘⇧G (global shortcut)
□ Context banner (app, title, method, confidence %)
□ Quick actions (contextual by app type)
□ Deep DOM read toggle (explicit)
□ Zone selection overlay (lasso/rectangle)
□ Privacy toast (first launch transparency)

# Interactions
□ Selection clipboard intercept (⌘C detection)
□ Focus element heuristic (text fields priority)
□ Error graceful degradation (fallback chains)
□ Confirmation dialogs (sensitive actions)
```

### Sécurité & Policies
```bash
# Security Layer
□ Application allowlist (bundleId/processName)
□ PII pattern blocklist (dynamic + static)
□ Rate limiting (actions per minute)
□ Audit trail (action logging + retention)
□ Sensitive app detection (banking/auth)

# Cross-Platform
□ macOS: TCC permissions flow (automation + accessibility)
□ Windows: UIA permissions + COM registration
□ Linux: X11/Wayland detection + degraded mode fallback
□ Permission error handling (clear user guidance)
```

### Tests & Validation
```bash
# Acceptance Tests
□ DOM extraction: 97% success rate (10 sites test)
□ Office API: 95% success rate (5 documents test)  
□ AX/UIA: 90% success rate (3 Electron apps)
□ OCR zone: 85% success rate, WER < 15% (5 fixtures)
□ PII redaction: 99% accuracy (email/phone/iban test)

# Performance Tests
□ DOM/AppleScript/COM: P95 < 250ms
□ AX/UIA: P95 < 400ms
□ OCR zone: P95 < 700ms
□ NER redaction: < 20ms / 1000 tokens
□ End-to-end: P95 < 2s (capture → analysis → response)
```

### Documentation
```bash
# User Documentation
□ Compatibility matrix (app × method × limitations)
□ Scenarios de repli (fallback chains par plateforme)
□ Privacy policy (data handling + retention)
□ Troubleshooting (permissions + common errors)

# Developer Documentation  
□ Architecture overview (AWCS ↔ Extension ↔ MCP)
□ API reference (ContextEnvelope + IntentionResult)
□ Extension guide (message contract + content scripts)
□ Deployment guide (permissions + distribution)
```

---

## 🎯 Definition of Done

### Epic AWCS Core
```
✅ User peut activer AWCS via ⌘⇧G sur n'importe quelle fenêtre
✅ Extraction automatique avec fallbacks (DOM → API → AX → OCR)
✅ Context banner affiche source + méthode + fiabilité
✅ Quick actions génèrent réponses contextuelles < 2s P95
✅ PII redaction automatique avant envoi à LLM
✅ Metrics Prometheus exposées (/metrics endpoint)
✅ Tests acceptance passent (success rates + latencies)
✅ Documentation utilisateur complète
✅ Zero PII leak détecté en audit
```

---

## 🎛️ Activation AWCS - Intégration UI

### Activation dans ConnectionTab (Recommandé)
```typescript
// Extension de ConnectionTab.tsx - Section AWCS
const AWCSSection = () => {
  const [awcsState, setAwcsState] = useState<AWCSActivationState>(AWCSActivationState.Disabled);
  const [showPermissionsHelp, setShowPermissionsHelp] = useState(false);
  
  return (
    <div className="awcs-activation-section border-t pt-4 mt-4">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <Eye size={18} className="text-blue-600" />
          <h3 className="font-medium">Active Window Context Service</h3>
          <Badge variant="outline" className="text-xs">BETA</Badge>
        </div>
        
        <AWCSActivationButton 
          state={awcsState} 
          onStateChange={setAwcsState}
          disabled={!isConnected}
        />
      </div>
      
      <p className="text-sm text-gray-600 mb-3">
        Analysez le contenu de votre fenêtre active avec <kbd className="bg-gray-100 px-1 rounded">⌘⇧G</kbd>
      </p>
      
      {/* État d'activation dynamique */}
      {awcsState === AWCSActivationState.Active && (
        <div className="bg-green-50 p-3 rounded-lg text-sm">
          <div className="flex items-center space-x-2 text-green-700">
            <CheckCircle size={14} />
            <span>AWCS actif • Raccourci : <kbd className="bg-white px-1 rounded">⌘⇧G</kbd></span>
          </div>
          <div className="mt-2 text-green-600">
            Extraction intelligente • Privacy-first • Données locales d'abord
          </div>
        </div>
      )}
      
      {awcsState === AWCSActivationState.PermissionsPending && (
        <div className="bg-amber-50 p-3 rounded-lg text-sm">
          <div className="flex items-center space-x-2 text-amber-700">
            <AlertCircle size={14} />
            <span>Configuration des permissions système en cours...</span>
          </div>
          <button 
            onClick={() => setShowPermissionsHelp(true)}
            className="mt-2 text-amber-600 hover:text-amber-800 underline text-xs"
          >
            Aide avec les permissions
          </button>
        </div>
      )}
      
      {awcsState === AWCSActivationState.Error && (
        <div className="bg-red-50 p-3 rounded-lg text-sm">
          <div className="flex items-center space-x-2 text-red-700">
            <XCircle size={14} />
            <span>Échec de l'activation AWCS</span>
          </div>
          <div className="mt-2 text-red-600">
            Vérifiez les permissions système dans Préférences Système
          </div>
        </div>
      )}
    </div>
  );
};
```

### États d'Activation Progressifs
```typescript
// src/types/awcs.ts
export enum AWCSActivationState {
  Disabled = 'disabled',
  PermissionsPending = 'permissions_pending', 
  PermissionsGranted = 'permissions_granted',
  Active = 'active',
  Error = 'error'
}

export interface AWCSPermissions {
  accessibility: boolean;
  automation: boolean;
  screenRecording: boolean;
}

// src/hooks/useAWCS.ts
export const useAWCS = () => {
  const [state, setState] = useState<AWCSActivationState>(AWCSActivationState.Disabled);
  const [permissions, setPermissions] = useState<AWCSPermissions | null>(null);
  
  const activateAWCS = async () => {
    try {
      setState(AWCSActivationState.PermissionsPending);
      
      // 1. Vérification permissions système
      const currentPermissions = await invoke<AWCSPermissions>('awcs_check_permissions');
      setPermissions(currentPermissions);
      
      if (!currentPermissions.accessibility || !currentPermissions.automation) {
        // Demande permissions avec guidance utilisateur
        await invoke('awcs_request_permissions');
        
        // Attente confirmation utilisateur (polling)
        await waitForPermissions();
      }
      
      // 2. Test extraction sur fenêtre courante
      const testContext = await invoke<ContextEnvelope>('awcs_get_current_context');
      if (!testContext) throw new Error('Test extraction failed');
      
      // 3. Setup raccourci global
      await invoke('awcs_setup_global_shortcut');
      
      // 4. Activation complète
      setState(AWCSActivationState.Active);
      
      // 5. Toast confirmation
      toast.success('AWCS activé ! Utilisez ⌘⇧G sur n\'importe quelle fenêtre');
      
    } catch (error) {
      setState(AWCSActivationState.Error);
      toast.error(`Activation AWCS échouée : ${error.message}`);
    }
  };
  
  const deactivateAWCS = async () => {
    try {
      await invoke('awcs_cleanup');
      setState(AWCSActivationState.Disabled);
      toast.info('AWCS désactivé');
    } catch (error) {
      toast.error(`Erreur désactivation : ${error.message}`);
    }
  };
  
  const testCurrentWindow = async () => {
    try {
      const context = await invoke<ContextEnvelope>('awcs_get_current_context');
      toast.success(`Extraction réussie : ${context.source.app} (${Math.round(context.confidence.text_completeness * 100)}% fiable)`);
      return context;
    } catch (error) {
      toast.error(`Test échoué : ${error.message}`);
      return null;
    }
  };
  
  return {
    state,
    permissions,
    activateAWCS,
    deactivateAWCS, 
    testCurrentWindow,
    isActive: state === AWCSActivationState.Active
  };
};
```

### Bouton d'Action Principal
```typescript
// src/components/AWCSActivationButton.tsx
interface AWCSActivationButtonProps {
  state: AWCSActivationState;
  onStateChange: (state: AWCSActivationState) => void;
  disabled?: boolean;
}

export const AWCSActivationButton: React.FC<AWCSActivationButtonProps> = ({
  state,
  disabled = false
}) => {
  const { activateAWCS, deactivateAWCS, testCurrentWindow } = useAWCS();
  
  const getButtonConfig = () => {
    switch (state) {
      case AWCSActivationState.Disabled:
        return {
          text: 'Activer Context Service',
          icon: <Eye size={16} />,
          variant: 'outline' as const,
          action: activateAWCS,
          loading: false
        };
      case AWCSActivationState.PermissionsPending:
        return {
          text: 'Configuration...',
          icon: <Loader2 size={16} className="animate-spin" />,
          variant: 'outline' as const,
          action: () => {},
          loading: true
        };
      case AWCSActivationState.Active:
        return {
          text: '⌘⇧G Actif',
          icon: <CheckCircle size={16} />,
          variant: 'default' as const,
          action: deactivateAWCS,
          loading: false
        };
      case AWCSActivationState.Error:
        return {
          text: 'Réessayer',
          icon: <AlertCircle size={16} />,
          variant: 'destructive' as const,
          action: activateAWCS,
          loading: false
        };
    }
  };
  
  const config = getButtonConfig();
  
  return (
    <div className="flex items-center space-x-2">
      {state === AWCSActivationState.Active && (
        <Button
          variant="ghost"
          size="sm"
          onClick={testCurrentWindow}
          className="text-xs"
        >
          <TestTube size={14} />
          Tester
        </Button>
      )}
      
      <Button
        variant={config.variant}
        onClick={config.action}
        disabled={disabled || config.loading}
        className="min-w-[140px]"
      >
        {config.icon}
        {config.text}
      </Button>
    </div>
  );
};
```

### Flow de Permissions Guidé
```typescript
// src/components/AWCSPermissionsDialog.tsx
export const AWCSPermissionsDialog: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  permissions: AWCSPermissions;
}> = ({ isOpen, onClose, permissions }) => {
  const steps = [
    {
      id: 'accessibility',
      title: 'Accessibilité',
      description: 'Permet à GRAVIS de lire le contenu des applications',
      required: true,
      granted: permissions.accessibility,
      instructions: 'Préférences Système > Sécurité et confidentialité > Accessibilité'
    },
    {
      id: 'automation',
      title: 'Automation',
      description: 'Permet l\'extraction via AppleScript/COM',
      required: true,
      granted: permissions.automation,
      instructions: 'Préférences Système > Sécurité et confidentialité > Automation'
    },
    {
      id: 'screenRecording',
      title: 'Enregistrement d\'écran',
      description: 'Pour le fallback OCR uniquement (optionnel)',
      required: false,
      granted: permissions.screenRecording,
      instructions: 'Préférences Système > Sécurité et confidentialité > Enregistrement d\'écran'
    }
  ];
  
  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center space-x-2">
            <Shield size={20} />
            <span>Permissions AWCS</span>
          </DialogTitle>
          <DialogDescription>
            GRAVIS a besoin de certaines permissions pour analyser vos fenêtres actives
          </DialogDescription>
        </DialogHeader>
        
        <div className="space-y-4">
          {steps.map((step) => (
            <div key={step.id} className="flex items-start space-x-3 p-3 rounded-lg border">
              <div className="flex-shrink-0 mt-1">
                {step.granted ? (
                  <CheckCircle size={16} className="text-green-600" />
                ) : step.required ? (
                  <AlertCircle size={16} className="text-amber-600" />
                ) : (
                  <Info size={16} className="text-gray-400" />
                )}
              </div>
              
              <div className="flex-1">
                <div className="flex items-center space-x-2">
                  <h4 className="font-medium text-sm">{step.title}</h4>
                  {step.required && <Badge variant="outline" className="text-xs">Requis</Badge>}
                </div>
                <p className="text-xs text-gray-600 mt-1">{step.description}</p>
                {!step.granted && (
                  <p className="text-xs text-blue-600 mt-2 font-mono">{step.instructions}</p>
                )}
              </div>
            </div>
          ))}
        </div>
        
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Fermer
          </Button>
          <Button onClick={() => invoke('awcs_open_system_preferences')}>
            Ouvrir Préférences
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
```

### Intégration dans useConnection
```typescript
// Extension de src/hooks/useConnection.ts
export const useConnection = () => {
  // ... état existant
  const [awcsState, setAwcsState] = useState<AWCSActivationState>(AWCSActivationState.Disabled);
  
  // Chargement état AWCS au démarrage
  useEffect(() => {
    if (isConnected) {
      loadAWCSState();
    }
  }, [isConnected]);
  
  const loadAWCSState = async () => {
    try {
      const state = await invoke<AWCSActivationState>('awcs_get_state');
      setAwcsState(state);
    } catch (error) {
      console.error('Failed to load AWCS state:', error);
    }
  };
  
  const activateAWCS = async () => {
    // Logique d'activation complète
    const { activateAWCS: activate } = useAWCS();
    await activate();
  };
  
  const deactivateAWCS = async () => {
    await invoke('awcs_cleanup');
    setAwcsState(AWCSActivationState.Disabled);
  };
  
  return {
    // ... retour existant
    awcsState,
    setAwcsState,
    activateAWCS,
    deactivateAWCS,
    isAWCSActive: awcsState === AWCSActivationState.Active
  };
};
```

### Raccourci Global et Palette
```typescript
// src/components/AWCSPalette.tsx - Palette contextuelle ⌘⇧G
export const AWCSPalette: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [context, setContext] = useState<ContextEnvelope | null>(null);
  const [query, setQuery] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  
  // Setup raccourci global
  useEffect(() => {
    const handleGlobalShortcut = async () => {
      try {
        // Extraction contexte automatique
        const currentContext = await invoke<ContextEnvelope>('awcs_get_current_context');
        setContext(currentContext);
        setIsOpen(true);
      } catch (error) {
        toast.error('Échec extraction contexte');
      }
    };
    
    // Écoute événement Tauri du raccourci global
    const unlisten = listen('awcs-shortcut-triggered', handleGlobalShortcut);
    
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);
  
  const handleQuery = async () => {
    if (!query.trim() || !context) return;
    
    setIsProcessing(true);
    try {
      const result = await invoke<TaskResult>('awcs_handle_query', {
        query,
        context
      });
      
      // Affichage résultat dans interface principale
      window.postMessage({
        type: 'AWCS_RESULT',
        payload: { query, result, context }
      }, '*');
      
      setIsOpen(false);
      setQuery('');
      
    } catch (error) {
      toast.error(`Erreur traitement : ${error.message}`);
    } finally {
      setIsProcessing(false);
    }
  };
  
  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center space-x-2">
            <Eye size={20} />
            <span>GRAVIS Context Analysis</span>
          </DialogTitle>
        </DialogHeader>
        
        {context && (
          <>
            <AWCSContextBanner context={context} />
            
            <div className="space-y-4">
              <div className="flex space-x-2">
                <Input
                  placeholder="Que voulez-vous savoir sur ce contenu ?"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleQuery()}
                  className="flex-1"
                />
                <Button 
                  onClick={handleQuery}
                  disabled={isProcessing || !query.trim()}
                >
                  {isProcessing ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
                  Analyser
                </Button>
              </div>
              
              <QuickActions context={context} onAction={setQuery} />
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
};
```

---

Cette spécification complète le rapport AWCS existant avec des deltas concrets et mesurables, prêts pour une implémentation production. Le focus sur les micro-détails UX, l'activation progressive et la matrice d'extraction robuste transforme AWCS d'une bonne idée en solution imbattable.