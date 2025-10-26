# GRAVIS RAG Architecture - Rust Implementation

## Vue d'ensemble

Implémentation d'un système RAG (Retrieval-Augmented Generation) robuste en Rust pour GRAVIS, utilisant les crates les plus performantes pour l'analyse de code et la recherche vectorielle.

## Stack Technologique

### Core RAG Components
- **candle-core** : Framework ML en Rust pur (alternative à PyTorch)
- **hf-hub** : Interface avec Hugging Face Hub pour les modèles
- **qdrant-client** : Base de données vectorielle haute performance
- **tokenizers** : Tokenisation rapide et efficace
- **serde** : Sérialisation des données

### Document Processing & OCR
- **tesseract-rs** : OCR pour extraction de texte depuis images/PDFs
- **pdf-extract** : Extraction de texte natif des PDFs
- **image** : Traitement d'images pour améliorer l'OCR
- **poppler-rs** : Conversion PDF vers images pour OCR
- **leptonica-sys** : Preprocessing d'images pour OCR optimal

### Intégration Tauri
- **tauri::command** : Exposition des fonctions Rust au frontend
- **tokio** : Runtime async pour les opérations I/O
- **sqlx** : Base de données pour les métadonnées
- **notify** : Watch des changements de fichiers

## Architecture Proposée

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Tauri Backend  │    │   RAG Engine    │
│   (TypeScript)  │◄──►│   (Rust)         │◄──►│   (Rust)        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │   Metadata DB    │    │   Qdrant Vector │
                       │   (SQLite)       │    │   Store         │
                       └──────────────────┘    └─────────────────┘
```

## Composants Détaillés

### 1. Document Processing Pipeline avec Groupes

```rust
// Structure des groupes de documents
#[derive(Serialize, Deserialize)]
pub struct DocumentGroup {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub chunk_config: ChunkConfig,
    pub metadata_config: MetadataConfig,
    pub documents: Vec<GroupDocument>,
    pub qdrant_collection: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

// Configuration de chunking par groupe
#[derive(Serialize, Deserialize)]
pub struct ChunkConfig {
    pub chunk_size: usize,    // 256-1024 tokens
    pub overlap: usize,       // 32-128 tokens  
    pub strategy: ChunkStrategy,
}

#[derive(Serialize, Deserialize)]
pub enum ChunkStrategy {
    AstFirst,      // Tree-sitter → fallback heuristique
    Heuristic,     // Fenêtres glissantes uniquement
    Hybrid,        // Mix AST + heuristique optimisé
}

// Structure des documents dans un groupe (remplace CodeDocument)
#[derive(Serialize, Deserialize)]
pub struct GroupDocument {
    pub id: String,
    pub file_path: PathBuf,
    pub language: String,
    pub content: String,
    pub chunks: Vec<EnrichedChunk>,
    pub metadata: EnrichedMetadata,
    pub last_modified: SystemTime,
    pub document_type: DocumentType,
    pub group_id: String,
}

#[derive(Serialize, Deserialize)]
pub enum DocumentType {
    SourceCode { language: String },
    PDF { has_text: bool, pages: usize },
    Image { ocr_confidence: f32 },
    Markdown,
    PlainText,
}

// Chunks avec métadonnées enrichies
#[derive(Serialize, Deserialize)]
pub struct EnrichedChunk {
    pub id: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType, // Function, Class, Module, etc.
    pub embedding: Option<Vec<f32>>,
    pub hash: String, // blake3 pour cache embeddings
    pub metadata: ChunkMetadata,
    pub group_id: String,
}

// Métadonnées enrichies par chunk
#[derive(Serialize, Deserialize)]  
pub struct ChunkMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub language: String,
    pub symbol: Option<String>, // Nom fonction/classe si AST
    pub context: Option<String>, // Contexte parent (imports, etc.)
    pub confidence: f32, // Score qualité du chunking
}

#[derive(Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2, 
    High = 3,
}

// Métadonnées document enrichies
#[derive(Serialize, Deserialize)]
pub struct EnrichedMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub description: Option<String>,
    pub author: Option<String>,
    pub project: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}
```

### 2. Embedding Models

**Modèles recommandés** :
- **CodeBERT** : Spécialisé pour le code source
- **UniXcoder** : Multi-langage, excellent pour la recherche de code
- **StarEncoder** : Nouveau modèle de Hugging Face pour les embeddings de code

**Implémentation** :
```rust
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::BertModel;

pub struct CodeEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl CodeEmbedder {
    pub async fn embed_code(&self, code: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Tokenisation + embedding avec candle
    }
}
```

### 3. Vector Store avec Qdrant

```rust
use qdrant_client::{QdrantClient, qdrant::*};

pub struct VectorStore {
    client: QdrantClient,
    collection_name: String,
}

