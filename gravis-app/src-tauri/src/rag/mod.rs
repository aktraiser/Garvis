// GRAVIS RAG Module - Phase 2: Architecture Complete avec Embeddings
// Architecture modulaire sécurisée pour préserver l'intégrité de l'application

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

// === Modules Phase 2 & 3 ===
pub mod embedder;
pub mod custom_e5;
pub mod qdrant;
pub mod qdrant_rest;
pub mod device_pool;
pub mod embedding_batcher;
pub mod document_sync_manager;
// pub mod benchmark; // Désactivé temporairement pour les erreurs

// Phase 2 exports
pub use embedder::{E5Config, E5Embedder};
pub use custom_e5::{CustomE5Config, CustomE5Embedder};

// Phase 3 exports
pub use qdrant::{
    OptimizedQdrantClient, QdrantConfig, EmbeddingPoint, EmbeddingPayload,
    SearchFilters, SearchResult, CollectionStats
};
pub use qdrant_rest::{
    QdrantRestClient, QdrantRestConfig, RestPoint, RestSearchResponse
};
pub use device_pool::{
    DevicePool, DevicePoolConfig, DevicePoolStats, GlobalDevicePool
};
pub use embedding_batcher::{
    EmbeddingBatcher, EmbeddingBatcherConfig, EmbeddingJob, BatcherStats
};
pub use document_sync_manager::{
    DocumentSyncManager, SyncManagerConfig, SyncMetadata, SyncStatus, SyncStats
};
// pub use benchmark::{
//     RagBenchmark, BenchmarkConfig, BenchmarkResults, run_benchmark_cli
// };

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
            chunk_size: 512,
            overlap: 64,
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
}

/// Type de document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    SourceCode { language: String },
    PDF { has_text: bool, pages: usize },
    Image { ocr_confidence: f32 },
    Markdown,
    PlainText,
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

/// Métadonnées par chunk
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct ChunkMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub language: String,
    pub symbol: Option<String>, // Nom fonction/classe si AST
    pub context: Option<String>, // Contexte parent (imports, etc.)
    pub confidence: f32, // Score qualité du chunking
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
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.overlap, 64);
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
            },
            group_id: "group1".to_string(),
        };
        
        chunk.generate_hash();
        assert!(!chunk.hash.is_empty());
        assert_eq!(chunk.hash.len(), 64); // blake3 hex = 64 chars
    }
}