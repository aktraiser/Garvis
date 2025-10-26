# Rapport de Recherche : Problème tauri_plugin_macos_permissions

## Résumé Exécutif

L'application VoiceTypr rencontre un **deadlock critique** avec le plugin `tauri_plugin_macos_permissions` v2.3.0 sur macOS Sequoia 15.5. L'appel `check_accessibility_permission().await` se bloque indéfiniment sans retourner de résultat ni générer d'erreur.

## Analyse du Problème

### 🔍 Symptômes Observés

- **Blocage silencieux** : `check_accessibility_permission().await` ne retourne jamais
- **Derniers logs** : "Target OS: macOS - Using plugin check"
- **Aucune erreur générée** : Pas d'exception ou de timeout
- **Deadlock async** : Le thread tokio reste suspendu indéfiniment
- **Environnement** : macOS Sequoia 15.5, Tauri v2.7.0

### 🧬 Code Problématique

```rust
// src-tauri/src/commands/permissions.rs:20
use tauri_plugin_macos_permissions::check_accessibility_permission;
let has_permission = check_accessibility_permission().await; // ⚠️ BLOQUE ICI
```

## Recherche et Découvertes

### 1. Problèmes Connus avec le Plugin

#### a) Absence de Documentation sur les Deadlocks
- **GitHub Issues** : Aucune issue spécifique sur les deadlocks documentée
- **Crates.io** : Build documentation échoue (indicateur de problèmes techniques)
- **Tauri v2.7.0** : Compatibilité non testée/confirmée

#### b) Problèmes Généraux Identifiés
- **Re-granting permissions** : Nécessité de re-accorder les permissions après chaque mise à jour
- **Microphone permissions** : Dysfonctionnements rapportés
- **Screen recording** : Comportements inattendus sur macOS 15.5

### 2. Problèmes macOS Sequoia 15.5

#### a) Changements TCC (Transparency, Consent, and Control)
- **CVE-2025-31250** : Correctif de sécurité critique dans macOS 15.5
- **Schema TCC.db** : Nouvelles colonnes dans la base de données des permissions
- **Race conditions** : Corruption possible de la base TCC au démarrage

#### b) APIs Accessibility
- **AXIsProcessTrusted** : Rapports de valeurs incorrectes intermittentes
- **Threading issues** : Problèmes de synchronisation avec le système TCC
- **Background apps** : Problèmes spécifiques avec les apps en arrière-plan

### 3. Problèmes Tokio/Async

#### a) Patterns de Deadlock Communs
- **std::sync::Mutex + .await** : Anti-pattern classique
- **Nested runtimes** : Impossible de créer un runtime dans un runtime
- **Blocking operations** : Opérations bloquantes dans un contexte async

#### b) Diagnostic du Code VoiceTypr
```rust
// Le plugin utilise probablement std::sync::Mutex ou équivalent
// qui bloque le thread tokio lors de l'appel à AXIsProcessTrusted
```

## Solutions et Alternatives

### 🛠️ Solution 1: Implémentation Native (Recommandée)

#### a) CoreFoundation Direct
```rust
use core_foundation_sys::base::{CFRelease, TCFTypeRef};
use core_foundation_sys::dictionary::{CFDictionaryAddValue, CFDictionaryCreateMutable};
use accessibility_sys::{kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions};

async fn check_accessibility_permission_native() -> Result<bool, String> {
    // Utiliser spawn_blocking pour éviter le deadlock
    tokio::task::spawn_blocking(|| {
        unsafe {
            let options = CFDictionaryCreateMutable(
                std::ptr::null_mut(), 
                0, 
                std::ptr::null(), 
                std::ptr::null()
            );
            
            if options.is_null() {
                return Err("Failed to create CF dictionary".to_string());
            }
            
            CFDictionaryAddValue(
                options, 
                kAXTrustedCheckOptionPrompt.as_void_ptr(), 
                core_foundation_sys::number::kCFBooleanFalse.as_void_ptr()
            );
            
            let result = accessibility_sys::AXIsProcessTrusted();
            CFRelease(options as *const _);
            
            Ok(result)
        }
    }).await.map_err(|e| format!("Task join error: {}", e))?
}
```

