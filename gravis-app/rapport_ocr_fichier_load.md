# Rapport OCR - Feuille de Route pour le Viewer PDF Interactif

**Date:** 2025-11-14
**Composant:** OCR Viewer avec overlays interactifs
**Status:** Architecture frontend complète, backend nécessite refonte

---

## 1. Vue d'ensemble

### Objectif
Créer un viewer PDF interactif où chaque élément de contenu (paragraphe, titre, tableau, image) détecté par OCR devient une zone cliquable/hoverable, permettant à l'utilisateur d'interagir directement avec des sections spécifiques du document.

### Architecture cible
```
Document PDF natif (texte sélectionnable)
    ↓
Extraction OCR → Grille de blocs structurés
    ↓
Frontend → Overlays interactifs sur PDF original
    ↓
Clic sur bloc → Question contextuelle au RAG
```

---

## 2. État actuel du système

### ✅ Frontend - Fonctionnel
**Fichier:** `src/components/PdfSemanticOverlay.tsx`

**Fonctionnalités implémentées:**
- ✅ Rendu PDF multi-pages avec `react-pdf` (PDF.js)
- ✅ Texte sélectionnable natif
- ✅ Overlays interactifs sur chaque page
- ✅ Normalisation des coordonnées (pixels → pourcentages)
- ✅ Hover effects et tooltips
- ✅ Click handlers pour envoyer contexte au RAG
- ✅ Highlighting des spans utilisés dans les réponses

**Exemple de code critique:**
```typescript
// Normalisation des coordonnées OCR
const normalizedX = bbox.x / ocrPage.width;      // 10.0 / 595.0 = 0.0168
const normalizedY = bbox.y / ocrPage.height;     // y / 842.0
const normalizedWidth = bbox.width / ocrPage.width;  // 580.0 / 595.0 = 0.975

// Positionnement des overlays
style={{
  left: `${normalizedX * 100}%`,
  top: `${normalizedY * 100}%`,
  width: `${normalizedWidth * 100}%`,
  height: `${normalizedHeight * 100}%`,
}}
```

### ❌ Backend - Problèmes architecturaux critiques

**Fichier problématique:** `src-tauri/src/rag/direct_chat_commands.rs:996-1133`

**Fonction défaillante:** `create_ocr_content_from_document()`

#### Problème 1: Reconstruction au lieu d'utilisation native
```rust
// ❌ MAUVAIS: Reconstruit des blocs depuis le texte plat
fn create_ocr_content_from_document(document: &GroupDocument) -> Result<OCRContent> {
    let content_lines: Vec<&str> = document.content.lines().collect();

    // Crée des bounding boxes synthétiques
    let block = OCRBlock {
        bounding_box: BoundingBox {
            x: 10.0,              // Position X fixe arbitraire
            y: current_y,         // Y incrémental synthétique
            width: 580.0,         // Largeur fixe arbitraire
            height: calculated_height,
        },
        block_type: BlockType::Paragraph,  // Type générique
        content: chunk,
        // ...
    };
}
```

**Conséquence:** Les blocs n'ont pas de positions réelles → overlays inutilisables

#### Problème 2: Une seule page générée
```rust
// ❌ MAUVAIS: Toujours une seule page
Ok(OCRContent {
    pages: vec![page],  // vec![page] au lieu de vec![page1, page2, ...]
    // ...
})
```

**Conséquence:** PDF de 22 pages → `🎯 OCR Pages: 1` → overlays seulement sur page 1

#### Problème 3: Champ page_number manquant
```rust
// Structure actuelle
pub struct OCRBlock {
    // ❌ Manque: pub page_number: u32,
    pub block_type: BlockType,
    pub content: String,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub spans: Vec<SourceSpan>,
}
```

**Conséquence:** Impossible de mapper les blocs à leurs pages d'origine

---

## 3. Architecture correcte - Spécifications

### 3.1 Structure des données OCR

