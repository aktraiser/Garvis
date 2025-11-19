// GRAVIS RAG Module - Phase 4: Clean Production Architecture  
// Architecture simplifiée avec CustomE5 + QdrantRest pour stabilité maximum

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

// === Modules Phase 4: Clean Architecture ===
// Core components
pub mod core;
// Search and retrieval
pub mod search;
// Document processing
pub mod processing;
// Text normalization
pub mod text;
// OCR functionality
pub mod ocr;
// Tauri commands
pub mod commands;
// Phase 2: Chat Direct commands
pub mod direct_chat_commands;

// Phase 4 exports - Production ready
pub use search::{
    CustomE5Config, CustomE5Embedder, EnhancedBM25Encoder,
    ScoringEngine, SearchIntent, IntentWeights
};
pub use core::{
    QdrantRestClient, QdrantRestConfig, RestPoint, RestSearchResponse
};
// Phase 2 OCR exports - Command-based implementation
pub use ocr::{
    OcrConfig, OcrResult, OcrPageResult, TesseractProcessor, TesseractConfig,
    PageSegMode, OcrEngineMode, BoundingBox, OcrMetadata, OcrError, OcrCache,
    CacheConfig, PreprocessConfig, PerformanceConfig, get_available_languages,
    get_tesseract_version, detect_file_format, FileFormat
};

// OCR Commands pour Tauri - Phase 2
pub use ocr::commands::{OcrCommands, OcrState, OcrCommandResponse};

// Core components exports
pub use core::{
    EmbedderManager, get_embedder, get_embedder_with_config,
    IngestionEngine, StrategyDetector, IngestionStrategy, IngestionResult,
    BatchIngestionResult, CacheStats,
    UnifiedCache, CachedDocument, CacheCleanupResult, CacheMetrics,
    // Phase 4A: Source Spans & Explainability
    SourceSpan, SourceSpanManager, CoordinateSystem, 
    ExtractionMetadata, ExplainabilityReport, SourceSpanError, SpanStats
};

// Alias pour éviter conflits avec BoundingBox de direct_chat
pub use crate::rag::core::source_spans::BoundingBox as SourceBoundingBox;

// Processing exports
pub use processing::{
    DocumentProcessor, DocumentClassifier, DocumentCategory, BusinessSignals,
    SmartChunker, SmartChunkConfig, SmartChunkResult, ChunkSection,
    BusinessMetadata, BusinessSection, FinancialKPI, BusinessMetadataEnricher,
    // Phase 4A: Span-Aware Chunking
    SpanAwareChunker, SpanAwareChunkConfig, SpanAwareChunkResult, 
    ChunkingStats, SpanChunkError
};

// Search exports
pub use search::{
    MMRReranker, SearchResult as MMRSearchResult,
    SearchEngine, EnhancedSearchResult, QueryIntent, l2_normalize, 
    detect_query_intent, compute_hybrid_score
};

// Text normalization exports
pub use text::{
    LigatureCleaner, record_ligature_global, log_ligature_summary_global, reset_ligature_counters_global,
    sanitize_pdf_text, detect_ligatures, clean_extracted_text, NormalizationStats
};
pub use commands::{
    RagState, DocumentIngestionResponse, SearchResponseWithMetadata, SearchResultWithMetadata,
    AdvancedSearchParams, DocumentMetadataResponse, CacheStats as CommandsCacheStats
};

// Phase 2: Chat Direct exports
pub use direct_chat_commands::{
    DirectChatState, ProcessDocumentResponse, ChatRequest, ChatResponse, SourceSummary
};

// === Core Data Structures (Phase 1) ===

/// Groupe de documents avec configuration de chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Configuration de chunking par groupe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    pub chunk_size: usize,    // 256-1024 tokens
    pub overlap: usize,       // 32-128 tokens  
    pub strategy: ChunkStrategy,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 384,   // Optimisé pour E5-small-v2 (256-512 tokens idéal)
            overlap: 48,       // 12.5% d'overlap pour continuité sans redondance excessive
            strategy: ChunkStrategy::AstFirst,
        }
    }
}

/// Stratégie de chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStrategy {
    AstFirst,      // Tree-sitter → fallback heuristique
    Heuristic,     // Fenêtres glissantes uniquement
    Hybrid,        // Mix AST + heuristique optimisé
}

/// Configuration des métadonnées par défaut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub default_tags: Vec<String>,
    pub default_priority: Priority,
    pub auto_language_detection: bool,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            default_tags: vec!["general".to_string()],
            default_priority: Priority::Normal,
            auto_language_detection: true,
        }
    }
}

/// Priorité pour le scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2, 
    High = 3,
}

/// Document dans un groupe
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub ocr_blocks: Vec<crate::rag::core::direct_chat::OCRBlock>,  // Figure blocks from PDF extraction
}

/// Type de document avec stratégies intelligentes - Phase 1 OCR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    SourceCode { language: String },
    PDF { 
        extraction_strategy: PdfStrategy,
        native_text_ratio: f32,
        ocr_pages: Vec<usize>,
        total_pages: usize,
    },
    Image { 
        ocr_result: OcrResult,
        preprocessing_config: PreprocessConfig,
    },
    Markdown,
    PlainText,
}

/// Stratégie d'extraction PDF intelligente - Phase 1 OCR
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PdfStrategy {
    NativeOnly,         // Extraction native uniquement
    OcrOnly,           // OCR uniquement (PDF scanné)
    HybridIntelligent, // Pipeline hybride avec heuristiques
}