#### b) Dépendances Requises
```toml
[dependencies]
accessibility-sys = "0.1"
core-foundation-sys = "0.8"
```

### 🛠️ Solution 2: Timeout Wrapper

```rust
use tokio::time::{timeout, Duration};

async fn check_accessibility_permission_with_timeout() -> Result<bool, String> {
    match timeout(
        Duration::from_secs(5),
        tauri_plugin_macos_permissions::check_accessibility_permission()
    ).await {
        Ok(result) => Ok(result),
        Err(_) => {
            log::error!("Plugin timeout - falling back to native check");
            check_accessibility_permission_native().await
        }
    }
}
```

### 🛠️ Solution 3: AppleScript Fallback

```rust
async fn check_accessibility_applescript() -> Result<bool, String> {
    let script = r#"
        tell application "System Events"
            try
                keystroke "test"
                return true
            on error
                return false
            end try
        end tell
    "#;
    
    tokio::task::spawn_blocking(move || {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("AppleScript error: {}", e))?;
            
        Ok(output.status.success())
    }).await.map_err(|e| format!("Task error: {}", e))?
}
```

## Plan d'Implémentation Recommandé

### Phase 1: Solution Immédiate (2-4 heures)
1. **Timeout wrapper** : Implémenter un timeout de 5 secondes
2. **Native fallback** : Ajouter l'implémentation CoreFoundation
3. **Tests** : Vérifier sur macOS Sequoia 15.5

### Phase 2: Solution Robuste (1-2 jours)
1. **Remplacer complètement** le plugin par l'implémentation native
2. **Tests exhaustifs** : Tous les cas de permissions
3. **Error handling** : Gestion complète des erreurs
4. **Documentation** : Mise à jour de la doc technique

### Phase 3: Monitoring (Continu)
1. **Logs détaillés** : Tracking des performances
2. **Métriques** : Temps de réponse des vérifications
3. **Alertes** : Détection automatique des problèmes

## Diagnostic et Debugging

### 🔬 Outils de Diagnostic

#### a) Inspection des Locks
```bash
# Identifier les threads bloqués
sudo sample [PID] 10 -file /tmp/voicetypr_deadlock.txt
```

#### b) TCC Database Inspection
```bash
# Vérifier l'état de la base TCC
sudo sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db \
  "SELECT * FROM access WHERE service='kTCCServiceAccessibility'"
```

#### c) Logs Système
```bash
# Surveiller les événements TCC
log stream --predicate 'subsystem == "com.apple.TCC"'
```

### 🧪 Tests de Reproduction

```rust
#[tokio::test]
async fn test_plugin_deadlock() {
    let start = std::time::Instant::now();
    
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        tauri_plugin_macos_permissions::check_accessibility_permission()
    ).await;
    
    match result {
        Ok(perm) => println!("Permission check succeeded: {}", perm),
        Err(_) => {
            println!("DEADLOCK CONFIRMED: Plugin hanging after {:?}", start.elapsed());
            assert!(false, "Plugin deadlock detected");
        }
    }
}
```

## Recommandations Finales

### ✅ Actions Immédiates

1. **Abandon du plugin** : Cesser d'utiliser `tauri_plugin_macos_permissions`
2. **Implémentation native** : Utiliser CoreFoundation directement
3. **spawn_blocking** : Toujours wrapper les appels natifs dans spawn_blocking
4. **Tests** : Validation sur macOS Sequoia 15.5

### ⚠️ Précautions

1. **Async safety** : Ne jamais tenir de std::sync::Mutex à travers des .await
2. **Timeouts** : Toujours timeout les opérations système
3. **Fallbacks** : Avoir des méthodes de fallback (AppleScript)
4. **Monitoring** : Logger les performances des vérifications