#### OCRBlock (à modifier)
```rust
pub struct OCRBlock {
    pub page_number: u32,        // ← À AJOUTER
    pub block_type: BlockType,   // Header, Paragraph, Table, Figure, List, KeyValue
    pub content: String,         // Texte extrait
    pub bounding_box: BoundingBox,  // Position réelle en pixels
    pub confidence: f64,         // 0.0-1.0
    pub spans: Vec<SourceSpan>,  // Pour le highlighting
}

pub struct BoundingBox {
    pub x: f64,       // Position X en pixels (0.0 - page_width)
    pub y: f64,       // Position Y en pixels (0.0 - page_height)
    pub width: f64,   // Largeur en pixels
    pub height: f64,  // Hauteur en pixels
}
```

#### OCRPage (OK)
```rust
pub struct OCRPage {
    pub page_number: u32,
    pub blocks: Vec<OCRBlock>,
    pub width: f64,   // 595.0 pour A4 portrait
    pub height: f64,  // 842.0 pour A4 portrait
}
```

#### OCRContent (OK)
```rust
pub struct OCRContent {
    pub pages: Vec<OCRPage>,  // Une page par page du PDF
    pub total_confidence: f64,
    pub layout_analysis: Option<String>,
}
```

### 3.2 Pipeline d'extraction OCR

```
PDF/Image
    ↓
1. Extraction OCR native (tesseract, proprietary OCR)
    ↓
2. Analyse de layout → Détection de blocs structurés
    │
    ├─ Headers (titres)
    ├─ Paragraphs (texte normal)
    ├─ Tables (tableaux)
    ├─ Figures (images, graphiques)
    ├─ Lists (listes à puces/numérotées)
    └─ KeyValue (paires clé-valeur)
    ↓
3. Pour chaque bloc:
    - Extraire texte
    - Extraire bounding box (x, y, width, height en pixels)
    - Détecter page_number
    - Calculer confidence
    ↓
4. Grouper par page → Vec<OCRPage>
    ↓
5. Retourner OCRContent
```

### 3.3 Exemple de structure correcte

```json
{
  "pages": [
    {
      "page_number": 1,
      "width": 595.0,
      "height": 842.0,
      "blocks": [
        {
          "page_number": 1,
          "block_type": "Header",
          "content": "Introduction",
          "bounding_box": {
            "x": 50.0,
            "y": 100.0,
            "width": 495.0,
            "height": 30.0
          },
          "confidence": 0.98
        },
        {
          "page_number": 1,
          "block_type": "Paragraph",
          "content": "Ce document présente...",
          "bounding_box": {
            "x": 50.0,
            "y": 150.0,
            "width": 495.0,
            "height": 120.0
          },
          "confidence": 0.95
        }
      ]
    },
    {
      "page_number": 2,
      "width": 595.0,
      "height": 842.0,
      "blocks": [...]
    }
  ]
}
```

---

## 4. Plan de refonte backend

### Phase 1: Modification des structures (URGENT)

**Fichier:** `src-tauri/src/rag/core/direct_chat.rs`

```rust
// 1. Ajouter page_number à OCRBlock
pub struct OCRBlock {
    pub page_number: u32,  // ← AJOUTER ICI
    pub block_type: BlockType,
    pub content: String,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub spans: Vec<SourceSpan>,
}

// 2. Mettre à jour les constructeurs et méthodes
```

### Phase 2: Refonte de create_ocr_content_from_document (PRIORITAIRE)

**Fichier:** `src-tauri/src/rag/direct_chat_commands.rs`

**Approche:** Utiliser les blocs OCR natifs au lieu de reconstruire