impl VectorStore {
    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<ScoredPoint>, QdrantError> {
        // Recherche vectorielle avec score de similarité
    }
    
    pub async fn index_document(&self, document: &CodeDocument) -> Result<(), QdrantError> {
        // Indexation des chunks avec métadonnées
    }
}
```

### 4. Document Processor avec OCR

```rust
use tesseract::Tesseract;
use pdf_extract::extract_text;
use poppler::Document as PopplerDocument;

pub struct DocumentProcessor {
    tesseract: Tesseract,
    code_analyzer: CodeAnalyzer,
}

impl DocumentProcessor {
    pub async fn process_document(&self, file_path: &Path) -> Result<CodeDocument, ProcessingError> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "pdf" => self.process_pdf(file_path).await,
            "png" | "jpg" | "jpeg" | "tiff" => self.process_image(file_path).await,
            "rs" | "ts" | "js" | "py" | "go" => self.process_source_code(file_path).await,
            "md" => self.process_markdown(file_path).await,
            _ => self.process_text_file(file_path).await,
        }
    }

    async fn process_pdf(&self, file_path: &Path) -> Result<CodeDocument, ProcessingError> {
        // 1. Tentative d'extraction de texte natif
        if let Ok(text) = extract_text(file_path) {
            if !text.trim().is_empty() {
                return Ok(self.create_document(file_path, text, DocumentType::PDF { 
                    has_text: true, 
                    pages: self.count_pdf_pages(file_path)? 
                }));
            }
        }

        // 2. Fallback vers OCR si pas de texte natif
        self.process_pdf_with_ocr(file_path).await
    }

    async fn process_pdf_with_ocr(&self, file_path: &Path) -> Result<CodeDocument, ProcessingError> {
        let doc = PopplerDocument::new_from_file(file_path, "")?;
        let mut full_text = String::new();
        let mut total_confidence = 0.0;
        let page_count = doc.get_n_pages();

        for page_num in 0..page_count {
            let page = doc.get_page(page_num)?;
            let surface = page.render(150.0, 150.0)?; // 150 DPI pour bonne qualité OCR
            
            // Conversion en format image pour Tesseract
            let image_data = surface.get_data()?;
            
            // OCR avec Tesseract
            let result = self.tesseract
                .set_image_from_mem(&image_data)?
                .get_text()?;
                
            let confidence = self.tesseract.mean_text_conf();
            total_confidence += confidence as f32;
            
            full_text.push_str(&result);
            full_text.push('\n');
        }

        let avg_confidence = total_confidence / page_count as f32;
        
        Ok(self.create_document(
            file_path, 
            full_text,
            DocumentType::PDF { 
                has_text: false, 
                pages: page_count as usize 
            }
        ))
    }

    async fn process_image(&self, file_path: &Path) -> Result<CodeDocument, ProcessingError> {
        // Chargement et preprocessing de l'image
        let img = image::open(file_path)?;
        
        // Amélioration pour OCR (contraste, netteté, etc.)
        let enhanced_img = self.enhance_image_for_ocr(img);
        
        // Conversion en format compatible Tesseract
        let mut img_data = Vec::new();
        enhanced_img.write_to(&mut Cursor::new(&mut img_data), image::ImageFormat::Png)?;
        
        // OCR
        let text = self.tesseract
            .set_image_from_mem(&img_data)?
            .get_text()?;
            
        let confidence = self.tesseract.mean_text_conf() as f32;
        
        Ok(self.create_document(
            file_path,
            text,
            DocumentType::Image { ocr_confidence: confidence }
        ))
    }

    fn enhance_image_for_ocr(&self, img: DynamicImage) -> DynamicImage {
        // Conversion en niveaux de gris
        let gray = img.to_luma8();
        
        // Amélioration du contraste
        let enhanced = gray.pixels()
            .map(|p| {
                let val = p[0] as f32;
                let enhanced = ((val / 255.0).powf(0.8) * 255.0) as u8;
                image::Luma([enhanced])
            })
            .collect::<image::ImageBuffer<image::Luma<u8>, Vec<u8>>>();
            
        DynamicImage::ImageLuma8(enhanced)
    }
}

### 5. Code Analysis & Chunking

pub struct CodeAnalyzer {
    // Parseurs pour différents langages
    parsers: HashMap<String, TreeSitterParser>,
}

impl CodeAnalyzer {
    pub fn chunk_document(&self, content: &str, doc_type: &DocumentType) -> Vec<CodeChunk> {
        match doc_type {
            DocumentType::SourceCode { language } => self.chunk_code(content, language),
            DocumentType::PDF { .. } | DocumentType::Image { .. } => self.chunk_text_document(content),
            DocumentType::Markdown => self.chunk_markdown(content),
            DocumentType::PlainText => self.chunk_text_document(content),
        }
    }