### 🔮 Perspectives Futures

1. **Plugin updates** : Surveiller les mises à jour du plugin
2. **Tauri v3** : Évaluer les améliorations futures
3. **macOS évolutions** : Adapter aux changements d'APIs Apple

## Conclusion

Le deadlock avec `tauri_plugin_macos_permissions` est un problème critique mais résolvable. L'implémentation native avec CoreFoundation et l'utilisation appropriée de `tokio::task::spawn_blocking` fournit une solution robuste et fiable pour macOS Sequoia 15.5.

**Priorité** : 🔴 **CRITIQUE**  
**Effort estimé** : 4-8 heures  
**Impact** : ✅ **Résolution complète**  

---

# Projet GRAVIS : Application Compagnon d'Audit et d'Analyse

## Vision du Projet

Suite aux problèmes rencontrés avec VoiceTypr, nous développons **GRAVIS** - une application compagnon spécialisée dans l'audit et l'analyse de code avec une interface moderne et des capacités d'IA avancées.

## Spécifications Fonctionnelles

### 🎯 Objectifs Principaux

1. **Interface Menu Bar** : Icône discrète dans la barre de menu macOS
2. **Popup Moderne** : Interface élégante similaire à l'aperçu fourni
3. **Commandes Vocales/Texte** : Saisie multimodale pour lancer des analyses
4. **MCP Integration** : Utilisation des Model Context Protocol servers
5. **Multi-Modèles** : Support de différents LLMs via LiteLLM
6. **Agents Spécialisés** : Focus audit de sécurité et analyse de code

### 🏗️ Architecture Technique

#### Frontend (Interface Utilisateur)
```
┌─────────────────────────────────────┐
│            Menu Bar Icon            │
│              ↓ Click                │
│  ┌─────────────────────────────────┐ │
│  │  "Poser une question" Input     │ │
│  │  ┌─────┬─────┬─────┬─────┬───┐  │ │
│  │  │  +  │ 🌐  │ 📡  │ 🔍  │ A │  │ │
│  │  └─────┴─────┴─────┴─────┴───┘  │ │
│  │  ┌─────────────────────────────┐ │ │
│  │  │ 🎤 Microphone   📊 Stats   │ │ │
│  │  └─────────────────────────────┘ │ │
│  └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

#### Backend (Rust/Tauri)
```
Core Services:
├── menu_bar.rs        # Gestion de l'icône et popup
├── voice_input.rs     # Traitement vocal (sans plugin problématique)
├── mcp_manager.rs     # Intégration MCP servers
├── litellm_client.rs  # Interface avec LiteLLM
├── agents/
│   ├── security_audit.rs    # Agent d'audit sécurité
│   ├── code_analysis.rs     # Agent d'analyse de code
│   ├── performance.rs       # Agent performance
│   └── compliance.rs        # Agent conformité
└── permissions.rs     # Permissions native (sans plugin)
```

### 🔧 Fonctionnalités Détaillées

#### 1. Interface Utilisateur
- **Design** : Interface dark moderne, inspirée de l'image fournie
- **Saisie** : Zone de texte avec placeholder "Poser une question"
- **Boutons rapides** :
  - `+` : Nouvelle analyse
  - `🌐` : Connexion aux services externes
  - `📄` : Interface RAG - Gestion documents et groupes
  - `📡` : État des MCP servers
  - `🔍` : Recherche dans l'historique
  - `A` : Sélection du modèle de langue
- **Microphone** : Bouton pour saisie vocale
- **Stats** : Indicateurs de performance en temps réel

#### 2. Commandes Supportées
```
Exemples de commandes:
- "Auditer ce fichier pour les vulnérabilités"
- "Analyser les performances de ce code"
- "Vérifier la conformité RGPD"
- "Détecter les code smells"
- "Générer un rapport de sécurité"
- "Optimiser cette fonction"
- "Chercher dans mes documents comment faire X"
- "Analyser cette fonction en tenant compte de ma documentation"
```

#### 3. Intégration MCP
- **Servers supportés** :
  - `filesystem` : Accès aux fichiers
  - `git` : Opérations Git
  - `database` : Analyse de schémas
  - `network` : Tests de sécurité réseau
  - `docker` : Audit de containers

#### 4. Système RAG Intégré - Interface Modale Avancée

##### 📄 Bouton RAG → Modale Complète
- **Positionnement** : À côté du bouton `🌐` dans la barre d'outils
- **Icône** : `📄` avec indicateur de statut (nombre de groupes actifs)
- **Ouverture** : Modale plein écran pour gestion avancée

##### 🗂️ Gestion des Groupes de Documents
```
┌─────────────────────────────────────────────────────────┐
│  📄 RAG - Gestion des Documents                        │
│                                                         │
│  ┌─ Groupes ────────────────────────────────────────┐   │
│  │ 📁 Projet Frontend [●] 12 docs    [Edit] [Del]  │   │
│  │ 📁 Documentation [○] 8 docs       [Edit] [Del]  │   │
│  │ 📁 Code Review [●] 5 docs         [Edit] [Del]  │   │
│  │ [+ Nouveau Groupe]                              │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─ Upload & Configuration ─────────────────────────┐   │
│  │ Drag & drop files here...                       │   │
│  │ [Parcourir] Support: PDF, TXT, MD, JS, TS, PY   │   │
│  │                                                 │   │
│  │ Groupe cible: [Dropdown: Projet Frontend ▼]    │   │
│  │                                                 │   │
│  │ ⚙️ Paramètres de Chunking:                      │   │
│  │ • Chunk Size: [512] tokens                     │   │
│  │ • Overlap: [64] tokens                         │   │
│  │ • Strategy: [AST-First ▼] [Heuristic]          │   │
│  │                                                 │   │
│  │ 🏷️ Métadonnées:                                 │   │
│  │ • Tags: [frontend, react, components]          │   │
│  │ • Priority: [Normal ▼]                         │   │
│  │ • Language: [Auto-detect ▼]                    │   │
│  │                                                 │   │
│  │ [Indexer Documents]                            │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

