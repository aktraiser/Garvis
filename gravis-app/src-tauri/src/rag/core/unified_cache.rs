// Unified Cache - Phase 2 Intégration OCR-RAG
// Cache multi-niveaux: OCR → Embeddings → Documents

use anyhow::Result;
use dashmap::DashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tracing::{debug, info};
use blake3::Hasher;

use crate::rag::{
    GroupDocument, ChunkConfig, RagResult, RagError
};
use super::ingestion_engine::CacheStats;
use crate::rag::ocr::OcrCache;

/// Cache unifié multi-niveaux pour pipeline OCR-RAG
pub struct UnifiedCache {
    // Cache OCR existant
    #[allow(dead_code)]
    ocr_cache: OcrCache,
    
    // Cache embeddings (par hash de chunk)
    embedding_cache: Arc<DashMap<String, Vec<f32>>>,
    
    // Cache documents (par hash de fichier + config)
    document_cache: Arc<DashMap<String, CachedDocument>>,
    
    // Statistiques temps réel
    stats: Arc<DashMap<String, CacheStats>>,
}

/// Document en cache avec métadonnées
#[derive(Debug, Clone)]
pub struct CachedDocument {
    pub document: GroupDocument,
    pub file_hash: String,
    pub config_hash: String,
    pub cached_at: SystemTime,
    pub file_modified: SystemTime,
}

impl UnifiedCache {
    /// Initialise le cache unifié
    pub fn new(ocr_cache: OcrCache) -> Self {
        Self {
            ocr_cache,
            embedding_cache: Arc::new(DashMap::new()),
            document_cache: Arc::new(DashMap::new()),
            stats: Arc::new(DashMap::new()),
        }
    }

    /// Récupère ou traite un document avec cache intelligent
    pub async fn get_or_process_document<F, Fut>(
        &self,
        file_path: &Path,
        group_id: &str,
        config: &ChunkConfig,
        processor: F,
    ) -> RagResult<(GroupDocument, CacheStats)>
    where
        F: FnOnce(&Path, &str, &ChunkConfig) -> Fut,
        Fut: std::future::Future<Output = RagResult<GroupDocument>>,
    {
        let cache_key = self.generate_document_cache_key(file_path, config)?;
        let mut stats = CacheStats::default();
        stats.total_cache_requests += 1;

        debug!("Cache lookup for document: {:?}", file_path);

        // 1. Vérifier le cache document
        if let Some(cached_doc) = self.document_cache.get(&cache_key) {
            // Vérifier si le fichier n'a pas été modifié
            if let Ok(metadata) = tokio::fs::metadata(file_path).await {
                let file_modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                
                if file_modified <= cached_doc.file_modified {
                    debug!("Document cache HIT: {:?}", file_path);
                    stats.document_cache_hits += 1;
                    
                    // Mettre à jour les stats globales
                    self.update_global_stats(&stats);
                    
                    return Ok((cached_doc.document.clone(), stats));
                } else {
                    info!("Document cache STALE (file modified): {:?}", file_path);
                    self.document_cache.remove(&cache_key);
                }
            }
        }

        debug!("Document cache MISS: {:?}", file_path);

        // 2. Traitement avec cache OCR/Embeddings
        let document = processor(file_path, group_id, config).await?;

        // 3. Cache le document traité
        let file_metadata = tokio::fs::metadata(file_path).await
            .map_err(|e| RagError::Io(e))?;
        let file_modified = file_metadata.modified().unwrap_or(UNIX_EPOCH);

        let cached_document = CachedDocument {
            document: document.clone(),
            file_hash: self.calculate_file_hash(file_path).await?,
            config_hash: self.calculate_config_hash(config),
            cached_at: SystemTime::now(),
            file_modified,
        };

        self.document_cache.insert(cache_key, cached_document);
        info!("Document cached: {:?}", file_path);

        // Mettre à jour les stats globales
        self.update_global_stats(&stats);

        Ok((document, stats))
    }

    /// Cache un embedding par hash de chunk
    pub fn cache_embedding(&self, chunk_hash: &str, embedding: Vec<f32>) {
        debug!("Caching embedding for chunk: {}", chunk_hash);
        self.embedding_cache.insert(chunk_hash.to_string(), embedding);
    }