```rust
// ✅ CORRECT: Utiliser les blocs natifs du document
fn create_ocr_content_from_document(
    document: &GroupDocument
) -> Result<OCRContent, String> {
    // Option A: Si document.metadata contient les blocs OCR natifs
    if let Some(native_ocr) = document.metadata.get("ocr_blocks") {
        return parse_native_ocr_blocks(native_ocr);
    }

    // Option B: Si on a un chemin vers le fichier original
    if let Some(file_path) = document.metadata.get("original_file") {
        return extract_ocr_from_file(file_path);
    }

    // Option C: Fallback actuel (pour rétrocompatibilité)
    return create_synthetic_ocr_content(document);
}

// Nouvelle fonction: Parser les blocs OCR natifs
fn parse_native_ocr_blocks(ocr_data: &serde_json::Value) -> Result<OCRContent> {
    let blocks: Vec<NativeOCRBlock> = serde_json::from_value(ocr_data.clone())?;

    // Grouper les blocs par page
    let mut pages_map: HashMap<u32, Vec<OCRBlock>> = HashMap::new();

    for native_block in blocks {
        let ocr_block = OCRBlock {
            page_number: native_block.page_number,
            block_type: map_block_type(&native_block.type_str),
            content: native_block.text,
            bounding_box: BoundingBox {
                x: native_block.bbox.x,
                y: native_block.bbox.y,
                width: native_block.bbox.width,
                height: native_block.bbox.height,
            },
            confidence: native_block.confidence,
            spans: vec![],
        };

        pages_map.entry(native_block.page_number)
            .or_insert_with(Vec::new)
            .push(ocr_block);
    }

    // Créer les OCRPage
    let mut pages: Vec<OCRPage> = pages_map.into_iter()
        .map(|(page_num, blocks)| OCRPage {
            page_number: page_num,
            blocks,
            width: 595.0,  // TODO: Extraire dimensions réelles
            height: 842.0,
        })
        .collect();

    pages.sort_by_key(|p| p.page_number);

    Ok(OCRContent {
        pages,
        total_confidence: calculate_confidence(&pages),
        layout_analysis: Some("Native OCR blocks".to_string()),
    })
}
```

### Phase 3: Amélioration de l'extraction PDF

**Fichier:** `src-tauri/src/rag/processing/document_processor.rs`

**Objectif:** Extraire les blocs OCR natifs dès le processing initial

```rust
impl DocumentProcessor {
    pub async fn process_pdf(&self, path: &Path) -> Result<ProcessedDocument> {
        // 1. Extraire le texte + métadonnées de structure
        let extraction_result = self.extract_with_layout_analysis(path).await?;

        // 2. Détecter les blocs structurés
        let ocr_blocks = self.detect_layout_blocks(&extraction_result)?;

        // 3. Stocker les blocs natifs dans les métadonnées
        let mut metadata = HashMap::new();
        metadata.insert(
            "ocr_blocks".to_string(),
            serde_json::to_value(&ocr_blocks)?
        );

        Ok(ProcessedDocument {
            content: extraction_result.text,
            metadata,
            ocr_content: Some(self.build_ocr_content(ocr_blocks)?),
            // ...
        })
    }

    fn detect_layout_blocks(&self, result: &ExtractionResult) -> Result<Vec<NativeOCRBlock>> {
        // Utiliser un OCR avec layout analysis:
        // - tesseract avec --psm 3 (Fully automatic page segmentation)
        // - PDF.js extractStructure
        // - pdfminer.six avec LAParams
        // - Azure Document Intelligence
        // - AWS Textract

        // Exemple avec pdfium-render ou pdf_extract:
        let mut blocks = Vec::new();

        for page in result.pages {
            for element in page.elements {
                let block = NativeOCRBlock {
                    page_number: page.page_num,
                    type_str: element.element_type,  // "header", "paragraph", etc.
                    text: element.text,
                    bbox: element.bounding_box,
                    confidence: element.confidence,
                };
                blocks.push(block);
            }
        }

        Ok(blocks)
    }
}
```

---

## 5. Feuille de route d'implémentation

### Étape 1: Structures de données (1-2h)
- [ ] Ajouter `page_number: u32` à `OCRBlock` dans `direct_chat.rs`
- [ ] Mettre à jour tous les constructeurs et serde impls
- [ ] Ajouter migration/compatibilité pour anciennes sessions
- [ ] Tests unitaires pour les nouvelles structures

### Étape 2: Backend - Parser natif (2-3h)
- [ ] Créer `parse_native_ocr_blocks()` dans `direct_chat_commands.rs`
- [ ] Créer structure `NativeOCRBlock` pour l'import
- [ ] Mapper les types de blocs (`block_type_from_string()`)
- [ ] Grouper blocs par page et trier
- [ ] Tests avec fixture JSON

### Étape 3: Extraction améliorée (3-4h)
- [ ] Rechercher meilleure lib Rust pour layout analysis
  - Option 1: `pdf_extract` avec structure
  - Option 2: `pdfium-render` avec annotations
  - Option 3: Bindings vers tesseract avec --psm 3