    fn chunk_code(&self, content: &str, language: &str) -> Vec<CodeChunk> {
        match language {
            "rust" => self.chunk_rust_code(content),
            "typescript" | "javascript" => self.chunk_ts_code(content),
            "python" => self.chunk_python_code(content),
            _ => self.chunk_generic_code(content),
        }
    }
    
    fn chunk_text_document(&self, content: &str) -> Vec<CodeChunk> {
        // Chunking par paragraphes pour documents textuels
        content.split("\n\n")
            .filter(|chunk| !chunk.trim().is_empty())
            .enumerate()
            .map(|(i, chunk)| CodeChunk {
                id: format!("text_chunk_{}", i),
                content: chunk.trim().to_string(),
                start_line: 0, // À calculer basé sur la position
                end_line: 0,
                chunk_type: ChunkType::TextBlock,
                embedding: Vec::new(), // À calculer plus tard
            })
            .collect()
    }

    fn chunk_markdown(&self, content: &str) -> Vec<CodeChunk> {
        // Chunking par sections (headers) pour Markdown
        // Implémentation similaire mais avec parsing des headers
        self.chunk_text_document(content)
    }
}
```

## Feuille de Route d'Implémentation

### Phase 1 : Infrastructure de base (Semaine 1-2)

#### 1.1 Setup des dépendances Cargo.toml
```toml
[dependencies]
# ML & Embeddings
candle-core = "0.3"
candle-nn = "0.3" 
candle-transformers = "0.3"
hf-hub = "0.3"
tokenizers = "0.13"

# Vector Database
qdrant-client = "1.7"

# OCR & Document Processing
tesseract = "0.13"
pdf-extract = "0.6"
poppler = "0.4"
image = { version = "0.24", features = ["png", "jpeg", "tiff"] }
leptonica-sys = "0.4"

# Database & Storage
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
sled = "0.34" # Alternative KV store

# Async & Concurrency
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# File Processing
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-typescript = "0.20"
tree-sitter-python = "0.20"
notify = "6.0"
walkdir = "2.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Tauri Integration
tauri = { version = "2.0", features = ["api-all"] }
```

#### 1.2 Structure du projet
```
src-tauri/src/
├── main.rs
├── rag/
│   ├── mod.rs
│   ├── embedder.rs      # Gestion des embeddings
│   ├── vector_store.rs  # Interface Qdrant
│   ├── analyzer.rs      # Analyse de code
│   ├── indexer.rs       # Pipeline d'indexation
│   ├── retriever.rs     # Système de recherche
│   ├── ocr.rs          # Moteur OCR avec Tesseract
│   └── processor.rs     # Document processor unifié
├── commands/
│   ├── mod.rs
│   └── rag_commands.rs  # Commandes Tauri pour le RAG
└── models/
    ├── mod.rs
    ├── document.rs      # Structures de données
    └── chunk_types.rs   # Types de chunks (code, texte, etc.)
```

### Phase 2 : Core RAG Engine (Semaine 2-3)

#### 2.1 Embedder Implementation
- [ ] Intégration candle + hf-hub
- [ ] Chargement des modèles CodeBERT/UniXcoder
- [ ] Optimisation GPU/CPU selon disponibilité
- [ ] Cache des embeddings

#### 2.2 Vector Store Setup
- [ ] Configuration Qdrant locale
- [ ] Schéma des collections
- [ ] Index HNSW optimisé
- [ ] Gestion des métadonnées

#### 2.3 Code Analyzer
- [ ] Parseurs tree-sitter pour les langages principaux
- [ ] Stratégies de chunking intelligentes
- [ ] Extraction des fonctions/classes/modules
- [ ] Métadonnées contextuelles

### Phase 3 : Integration Tauri (Semaine 3-4)

#### 3.1 Commandes Tauri - Gestion des Groupes RAG
```rust
// === Gestion des Groupes ===
#[tauri::command]
pub async fn create_group(name: String, chunk_config: ChunkConfig) -> Result<DocumentGroup, String> {
    // Créer un nouveau groupe avec configuration de chunking
}

#[tauri::command]
pub async fn list_groups() -> Result<Vec<DocumentGroup>, String> {
    // Lister tous les groupes avec statuts
}

#[tauri::command]
pub async fn update_group(group_id: String, updates: GroupUpdates) -> Result<DocumentGroup, String> {
    // Mettre à jour nom, config chunking, métadonnées
}

#[tauri::command]
pub async fn delete_group(group_id: String) -> Result<bool, String> {
    // Supprimer groupe et sa collection Qdrant
}

#[tauri::command]
pub async fn toggle_group(group_id: String, active: bool) -> Result<bool, String> {
    // Activer/désactiver un groupe pour les requêtes
}