/// Chunk enrichi avec métadonnées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedChunk {
    pub id: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType,
    pub embedding: Option<Vec<f32>>,
    pub hash: String, // blake3 pour cache embeddings
    pub metadata: ChunkMetadata,
    pub group_id: String,
    // Phase 4A: Source Spans & Explainability
    pub source_spans: Option<Vec<String>>, // Références aux span IDs
    // Phase 3: Vision-Aware RAG
    pub chunk_source: ChunkSource, // D'où vient ce chunk (body, figure, table...)
    pub figure_id: Option<String>, // ID de la figure si applicable (ex: "Figure 3", "Table 1")
}

/// Type de chunk
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    TextBlock,
    Comment,
}

/// Source du chunk - Vision-Aware RAG Phase 3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkSource {
    /// Texte du corps principal du document
    BodyText,
    /// Légende de figure (ex: "Figure 3: Compression ratio")
    FigureCaption,
    /// Texte OCR extrait de la zone d'une figure/graphique
    FigureRegionText,
    /// Texte extrait d'un tableau
    Table,
    /// En-tête de section
    SectionHeader,
}

/// Type de source pour l'extraction - Phase 1 OCR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    NativeText,        // Texte directement lisible
    OcrExtracted,      // Texte extrait par OCR
    HybridPdfNative,   // PDF avec texte natif
    HybridPdfOcr,      // PDF avec zones OCR
}

/// Méthode d'extraction utilisée - Phase 1 OCR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtractionMethod {
    DirectRead,                                    // Lecture directe fichier
    TesseractOcr { confidence: f32, language: String }, // OCR Tesseract
    PdfNative,                                     // Extraction PDF native
    PdfOcrFallback,                               // PDF fallback OCR
    HybridIntelligent,                            // Pipeline hybride
}

/// Métadonnées par chunk
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct ChunkMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub language: String,
    pub symbol: Option<String>, // Nom fonction/classe si AST
    pub context: Option<String>, // Contexte parent (imports, etc.)
    pub confidence: f32, // Score qualité du chunking
    // === Métadonnées OCR - Phase 1 ===
    pub ocr_metadata: Option<crate::rag::ocr::OcrMetadata>,
    pub source_type: SourceType,
    pub extraction_method: ExtractionMethod,
}

/// Métadonnées document enrichies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub description: Option<String>,
    pub author: Option<String>,
    pub project: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

// === Error Types ===

#[derive(Debug, thiserror::Error)]
pub enum RagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
}

pub type RagResult<T> = Result<T, RagError>;

// === Utils ===

impl DocumentGroup {
    /// Créer un nouveau groupe avec configuration par défaut
    pub fn new(name: String) -> Self {
        let id = format!("group_{}", uuid::Uuid::new_v4().simple());
        let now = SystemTime::now();

        Self {
            id: id.clone(),
            name,
            active: true,
            chunk_config: ChunkConfig::default(),
            metadata_config: MetadataConfig::default(),
            documents: Vec::new(),
            qdrant_collection: format!("collection_{}", id),
            created_at: now,
            updated_at: now,
        }
    }

    /// Créer un nouveau groupe avec un ID spécifique (pour groupes prédéfinis comme "default_group")
    pub fn new_with_id(id: String, name: String) -> Self {
        let now = SystemTime::now();

        Self {
            id: id.clone(),
            name,
            active: true,
            chunk_config: ChunkConfig::default(),
            metadata_config: MetadataConfig::default(),
            documents: Vec::new(),
            qdrant_collection: format!("collection_{}", id),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Vérifier si le groupe peut être utilisé pour les requêtes
    pub fn is_ready(&self) -> bool {
        self.active && !self.documents.is_empty()
    }
}

impl EnrichedChunk {
    /// Générer un hash pour le cache d'embeddings
    pub fn generate_hash(&mut self) {
        let content = format!("{}{}{}{}", 
            self.content, self.group_id, self.chunk_type as u8, self.metadata.language);
        self.hash = blake3::hash(content.as_bytes()).to_hex().to_string();
    }
}

// === Tests unitaires (Phase 1) ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_group_creation() {
        let group = DocumentGroup::new("Test Group".to_string());
        assert_eq!(group.name, "Test Group");
        assert!(group.active);
        assert!(!group.is_ready()); // Pas de documents
    }

    #[test]
    fn test_chunk_config_default() {
        let config = ChunkConfig::default();
        assert_eq!(config.chunk_size, 384);
        assert_eq!(config.overlap, 48);
    }

    #[test]
    fn test_chunk_hash_generation() {
        let mut chunk = EnrichedChunk {
            id: "test".to_string(),
            content: "test content".to_string(),
            start_line: 1,
            end_line: 5,
            chunk_type: ChunkType::Function,
            embedding: None,
            hash: String::new(),
            metadata: ChunkMetadata {
                tags: vec!["test".to_string()],
                priority: Priority::Normal,
                language: "rust".to_string(),
                symbol: None,
                context: None,
                confidence: 1.0,
                ocr_metadata: None,
                source_type: SourceType::NativeText,
                extraction_method: ExtractionMethod::DirectRead,
            },
            group_id: "group1".to_string(),
            source_spans: None,
        };
        
        chunk.generate_hash();
        assert!(!chunk.hash.is_empty());
        assert_eq!(chunk.hash.len(), 64); // blake3 hex = 64 chars
    }
}