    /// Récupère un embedding depuis le cache
    pub fn get_cached_embedding(&self, chunk_hash: &str) -> Option<Vec<f32>> {
        self.embedding_cache.get(chunk_hash).map(|entry| entry.clone())
    }

    /// Génère la clé de cache pour un document
    fn generate_document_cache_key(&self, file_path: &Path, config: &ChunkConfig) -> RagResult<String> {
        let path_str = file_path.to_string_lossy();
        let config_hash = self.calculate_config_hash(config);
        
        let mut hasher = Hasher::new();
        hasher.update(path_str.as_bytes());
        hasher.update(config_hash.as_bytes());
        
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Calcule le hash d'un fichier
    async fn calculate_file_hash(&self, file_path: &Path) -> RagResult<String> {
        let content = tokio::fs::read(file_path).await
            .map_err(|e| RagError::Io(e))?;
        
        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }

    /// Calcule le hash d'une configuration
    fn calculate_config_hash(&self, config: &ChunkConfig) -> String {
        let config_str = format!("{:?}", config);
        blake3::hash(config_str.as_bytes()).to_hex().to_string()
    }

    /// Met à jour les statistiques globales
    fn update_global_stats(&self, stats: &CacheStats) {
        let global_key = "global".to_string();
        
        self.stats.entry(global_key).and_modify(|global| {
            global.ocr_cache_hits += stats.ocr_cache_hits;
            global.embedding_cache_hits += stats.embedding_cache_hits;
            global.document_cache_hits += stats.document_cache_hits;
            global.total_cache_requests += stats.total_cache_requests;
        }).or_insert_with(|| stats.clone());
    }

    /// Récupère les statistiques globales
    pub fn get_global_stats(&self) -> CacheStats {
        self.stats.get("global")
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    /// Récupère les statistiques par groupe
    pub fn get_group_stats(&self, group_id: &str) -> CacheStats {
        self.stats.get(group_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    /// Nettoie le cache selon des critères
    pub async fn cleanup_cache(&self, max_entries: usize, max_age_hours: u64) -> Result<CacheCleanupResult> {
        let start_time = std::time::Instant::now();
        let mut removed_documents = 0;
        let mut removed_embeddings = 0;

        let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
        let now = SystemTime::now();

        // Nettoyage cache documents
        let docs_to_remove: Vec<String> = self.document_cache
            .iter()
            .filter_map(|entry| {
                let (key, cached_doc) = entry.pair();
                if let Ok(age) = now.duration_since(cached_doc.cached_at) {
                    if age > max_age {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    Some(key.clone()) // Supprimer si durée invalide
                }
            })
            .collect();

        for key in docs_to_remove {
            self.document_cache.remove(&key);
            removed_documents += 1;
        }

        // Limitation par nombre d'entrées si nécessaire
        if self.document_cache.len() > max_entries {
            let excess = self.document_cache.len() - max_entries;
            let keys_to_remove: Vec<String> = self.document_cache
                .iter()
                .take(excess)
                .map(|entry| entry.key().clone())
                .collect();
            
            for key in keys_to_remove {
                self.document_cache.remove(&key);
                removed_documents += 1;
            }
        }

        // Nettoyage cache embeddings (plus conservateur)
        if self.embedding_cache.len() > max_entries * 10 {
            let excess = self.embedding_cache.len() - (max_entries * 10);
            let keys_to_remove: Vec<String> = self.embedding_cache
                .iter()
                .take(excess)
                .map(|entry| entry.key().clone())
                .collect();
            
            for key in keys_to_remove {
                self.embedding_cache.remove(&key);
                removed_embeddings += 1;
            }
        }

        let cleanup_time = start_time.elapsed();
        
        info!("Cache cleanup completed: {} docs, {} embeddings removed in {:?}",
              removed_documents, removed_embeddings, cleanup_time);

        Ok(CacheCleanupResult {
            removed_documents,
            removed_embeddings,
            cleanup_time_ms: cleanup_time.as_millis() as u64,
        })
    }

    /// Invalide le cache pour un fichier spécifique
    pub fn invalidate_file_cache(&self, file_path: &Path) {
        let path_str = file_path.to_string_lossy();
        
        // Supprimer toutes les entrées de cache pour ce fichier
        let keys_to_remove: Vec<String> = self.document_cache
            .iter()
            .filter_map(|entry| {
                let (key, _) = entry.pair();
                if key.contains(path_str.as_ref()) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.document_cache.remove(&key);
            debug!("Invalidated cache entry: {}", key);
        }
    }

    /// Obtient des métriques détaillées du cache
    pub fn get_cache_metrics(&self) -> CacheMetrics {
        CacheMetrics {
            document_cache_size: self.document_cache.len(),
            embedding_cache_size: self.embedding_cache.len(),
            ocr_cache_size: 0, // TODO: Implémenter len() pour OcrCache
            global_stats: self.get_global_stats(),
            memory_usage_estimate: self.estimate_memory_usage(),
        }
    }

    /// Estime l'utilisation mémoire du cache
    fn estimate_memory_usage(&self) -> usize {
        // Estimation approximative
        let doc_cache_size = self.document_cache.len() * 1024; // ~1KB par document en moyenne
        let embedding_cache_size = self.embedding_cache.len() * 384 * 4; // 384D float32
        let ocr_cache_size = 0; // TODO: Implémenter len() pour OcrCache
        
        doc_cache_size + embedding_cache_size + ocr_cache_size
    }
}

/// Résultat du nettoyage de cache
#[derive(Debug)]
pub struct CacheCleanupResult {
    pub removed_documents: usize,
    pub removed_embeddings: usize,
    pub cleanup_time_ms: u64,
}

/// Métriques détaillées du cache
#[derive(Debug)]
pub struct CacheMetrics {
    pub document_cache_size: usize,
    pub embedding_cache_size: usize,
    pub ocr_cache_size: usize,
    pub global_stats: CacheStats,
    pub memory_usage_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ocr::{CacheConfig, OcrConfig};
    use tempfile::NamedTempFile;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_unified_cache_creation() {
        let cache_config = CacheConfig::default();
        let ocr_config = OcrConfig::default();
        let ocr_cache = OcrCache::new(cache_config, ocr_config).unwrap();
        
        let unified_cache = UnifiedCache::new(ocr_cache);
        let metrics = unified_cache.get_cache_metrics();
        
        assert_eq!(metrics.document_cache_size, 0);
        assert_eq!(metrics.embedding_cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let cache_config = CacheConfig::default();
        let ocr_config = OcrConfig::default();
        let ocr_cache = OcrCache::new(cache_config, ocr_config).unwrap();
        let unified_cache = UnifiedCache::new(ocr_cache);

        let temp_file = NamedTempFile::new().unwrap();
        let chunk_config = ChunkConfig::default();
        
        let key1 = unified_cache.generate_document_cache_key(temp_file.path(), &chunk_config).unwrap();
        let key2 = unified_cache.generate_document_cache_key(temp_file.path(), &chunk_config).unwrap();
        
        assert_eq!(key1, key2, "Same file and config should generate same key");
        
        let different_config = ChunkConfig {
            chunk_size: 256,
            overlap: 32,
            strategy: crate::rag::ChunkStrategy::Heuristic,
        };
        
        let key3 = unified_cache.generate_document_cache_key(temp_file.path(), &different_config).unwrap();
        assert_ne!(key1, key3, "Different config should generate different key");
    }

    #[tokio::test]
    async fn test_embedding_cache() {
        let cache_config = CacheConfig::default();
        let ocr_config = OcrConfig::default();
        let ocr_cache = OcrCache::new(cache_config, ocr_config).unwrap();
        let unified_cache = UnifiedCache::new(ocr_cache);

        let chunk_hash = "test_chunk_hash";
        let embedding = vec![1.0, 2.0, 3.0, 4.0];
        
        // Cache miss initial
        assert!(unified_cache.get_cached_embedding(chunk_hash).is_none());
        
        // Cache l'embedding
        unified_cache.cache_embedding(chunk_hash, embedding.clone());
        
        // Cache hit
        let cached = unified_cache.get_cached_embedding(chunk_hash).unwrap();
        assert_eq!(cached, embedding);
    }
}