##### 🔧 Configuration Avancée par Groupe
- **Activation Toggle** : Groupes actifs/inactifs pour les requêtes
- **Paramètres de Chunking** :
  - **Chunk Size** : 256-1024 tokens (défaut: 512)
  - **Overlap** : 32-128 tokens (défaut: 64)
  - **Strategy** : AST-First / Heuristic / Hybrid
- **Métadonnées Enrichies** :
  - **Tags** : Classification libre (frontend, backend, docs, etc.)
  - **Priority** : Low / Normal / High (influence le scoring)
  - **Language** : Auto-detect / Force specific

##### 📊 Collections Qdrant Organisées
- **Structure hiérarchique** :
  ```
  qdrant_collections/
  ├── group_frontend/          # Collection par groupe
  │   ├── chunks_384d         # Embeddings E5
  │   └── metadata           # Tags, priority, etc.
  ├── group_documentation/
  └── group_code_review/
  ```

##### 🔍 Recherche Contextuelle Intelligente
- **Filtrage par groupes** : Recherche dans groupes actifs uniquement
- **Scoring hybride** : 
  - Similarité vectorielle (70%)
  - Priority metadata (20%) 
  - Freshness/usage (10%)
- **Context injection** : Résultats RAG injectés automatiquement dans les prompts LLM

#### 5. Modèles LLM (via LiteLLM)
```yaml
Modèles supportés:
- GPT-4 Turbo (général)
- Claude 3.5 Sonnet (analyse)
- GPT-4o (vision + code)
- Codellama (code spécialisé)
- DeepSeek Coder (performance)
```

### 🛠️ Stack Technique