- [ ] Implémenter `detect_layout_blocks()` dans `document_processor.rs`
- [ ] Extraire bounding boxes réelles
- [ ] Stocker blocs natifs dans metadata du document
- [ ] Tests avec PDFs réels

### Étape 4: Compatibilité ascendante (1h)
- [ ] Garder ancien `create_synthetic_ocr_content()` comme fallback
- [ ] Détecter format des données (natif vs synthétique)
- [ ] Logger warnings pour documents sans blocs natifs
- [ ] Documentation de migration

### Étape 5: Frontend - Validation (1h)
- [ ] Tester avec nouveaux blocs multi-pages
- [ ] Vérifier overlays sur toutes les pages
- [ ] Valider hover/click sur chaque type de bloc
- [ ] Performance avec documents longs (100+ pages)

### Étape 6: Actions contextuelles (2h)
- [ ] Implémenter génération de questions par type de bloc
- [ ] Envoyer question contextuelle à DirectChat depuis overlay
- [ ] Highlighting bidirectionnel (réponse → blocs sources)
- [ ] UX pour édition manuelle de la question

---

## 6. Tests requis

### Tests unitaires backend
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_native_ocr_blocks_multipage() {
        let json_data = r#"{
            "blocks": [
                {"page_number": 1, "type": "Header", ...},
                {"page_number": 1, "type": "Paragraph", ...},
                {"page_number": 2, "type": "Table", ...}
            ]
        }"#;

        let ocr_content = parse_native_ocr_blocks(&json_data).unwrap();
        assert_eq!(ocr_content.pages.len(), 2);
        assert_eq!(ocr_content.pages[0].page_number, 1);
        assert_eq!(ocr_content.pages[1].page_number, 2);
    }

    #[test]
    fn test_bbox_normalization() {
        // Vérifier que les coordonnées sont en pixels
        let block = create_test_block();
        assert!(block.bounding_box.x >= 0.0);
        assert!(block.bounding_box.x < 1000.0); // Assume page width < 1000px
    }
}
```

### Tests d'intégration
1. **PDF natif avec texte**: Extraire blocs, vérifier positions
2. **PDF scanné**: OCR complet, layout analysis
3. **PDF multi-colonnes**: Détecter ordre de lecture
4. **PDF avec tableaux**: Détecter cellules et structure
5. **Document long (50+ pages)**: Performance et mémoire

---

## 7. Dépendances et outils

### Librairies Rust recommandées

#### Pour extraction PDF avec layout:
```toml
[dependencies]
# Option 1: pdf_extract (simple, léger)
pdf_extract = "0.7"

# Option 2: pdfium-render (puissant, binding vers PDFium)
pdfium-render = "0.8"

# Option 3: lopdf + layout analysis custom
lopdf = "0.32"

# Pour OCR de scans
tesseract-rs = "0.1"  # ou appel CLI
```

#### Pour analyse de layout:
```toml
# Détection de blocs/régions
opencv = "0.88"  # Pour analyse d'image si PDF scanné
imageproc = "0.24"

# NLP pour classification de blocs
rust-bert = "0.21"  # Si besoin de classifier les types
```

### Services externes (optionnel)
- **Azure Document Intelligence**: Layout analysis de qualité supérieure
- **AWS Textract**: Détection de tableaux et formulaires
- **Google Document AI**: OCR multi-langue avancé

---

## 8. Diagrammes

### Architecture actuelle (❌ Problématique)
```
PDF → document.content (texte plat)
         ↓
    Reconstruction synthétique
         ↓
    OCRContent { pages: [page1_only] }
         ↓
    Blocs avec positions inventées
         ↓
    Frontend → Overlays inutilisables
```

### Architecture cible (✅ Correcte)
```
PDF → Extraction avec layout analysis
         ↓
    Blocs natifs avec positions réelles
         ↓
    Groupement par page
         ↓
    OCRContent { pages: [page1, page2, ...] }
         ↓
    Frontend → Overlays précis sur chaque page
         ↓
    Clic → Question contextuelle au RAG
```

### Flow d'interaction utilisateur
```
1. User ouvre document
    ↓
2. OCRViewerPage charge session
    ↓
3. PdfSemanticOverlay render PDF + overlays
    ↓
