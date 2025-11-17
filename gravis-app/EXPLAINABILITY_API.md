# API d'Explainability - Traçabilité du Raisonnement IA

> **Documentation technique** pour le système de chat direct avec documents et traçabilité des sources.
>
> 📦 **Archive complète** : `EXPLAINABILITY_API_ARCHIVE_2024-11-16.md`

## Vue d'ensemble

L'API d'explainability permet de tracer précisément comment l'IA a raisonné pour produire une réponse, en identifiant les passages exacts des documents sources qui ont contribué à la génération.

### Architecture Actuelle (Novembre 2024)

**Système Simplifié - Chat Direct avec PDF natif** :

```
Document PDF → Drag & Drop → SimplePdfViewer → Sélection Native → Context Menu → Chat RAG
                      ↓              ↓                ↓               ↓            ↓
               Session PDF      react-pdf         getSelection()  Expliquer/    Backend
                                natif             window API      Résumer       RAG
```

**Composants Principaux** :
- **[SimplePdfViewer.tsx](gravis-app/src/components/SimplePdfViewer.tsx)** - Viewer PDF avec sélection native et context menu
- **[DirectChatPage.tsx](gravis-app/src/pages/DirectChatPage.tsx)** - Interface de chat avec drag & drop
- **Backend RAG** - `DirectChatSession` + commandes Tauri (`process_dropped_document`, `chat_with_dropped_document`)

### Fonctionnalités Implémentées