#### Core
- **Tauri v2.x** : Framework d'application native
- **Rust** : Backend performant et sécurisé
- **TypeScript + React** : Frontend moderne
- **Tailwind CSS** : Styling rapide et consistent

#### Intégrations
- **LiteLLM** : Proxy unifié pour les LLMs
- **MCP SDK** : Protocol servers
- **Tokio** : Runtime async (avec corrections pour éviter les deadlocks)
- **Serde** : Sérialisation JSON
- **CoreFoundation** : APIs macOS natives

#### Sécurité
- **Permissions natives** : Implémentation CoreFoundation directe
- **Sandboxing** : Isolation des opérations dangereuses
- **Audit logs** : Traçabilité complète des actions
- **Rate limiting** : Protection contre l'abus d'API

### 📋 Plan de Développement

#### Phase 1 : MVP (1-2 semaines) ✅ COMPLÉTÉ
1. **Setup projet Tauri** ✅
2. **Menu bar + popup basique** ✅
3. **Interface de saisie** ✅
4. **Intégration LiteLLM simple** ✅
5. **Agent d'analyse basique** ✅

#### Phase 2 : RAG + Fonctionnalités Core (2-3 semaines)
1. **Interface Settings RAG** : Upload zone + toggle activation
2. **Backend RAG** : E5 embedder + Qdrant + chunking AST
3. **Commandes Tauri RAG** : upload, index, search, toggle
4. **Intégration MCP complète**
5. **Agents spécialisés avec contexte RAG**

#### Phase 3 : Avancé + Optimisations (3-4 semaines)
1. **Cache embeddings** + batch processing optimisé
2. **Recherche hybride** : vectorielle + BM25
3. **Streaming UI** + virtualisation résultats
4. **Métriques et analytics**
5. **Export des résultats**

### 🎨 Design System

#### Couleurs
```css
:root {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --text-primary: #ffffff;
  --text-secondary: #a0a0a0;
  --accent: #0ea5e9;
  --success: #10b981;
  --warning: #f59e0b;
  --error: #ef4444;
}
```

#### Composants
- **GlassCard** : Effet de verre pour les modales
- **CommandInput** : Zone de saisie intelligente
- **ModelSelector** : Dropdown pour choisir le LLM
- **StatusIndicator** : État des services
- **ResultPanel** : Affichage des analyses

### 🔍 Agents Spécialisés

#### 1. SecurityAuditAgent
```rust
Capabilities:
- Détection de vulnérabilités OWASP
- Analyse de dépendances
- Vérification des secrets exposés
- Audit des permissions
- Conformité sécurité
```

#### 2. CodeAnalysisAgent
```rust
Capabilities:
- Code smells detection
- Complexité cyclomatique
- Patterns anti-patterns
- Suggestions d'optimisation
- Documentation gaps
```

#### 3. PerformanceAgent
```rust
Capabilities:
- Hotspots identification
- Memory leaks detection
- Algorithmic complexity
- Resource usage analysis
- Optimization recommendations
```

#### 4. ComplianceAgent
```rust
Capabilities:
- RGPD compliance
- Accessibility standards
- Coding standards
- License compliance
- Industry regulations
```

### 🚀 Objectifs de Performance

- **Startup** : < 500ms
- **Popup opening** : < 100ms
- **Command processing** : < 2s (simple), < 30s (complex)
- **Memory usage** : < 100MB idle, < 500MB active
- **Battery impact** : Minimal (background dormant)

### 📊 Métriques de Succès

1. **Adoption** : Utilisation quotidienne par les développeurs
2. **Précision** : 95%+ de détection de vrais positifs
3. **Performance** : Réponses sub-secondes pour 80% des commandes
4. **Stabilité** : 99.9% uptime sans crashes
5. **Satisfaction** : NPS > 8/10

---

**Auteur** : Claude Code Assistant  
**Date** : 24 octobre 2025  
**Version** : 2.0  
**Statut** : Spécifications complètes - Prêt pour développement