4. User hover bloc → Tooltip avec type et contexte
    ↓
5. User clique bloc
    ↓
6. Question contextuelle générée: "Explique ce paragraphe: ..."
    ↓
7. Envoi à DirectChat
    ↓
8. Réponse avec SourceSpans
    ↓
9. Highlighting des blocs sources utilisés
```

---

## 9. Métriques de succès

### Critères d'acceptation
- [ ] **Multi-pages**: Overlays sur toutes les pages du PDF (100%)
- [ ] **Précision**: Bounding boxes alignées avec contenu réel (<5px erreur)
- [ ] **Performance**: Chargement < 2s pour PDF 20 pages
- [ ] **Types de blocs**: Détection correcte de 5+ types (Header, Paragraph, Table, etc.)
- [ ] **Interactions**: Hover + click fonctionnent sur 100% des blocs
- [ ] **Highlighting**: Blocs sources illuminés lors de la réponse RAG

### Métriques techniques
- Couverture de tests: >80%
- Temps de rendu overlay: <100ms par page
- Mémoire: <50MB pour PDF 100 pages
- Pas de crashes sur PDFs malformés

---

## 10. Risques et mitigations

### Risque 1: PDF complexes sans structure claire
**Mitigation:** Fallback vers l'ancien système synthétique + warning

### Risque 2: Performance sur gros documents
**Mitigation:**
- Lazy loading des overlays (render only visible pages)
- Pagination backend des blocs OCR
- Cache des positions normalisées

### Risque 3: Qualité OCR variable
**Mitigation:**
- Afficher confidence score par bloc
- Permettre édition manuelle du texte OCR
- Multiple OCR providers en fallback

---

## 11. Documentation utilisateur

### Feature: Overlays interactifs

**Pour l'utilisateur:**
> Lorsque vous ouvrez un document, chaque élément (titre, paragraphe, tableau) devient cliquable.
>
> **Hover:** Affiche le type et une suggestion de question
> **Click:** Pose automatiquement une question contextuelle à l'IA
> **Blocs surlignés:** Indiquent les sources utilisées dans la réponse

**Exemple:**
1. Hover sur un tableau → Tooltip: "Table • Résume ce tableau"
2. Click → Question envoyée: "Résume ce tableau: [contenu]"
3. Réponse IA → Tableau source surligné en bleu

---

## 12. Checklist finale

### Avant merge en production
- [ ] Toutes les phases 1-4 implémentées
- [ ] Tests unitaires passent (>80% couverture)
- [ ] Tests d'intégration avec 5 types de PDF différents
- [ ] Performance validée (<2s chargement 20 pages)
- [ ] Documentation API mise à jour
- [ ] Migration des sessions existantes testée
- [ ] Code review approuvé
- [ ] Feature flag pour rollout progressif

---

## 13. Prochaines étapes immédiates

### Cette semaine
1. **Jour 1:** Phase 1 - Modifier structure `OCRBlock` avec `page_number`
2. **Jour 2:** Phase 2 - Implémenter `parse_native_ocr_blocks()`
3. **Jour 3:** Phase 3 - Recherche lib extraction + POC
4. **Jour 4:** Phase 3 - Implémenter extraction complète
5. **Jour 5:** Tests + debugging

### Sprint suivant
- Phase 5: Actions contextuelles avancées
- Phase 6: Optimisations performance
- Intégration avec système de cache RAG
- A/B testing avec utilisateurs beta

---

## Conclusion

Le système d'overlays OCR est architecturalement correct côté frontend mais nécessite une refonte backend complète pour utiliser des blocs natifs au lieu de reconstructions synthétiques. La priorité absolue est:

1. ✅ **Ajouter `page_number` à `OCRBlock`**
2. ✅ **Réécrire `create_ocr_content_from_document()` pour parser blocs natifs**
3. ✅ **Améliorer extraction PDF pour capturer vrais blocs dès le processing**

Une fois ces 3 points résolus, les overlays fonctionneront correctement sur toutes les pages avec des positions précises, permettant une interaction riche et contextuelle avec le contenu du document.

**Estimation totale:** 10-15 heures de développement + 5 heures de tests
**Impact:** Transformation de l'expérience utilisateur pour l'analyse de documents