✅ **Interface Utilisateur** :
- Badge drag & drop élégant avec auto-resize
- PDF natif avec `react-pdf` (performant, pas d'overlay complexe)
- Sélection de texte native avec `window.getSelection()`
- Context menu avec actions "Expliquer" et "Résumer"

✅ **Backend** :
- DirectChatSession avec TTL pour sessions temporaires
- OCR + Layout Analysis intelligent pour extraction structurée
- Recherche sémantique avec CustomE5 embeddings
- Source spans pour traçabilité des citations

✅ **Communication** :
- Événements Tauri entre fenêtres (`auto_question_from_ocr`)
- Synchronisation highlights temps réel
- Questions automatiques depuis sélection PDF

## Structures de Données Essentielles

### DirectChatSession

```rust
pub struct DirectChatSession {
    pub session_id: String,
    pub document_path: PathBuf,
    pub document_name: String,
    pub document_type: DocumentType,
    pub chunks: Vec<EnrichedChunk>,
    pub ocr_content: OCRContent,
    pub embeddings: Vec<f32>,
    pub is_temporary: bool,
}
```

**Référence complète** : [direct_chat.rs](gravis-app/src-tauri/src/rag/core/direct_chat.rs)

### SourceSpan

```rust
pub struct SourceSpan {
    pub id: String,
    pub source_file: String,
    pub page_number: Option<u32>,
    pub bounding_box: Option<BoundingBox>,
    pub text_content: String,
    pub confidence_score: f64,
}
```

**Référence complète** : [source_spans.rs](gravis-app/src-tauri/src/rag/core/source_spans.rs)

### OCRContent Multi-Pages

```rust
pub struct OCRContent {
    pub pages: Vec<OCRPage>,
    pub total_confidence: f64,
}

pub struct OCRPage {
    pub page_number: u32,
    pub blocks: Vec<OCRBlock>,
    pub width: f64,
    pub height: f64,
}

pub struct OCRBlock {
    pub page_number: u32,  // Mapping page pour overlays
    pub block_type: BlockType,
    pub content: String,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
}
```

## Workflow Utilisateur

### 1. Drag & Drop d'un Document

```typescript
// DirectChatPage.tsx
const handleDrop = async (file: File) => {
  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);

  const response = await invoke('process_dropped_document', {
    filePath: file.name,
    fileData: Array.from(uint8Array),
    mimeType: file.type
  });

  setSessionId(response.session.session_id);
  // → Interface split automatique : chat gauche + PDF droit
};
```

### 2. Sélection de Texte → Actions

```typescript
// SimplePdfViewer.tsx - Context Menu
const handleTextAction = (action: 'explain' | 'summarize', text: string) => {
  const question = action === 'explain'
    ? `Explique : "${text}"`
    : `Résume : "${text}"`;

  onTextAction(action, text);  // → DirectChatPage
};
```

### 3. Chat avec Citations

```typescript
// DirectChatPage.tsx
const handleSubmit = async (query: string) => {
  const response = await invoke('chat_with_dropped_document', {
    request: {
      session_id: sessionId,
      query: query,
      selection: { text: selectedText },
      limit: null,
    }
  });

  // Afficher réponse avec sources
  setMessages([...messages, {
    content: response.response,
    sources: response.sources_summary,
    confidence: response.confidence_score
  }]);
};
```

## Commandes Tauri

### `process_dropped_document`

**Fichier** : [direct_chat_commands.rs](gravis-app/src-tauri/src/rag/direct_chat_commands.rs)

```rust
#[tauri::command]
pub async fn process_dropped_document(
    file_path: String,
    file_data: Vec<u8>,
    mime_type: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessDocumentResponse, String>
```

**Traitement** :
1. OCR + Layout Analysis (avec détection de figures/graphiques)
2. Création de chunks intelligents avec source spans
3. Génération d'embeddings CustomE5
4. Stockage session temporaire avec TTL

### `chat_with_dropped_document`

```rust
#[tauri::command]
pub async fn chat_with_dropped_document(
    request: ChatRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ChatResponse, String>
```

**Pipeline** :
1. Recherche sémantique dans chunks de la session
2. Extraction des source spans contributeurs (confidence > 0.5)
3. Génération de réponse contextuelle
4. Retour avec citations + scores de confiance

## Composants React Clés

### SimplePdfViewer

**Fichier** : [SimplePdfViewer.tsx](gravis-app/src/components/SimplePdfViewer.tsx)

**Responsabilités** :
- Affichage PDF natif avec `react-pdf`
- Gestion sélection de texte via `window.getSelection()`
- Context menu avec actions "Expliquer" / "Résumer"
- Communication des actions vers `DirectChatPage`

**Props** :
```typescript
interface SimplePdfViewerProps {
  sessionId: string;
  onTextAction?: (action: 'explain' | 'summarize', text: string) => void;
}
```

### DirectChatPage

**Fichier** : [DirectChatPage.tsx](gravis-app/src/pages/DirectChatPage.tsx)

**Responsabilités** :
- Interface split (chat gauche + PDF droit)
- Drag & drop de documents
- Gestion des messages et réponses
- Affichage des sources avec scores de confiance

## Problèmes Résolus

### Re-renders Excessifs (Novembre 2024)

**Symptôme** : Re-rendering infini empêchant les interactions utilisateur

**Solution** :
- `useCallback` avec dépendances correctes
- Pattern de refs pour éviter re-création de callbacks
- Suppression de `contextMenu` des dépendances d'effets

**Référence** : PR #4 Phase 2

### Événements Context Menu Perdus (16 Novembre 2024)

**Symptôme** : Boutons "Expliquer" et "Résumer" ne déclenchaient pas d'action

**Solution** :
- Remplacement `onClick` → `onMouseDown` (détection avant re-render)
- Ajout `stopPropagation()` sur conteneur menu
- Amélioration `handleClickOutside` pour ne pas fermer sur clic boutons

**Référence** : PR #6

---

## Historique des Développements

### ✅ PR #1 - Source Spans & Explainability (Octobre 2024)

- Source Spans avec bounding boxes + char offsets
- ExplainabilityReport avec coverage metrics
- SpanAwareChunker pour génération automatique
- 9 tests unitaires PASS

### ✅ PR #2 - Chat Direct Backend (Novembre 2024)

- DirectChatSession + DirectChatManager avec TTL
- Processing OCR intelligent + CustomE5 embeddings
- Commandes Tauri complètes
- Build backend: 0 erreurs

### ✅ PR #3 - Chat Direct MVP Fonctionnel (14 Nov 2024)

**Test de validation réussi** :
```
✅ Fichier: 2510.18234v1.pdf (DeepSeek-OCR paper)
✅ Processing: 26 sections en 849ms (confiance 70%)
✅ Chat: "fait un résumé" → 5 sources citées
✅ Sources: 48-52% pertinence, 100% confiance, 2ms recherche
```

### ✅ PR #4 - Refactoring & UI Enhancements (14 Nov 2024)

**Phase 1 - DirectChatPage** :
- Interface split avec drag & drop
- SimplePdfViewer avec sélection native
- Communication bidirectionnelle

**Phase 2 - Refactoring CommandInterface** :
- Hook `useDirectChat` (213 lignes)
- Composants réutilisables : FileBadge, OCRPanel
- Drag Counter Pattern (fix flicker)
- Auto-resize fenêtre (+40px)

**Phase 3 - Backend OCR Multi-Pages** :
- Champ `page_number` sur OCRBlock
- Parser natif de blocs OCR avec positions réelles
- Support multi-pages pour overlays futurs

### ✅ PR #5 - OCR Layout Analysis (14 Nov 2024)

- Détection automatique de figures/graphiques dans PDFs
- Routage intelligent : extraction native vs OCR+Layout
- Préservation de la structure du document
- Analyse spatiale pour détection de régions

### ✅ PR #6 - Correction Bug Context Menu (16 Nov 2024)

**Problème** : Boutons context menu ne déclenchaient aucune action

**Analyse** :
- Événements `onClick` perdus lors des re-renders excessifs (#23 → #50)
- Bouton "TEST" avec `alert()` au lieu d'appel réel
- Propagation d'événements non bloquée

**Corrections** :
1. Remplacement `onClick` → `onMouseDown` (détection immédiate)
2. Ajout `stopPropagation()` sur conteneur menu et boutons
3. Amélioration `handleClickOutside` (ne ferme plus sur clic boutons)
4. Appel correct de `onTextAction('explain', text)` et `onTextAction('summarize', text)`
5. Suppression fonction locale `handleTextAction` inutilisée
6. Fix warnings TypeScript (`_e: MouseEvent`)

**Résultat** :
- ✅ Actions "Expliquer" et "Résumer" fonctionnelles
- ✅ Détection instantanée des clics (onMouseDown)
- ✅ Pas de fermeture intempestive du menu
- ✅ Code nettoyé, maintenable

**Workflow fonctionnel** :
```
1. Sélection de texte "DeepSeek" dans PDF
2. Menu contextuel apparaît avec boutons
3. Clic sur "Expliquer" (onMouseDown immédiat)
4. Log: "🔥🔥🔥 EXPLAIN BUTTON CLICKED!"
5. Appel: onTextAction('explain', 'DeepSeek')
6. DirectChatPage génère question: "Explique : \"DeepSeek\""
7. Envoi au backend RAG
8. Réponse affichée avec sources
```

---

## Architecture Technique

### Stack Technologique

**Frontend** :
- React + TypeScript
- Tauri pour API système
- react-pdf pour affichage PDF natif
- Tailwind CSS pour styling

**Backend** :
- Rust (Tauri commands)
- CustomE5 pour embeddings
- OCR avec layout analysis
- Qdrant pour vector search

### Flux de Données

```
User Action (drag/select/chat)
    ↓
React Component (SimplePdfViewer / DirectChatPage)
    ↓
Tauri Command (invoke)
    ↓
Rust Backend (process/search/generate)
    ↓
Response (JSON avec sources + spans)
    ↓
UI Update (messages + citations)
```

---

## Points d'Attention

### Performance

- **Sessions temporaires** : TTL pour éviter accumulation mémoire
- **Embeddings on-demand** : Générés lors du premier chat, pas au processing
- **PDF natif** : Pas d'overlay complexe, juste react-pdf performant

### UX

- **Auto-resize fenêtre** : S'adapte au contenu (+40px file badge)
- **Feedback visuel** : Bordure bleue pendant drag, spinner pendant processing
- **Context menu positionné** : Centré au-dessus de la sélection avec protection débordement

### Qualité Code

- **TypeScript strict** : Pas de `any`, interfaces typées
- **Rust sans warnings** : Build propre
- **Composants réutilisables** : Architecture modulaire
- **Tests unitaires** : Coverage backend RAG core

---

## Prochaines Étapes (Roadmap)

### Court Terme (À Implémenter)

1. **Highlighting temps réel** : Surligner sources dans PDF pendant génération réponse
2. **Figure detection** : Overlay jaune sur figures détectées
3. **Multi-selection** : Sélectionner plusieurs passages pour questions complexes

### Moyen Terme (Explorations)

1. **Documents typés** : Extraction spécialisée Payslip, Invoice, BankStatement
2. **Overlay interactif** : Zones cliquables sur PDF pour questions contextuelles
3. **Export annotations** : Sauvegarder questions/réponses avec liens vers sources

### Long Terme (Vision)

1. **Mode collaboratif** : Partager sessions avec annotations
2. **Timeline questions** : Historique navigation dans document
3. **Smart suggestions** : Questions automatiques selon contexte

---

## Références

### Documentation Complète

- **Archive détaillée** : `EXPLAINABILITY_API_ARCHIVE_2024-11-16.md` (3555 lignes)
- **Code source** :
  - Frontend : `gravis-app/src/components/`, `gravis-app/src/pages/`
  - Backend : `gravis-app/src-tauri/src/rag/`

### Fichiers Clés

| Fichier | Description | Lignes |
|---------|-------------|--------|
| `SimplePdfViewer.tsx` | Viewer PDF avec sélection native | ~550 |
| `DirectChatPage.tsx` | Interface chat + drag & drop | ~386 |
| `direct_chat_commands.rs` | Commandes Tauri backend | ~1200 |
| `direct_chat.rs` | Structures DirectChatSession | ~300 |
| `source_spans.rs` | Source spans + explainability | ~400 |

---

**Dernière mise à jour** : 16 novembre 2024
**Version** : 1.0 (nettoyée)
**Archive** : `EXPLAINABILITY_API_ARCHIVE_2024-11-16.md`