// === Upload et Indexation ===
#[tauri::command]
pub async fn upload_to_group(
    group_id: String, 
    files: Vec<FileUpload>, 
    metadata: UploadMetadata
) -> Result<UploadResult, String> {
    // Upload fichiers vers un groupe avec métadonnées
}

#[tauri::command]
pub async fn index_group_documents(
    group_id: String,
    progress_callback: Option<String>
) -> Result<IndexingResult, String> {
    // Indexer tous les documents d'un groupe avec progress
}

#[tauri::command]
pub async fn reindex_document(
    document_id: String,
    new_config: Option<ChunkConfig>
) -> Result<DocumentInfo, String> {
    // Re-indexer un document avec nouvelle config
}

// === Recherche Contextuelle ===
#[tauri::command]
pub async fn search_in_groups(
    query: String, 
    active_groups: Vec<String>,
    filters: SearchFilters,
    limit: usize
) -> Result<Vec<EnrichedSearchResult>, String> {
    // Recherche dans groupes actifs avec scoring hybride
}

#[tauri::command]
pub async fn get_context_for_query(
    query: String,
    max_chunks: usize
) -> Result<ContextResult, String> {
    // Récupérer contexte RAG pour injection dans prompt LLM
}

// === Gestion Documents ===
#[tauri::command]
pub async fn list_group_documents(group_id: String) -> Result<Vec<GroupDocument>, String> {
    // Lister documents d'un groupe
}

#[tauri::command]
pub async fn remove_document(document_id: String) -> Result<bool, String> {
    // Supprimer document d'un groupe
}

#[tauri::command]
pub async fn get_document_chunks(document_id: String) -> Result<Vec<EnrichedChunk>, String> {
    // Voir les chunks d'un document pour debug
}

// === Structures de données ===
#[derive(Serialize, Deserialize)]
pub struct FileUpload {
    pub path: String,
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct UploadMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub description: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct EnrichedSearchResult {
    pub chunk: EnrichedChunk,
    pub document: GroupDocument,
    pub group: DocumentGroup,
    pub score: f32,
    pub explanation: String, // Pourquoi ce résultat
}

#[derive(Serialize, Deserialize)]
pub struct ContextResult {
    pub chunks: Vec<EnrichedChunk>,
    pub total_tokens: usize,
    pub context_prompt: String, // Contexte formaté pour LLM
}
```

#### 3.2 Frontend Integration
- [ ] Interface d'indexation de projets
- [ ] Recherche en temps réel
- [ ] Affichage des résultats avec highlighting
- [ ] Gestion des filtres (langage, type, etc.)

### Phase 4 : Optimisations & Features (Semaine 4-5)

#### 4.1 Performance
- [ ] Indexation incrémentale
- [ ] Cache intelligent des requêtes
- [ ] Parallélisation des opérations
- [ ] Optimisation mémoire

#### 4.2 Features Avancées
- [ ] Recherche sémantique multi-modale
- [ ] Analyse des dépendances
- [ ] Détection de patterns/anti-patterns
- [ ] Suggestions de refactoring

## Configuration & Déploiement

### Qdrant Setup Local
```bash
# Docker
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Ou installation native
cargo install --git https://github.com/qdrant/qdrant.git
```

### Modèles Embeddings
```rust
// Configuration des modèles par défaut
const DEFAULT_MODELS: &[&str] = &[
    "microsoft/codebert-base",
    "microsoft/unixcoder-base", 
    "bigcode/starencoder",
];
```

## Métriques & Monitoring

### Métriques de Performance
- Vitesse d'indexation (docs/sec)
- Latence de recherche (ms)
- Qualité des résultats (precision@k)
- Utilisation mémoire/CPU

### Logging
```rust
use tracing::{info, warn, error, instrument};

#[instrument]
pub async fn index_document(doc: &CodeDocument) -> Result<(), IndexError> {
    info!("Indexing document: {}", doc.file_path.display());
    // ...
}
```

## Sécurité & Privacy

- **Données locales** : Tout reste sur la machine utilisateur
- **Chiffrement** : Chiffrement optionnel de la base vectorielle
- **Isolation** : Sandboxing Tauri standard
- **Audit trail** : Logging des opérations sensibles

## Ressources & Références

### Documentation
- [Candle Book](https://huggingface.co/docs/candle/index)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)

### Modèles Recommandés
- **CodeBERT** : `microsoft/codebert-base`
- **UniXcoder** : `microsoft/unixcoder-base`
- **StarEncoder** : `bigcode/starencoder`

### Benchmarks
- CodeSearchNet pour évaluation
- Métriques MRR (Mean Reciprocal Rank)
- Tests de performance sur repos réels

---

Cette architecture fournit une base solide pour un système RAG robuste et performant, entièrement intégré à l'écosystème Rust/Tauri de GRAVIS.