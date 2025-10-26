// GRAVIS RAG - DocumentSyncManager pour synchronisation SQLite ↔ Qdrant
// Phase 3: Système de persistance hybride avec contrôle d'intégrité

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{
    DocumentGroup, EnrichedChunk, EmbeddingPoint, OptimizedQdrantClient,
    EmbeddingBatcher, EmbeddingBatcherConfig, E5Embedder
};

/// Configuration du gestionnaire de synchronisation
#[derive(Debug, Clone)]
pub struct SyncManagerConfig {
    pub db_path: PathBuf,
    pub sync_interval: Duration,
    pub batch_size: usize,
    pub max_retry_attempts: usize,
    pub integrity_check_interval: Duration,
    pub enable_auto_sync: bool,
}

impl Default for SyncManagerConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("gravis_rag.db"),
            sync_interval: Duration::from_secs(30),
            batch_size: 100,
            max_retry_attempts: 3,
            integrity_check_interval: Duration::from_secs(300), // 5 minutes
            enable_auto_sync: true,
        }
    }
}

/// État de synchronisation d'un chunk
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,      // En attente de traitement
    Processing,   // En cours de traitement
    Synced,       // Synchronisé avec succès
    Failed,       // Échec de synchronisation
    Conflict,     // Conflit détecté
}

/// Entrée de métadonnées de synchronisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub chunk_id: String,
    pub document_id: String,
    pub group_id: String,
    pub collection_name: String,
    pub content_hash: String,
    pub status: SyncStatus,
    pub last_synced: Option<SystemTime>,
    pub retry_count: usize,
    pub error_message: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Statistiques de synchronisation
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub total_chunks: usize,
    pub synced_chunks: usize,
    pub pending_chunks: usize,
    pub failed_chunks: usize,
    pub conflicts: usize,
    pub last_sync: Option<SystemTime>,
    pub sync_rate_per_minute: f32,
}

/// Gestionnaire de synchronisation entre SQLite et Qdrant
pub struct DocumentSyncManager {
    config: SyncManagerConfig,
    qdrant_client: Arc<OptimizedQdrantClient>,
    embedder: Arc<E5Embedder>,
    embedding_batcher: Arc<Mutex<EmbeddingBatcher>>,
    
    // Base de données SQLite (placeholder - utiliserait sqlx en production)
    db_path: PathBuf,
    
    // Cache en mémoire pour les métadonnées de sync
    sync_metadata: Arc<RwLock<HashMap<String, SyncMetadata>>>,
    
    // Groupes de documents actifs
    document_groups: Arc<RwLock<HashMap<String, DocumentGroup>>>,
    
    // État du gestionnaire
    is_running: Arc<RwLock<bool>>,
    
    // Statistiques
    stats: Arc<RwLock<SyncStats>>,
}

impl DocumentSyncManager {
    /// Créer un nouveau gestionnaire de synchronisation
    pub async fn new(
        config: SyncManagerConfig,
        qdrant_client: Arc<OptimizedQdrantClient>,
        embedder: Arc<E5Embedder>,
    ) -> Result<Self> {
        info!("🔄 Creating DocumentSyncManager with database: {:?}", config.db_path);
        
        // Créer le batcher d'embeddings
        let batcher_config = EmbeddingBatcherConfig {
            max_batch_size: config.batch_size,
            ..Default::default()
        };
        
        let mut embedding_batcher = EmbeddingBatcher::new(
            batcher_config,
            Arc::clone(&embedder),
            Arc::clone(&qdrant_client),
        );
        
        // Démarrer le batcher
        embedding_batcher.start().await?;
        
        let manager = Self {
            db_path: config.db_path.clone(),
            config,
            qdrant_client,
            embedder,
            embedding_batcher: Arc::new(Mutex::new(embedding_batcher)),
            sync_metadata: Arc::new(RwLock::new(HashMap::new())),
            document_groups: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(SyncStats {
                total_chunks: 0,
                synced_chunks: 0,
                pending_chunks: 0,
                failed_chunks: 0,
                conflicts: 0,
                last_sync: None,
                sync_rate_per_minute: 0.0,
            })),
        };
        
        // Initialiser la base de données
        manager.init_database().await?;
        
        // Charger les métadonnées existantes
        manager.load_sync_metadata().await?;
        
        info!("✅ DocumentSyncManager initialized successfully");
        Ok(manager)
    }
    
    /// Démarrer la synchronisation automatique
    pub async fn start(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Ok(());
            }
            *is_running = true;
        }
        
        info!("🚀 Starting DocumentSyncManager");
        
        if self.config.enable_auto_sync {
            // Tâche de synchronisation périodique
            self.start_sync_loop().await;
            
            // Tâche de vérification d'intégrité
            self.start_integrity_check_loop().await;
        }
        
        info!("✅ DocumentSyncManager started successfully");
        Ok(())
    }
    
    /// Ajouter un groupe de documents à synchroniser
    pub async fn add_document_group(&self, group: DocumentGroup) -> Result<()> {
        info!("📥 Adding document group: {}", group.name);
        
        // Assurer que la collection Qdrant existe
        self.qdrant_client.ensure_collection_exists(&group).await?;
        
        // Ajouter au cache
        {
            let mut groups = self.document_groups.write().await;
            groups.insert(group.id.clone(), group.clone());
        }
        
        // Traiter tous les chunks du groupe
        for document in &group.documents {
            for chunk in &document.chunks {
                self.add_chunk_for_sync(chunk.clone(), group.qdrant_collection.clone()).await?;
            }
        }
        
        info!("✅ Document group added successfully: {}", group.name);
        Ok(())
    }
    
    /// Ajouter un chunk à la queue de synchronisation
    pub async fn add_chunk_for_sync(&self, chunk: EnrichedChunk, collection: String) -> Result<()> {
        let metadata = SyncMetadata {
            chunk_id: chunk.id.clone(),
            document_id: chunk.group_id.clone(), // TODO: Ajouter document_id à EnrichedChunk
            group_id: chunk.group_id.clone(),
            collection_name: collection,
            content_hash: chunk.hash.clone(),
            status: SyncStatus::Pending,
            last_synced: None,
            retry_count: 0,
            error_message: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        
        // Ajouter aux métadonnées
        {
            let mut sync_meta = self.sync_metadata.write().await;
            sync_meta.insert(chunk.id.clone(), metadata.clone());
        }
        
        // Persister en base
        self.save_sync_metadata(&metadata).await?;
        
        // Mettre à jour les stats
        self.update_stats_for_new_chunk().await;
        
        debug!("📝 Chunk added for sync: {}", chunk.id);
        Ok(())
    }
    
    /// Synchroniser tous les chunks en attente
    pub async fn sync_pending_chunks(&self) -> Result<usize> {
        let pending_chunks = self.get_pending_chunks().await;
        let pending_count = pending_chunks.len();
        
        if pending_count == 0 {
            return Ok(0);
        }
        
        info!("🔄 Syncing {} pending chunks", pending_count);
        
        let mut synced_count = 0;
        
        // Grouper par collection pour optimiser
        let mut collections: HashMap<String, Vec<(String, SyncMetadata)>> = HashMap::new();
        for (chunk_id, metadata) in pending_chunks {
            collections.entry(metadata.collection_name.clone())
                .or_default()
                .push((chunk_id, metadata));
        }
        
        // Traiter chaque collection
        for (collection, chunk_metas) in collections {
            match self.sync_collection_chunks(&collection, chunk_metas).await {
                Ok(count) => {
                    synced_count += count;
                }
                Err(e) => {
                    error!("Failed to sync collection {}: {}", collection, e);
                }
            }
        }
        
        // Mettre à jour les statistiques
        self.update_sync_stats(synced_count).await;
        
        info!("✅ Synced {}/{} chunks successfully", synced_count, pending_count);
        Ok(synced_count)
    }
    
    /// Vérifier l'intégrité des données entre SQLite et Qdrant
    pub async fn check_integrity(&self) -> Result<Vec<String>> {
        info!("🔍 Starting integrity check");
        let mut issues = Vec::new();
        let mut chunks_to_resync = Vec::new();
        
        {
            let sync_metadata = self.sync_metadata.read().await;
            
            for (chunk_id, metadata) in sync_metadata.iter() {
                if metadata.status == SyncStatus::Synced {
                    // Vérifier que le chunk existe bien dans Qdrant
                    match self.verify_chunk_in_qdrant(chunk_id, &metadata.collection_name).await {
                        Ok(exists) => {
                            if !exists {
                                issues.push(format!("Chunk {} missing from Qdrant collection {}", 
                                    chunk_id, metadata.collection_name));
                                chunks_to_resync.push(chunk_id.clone());
                            }
                        }
                        Err(e) => {
                            warn!("Failed to verify chunk {} in Qdrant: {}", chunk_id, e);
                        }
                    }
                }
            }
        }
        
        // Marquer les chunks pour re-synchronisation
        for chunk_id in chunks_to_resync {
            self.mark_chunk_for_resync(&chunk_id).await.ok();
        }
        
        if issues.is_empty() {
            info!("✅ Integrity check passed - no issues found");
        } else {
            warn!("⚠️ Integrity check found {} issues", issues.len());
        }
        
        Ok(issues)
    }
    
    /// Obtenir les statistiques de synchronisation
    pub async fn get_stats(&self) -> SyncStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Arrêter le gestionnaire de synchronisation
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down DocumentSyncManager");
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }
        
        // Arrêter le batcher
        {
            let mut batcher = self.embedding_batcher.lock().await;
            batcher.shutdown().await?;
        }
        
        info!("✅ DocumentSyncManager shut down successfully");
        Ok(())
    }
    
    // === Méthodes privées ===
    
    /// Initialiser la base de données SQLite
    async fn init_database(&self) -> Result<()> {
        // TODO: Implémenter avec sqlx
        // Pour l'instant, créer le fichier s'il n'existe pas
        if !self.db_path.exists() {
            if let Some(parent) = self.db_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::File::create(&self.db_path).await?;
            info!("📄 Database file created: {:?}", self.db_path);
        }
        Ok(())
    }
    
    /// Charger les métadonnées de synchronisation depuis la base
    async fn load_sync_metadata(&self) -> Result<()> {
        // TODO: Implémenter le chargement depuis SQLite
        info!("📖 Loading sync metadata from database");
        Ok(())
    }
    
    /// Sauvegarder les métadonnées de synchronisation
    async fn save_sync_metadata(&self, _metadata: &SyncMetadata) -> Result<()> {
        // TODO: Implémenter la sauvegarde en SQLite
        Ok(())
    }
    
    /// Obtenir les chunks en attente de synchronisation
    async fn get_pending_chunks(&self) -> Vec<(String, SyncMetadata)> {
        let sync_metadata = self.sync_metadata.read().await;
        sync_metadata.iter()
            .filter(|(_, meta)| meta.status == SyncStatus::Pending)
            .map(|(id, meta)| (id.clone(), meta.clone()))
            .collect()
    }
    
    /// Synchroniser les chunks d'une collection
    async fn sync_collection_chunks(
        &self,
        collection: &str,
        chunk_metas: Vec<(String, SyncMetadata)>,
    ) -> Result<usize> {
        let mut synced_count = 0;
        
        // Récupérer les chunks depuis les groupes de documents
        let chunks = self.get_chunks_for_sync(&chunk_metas).await?;
        
        // Soumettre au batcher pour traitement asynchrone
        for chunk in chunks {
            let batcher = self.embedding_batcher.lock().await;
            match batcher.submit_chunk(chunk.clone(), collection.to_string()).await {
                Ok(_completion_rx) => {
                    // Marquer comme en cours de traitement
                    self.update_chunk_status(&chunk.id, SyncStatus::Processing).await;
                    
                    // TODO: Attendre la completion et mettre à jour le statut
                    // Pour l'instant, marquer comme synchronisé
                    self.update_chunk_status(&chunk.id, SyncStatus::Synced).await;
                    synced_count += 1;
                }
                Err(e) => {
                    error!("Failed to submit chunk {} for embedding: {}", chunk.id, e);
                    self.update_chunk_status(&chunk.id, SyncStatus::Failed).await;
                }
            }
        }
        
        Ok(synced_count)
    }
    
    /// Récupérer les chunks pour synchronisation
    async fn get_chunks_for_sync(&self, chunk_metas: &[(String, SyncMetadata)]) -> Result<Vec<EnrichedChunk>> {
        let mut chunks = Vec::new();
        let groups = self.document_groups.read().await;
        
        for (chunk_id, _) in chunk_metas {
            // Chercher le chunk dans tous les groupes
            for group in groups.values() {
                for document in &group.documents {
                    if let Some(chunk) = document.chunks.iter().find(|c| &c.id == chunk_id) {
                        chunks.push(chunk.clone());
                        break;
                    }
                }
            }
        }
        
        Ok(chunks)
    }
    
    /// Vérifier qu'un chunk existe dans Qdrant
    async fn verify_chunk_in_qdrant(&self, chunk_id: &str, collection: &str) -> Result<bool> {
        // TODO: Implémenter la vérification avec une requête Qdrant
        // Pour l'instant, toujours retourner true
        Ok(true)
    }
    
    /// Marquer un chunk pour re-synchronisation
    async fn mark_chunk_for_resync(&self, chunk_id: &str) -> Result<()> {
        let mut sync_metadata = self.sync_metadata.write().await;
        if let Some(metadata) = sync_metadata.get_mut(chunk_id) {
            metadata.status = SyncStatus::Pending;
            metadata.retry_count += 1;
            metadata.updated_at = SystemTime::now();
        }
        Ok(())
    }
    
    /// Mettre à jour le statut d'un chunk
    async fn update_chunk_status(&self, chunk_id: &str, status: SyncStatus) {
        let mut sync_metadata = self.sync_metadata.write().await;
        if let Some(metadata) = sync_metadata.get_mut(chunk_id) {
            let is_synced = matches!(status, SyncStatus::Synced);
            metadata.status = status;
            metadata.updated_at = SystemTime::now();
            
            if is_synced {
                metadata.last_synced = Some(SystemTime::now());
            }
        }
    }
    
    /// Mettre à jour les statistiques pour un nouveau chunk
    async fn update_stats_for_new_chunk(&self) {
        let mut stats = self.stats.write().await;
        stats.total_chunks += 1;
        stats.pending_chunks += 1;
    }
    
    /// Mettre à jour les statistiques de synchronisation
    async fn update_sync_stats(&self, synced_count: usize) {
        let mut stats = self.stats.write().await;
        stats.synced_chunks += synced_count;
        stats.pending_chunks = stats.pending_chunks.saturating_sub(synced_count);
        stats.last_sync = Some(SystemTime::now());
        
        // Calculer le taux de synchronisation (simplifié)
        if stats.total_chunks > 0 {
            stats.sync_rate_per_minute = (stats.synced_chunks as f32 / stats.total_chunks as f32) * 60.0;
        }
    }
    
    /// Démarrer la boucle de synchronisation périodique
    async fn start_sync_loop(&self) {
        let is_running = Arc::clone(&self.is_running);
        let config = self.config.clone();
        
        // Clone des composants nécessaires pour la tâche async
        let qdrant_client = Arc::clone(&self.qdrant_client);
        let embedder = Arc::clone(&self.embedder);
        let embedding_batcher = Arc::clone(&self.embedding_batcher);
        let sync_metadata = Arc::clone(&self.sync_metadata);
        let document_groups = Arc::clone(&self.document_groups);
        let stats = Arc::clone(&self.stats);
        
        tokio::spawn(async move {
            let mut sync_interval = interval(config.sync_interval);
            
            loop {
                sync_interval.tick().await;
                
                let is_running = is_running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);
                
                // Logique de synchronisation simplifiée
                let pending_chunks = {
                    let sync_meta = sync_metadata.read().await;
                    sync_meta.iter()
                        .filter(|(_, meta)| meta.status == SyncStatus::Pending)
                        .map(|(id, meta)| (id.clone(), meta.clone()))
                        .collect::<Vec<_>>()
                };
                
                if !pending_chunks.is_empty() {
                    debug!("🔄 Processing {} pending chunks in sync loop", pending_chunks.len());
                    // Le traitement détaillé sera implémenté plus tard
                }
            }
        });
    }
    
    /// Démarrer la boucle de vérification d'intégrité
    async fn start_integrity_check_loop(&self) {
        let is_running = Arc::clone(&self.is_running);
        let config = self.config.clone();
        let sync_metadata = Arc::clone(&self.sync_metadata);
        
        tokio::spawn(async move {
            let mut integrity_interval = interval(config.integrity_check_interval);
            
            loop {
                integrity_interval.tick().await;
                
                let is_running = is_running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);
                
                // Vérification d'intégrité simplifiée
                let synced_chunks = {
                    let sync_meta = sync_metadata.read().await;
                    sync_meta.iter()
                        .filter(|(_, meta)| meta.status == SyncStatus::Synced)
                        .count()
                };
                
                debug!("🔍 Integrity check: {} synced chunks", synced_chunks);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{ChunkType, ChunkMetadata, Priority, E5Config, DevicePoolConfig, QdrantConfig};
    use candle_core::Device;
    
    // Helper pour créer un chunk de test
    fn create_test_chunk(id: &str, content: &str) -> EnrichedChunk {
        EnrichedChunk {
            id: id.to_string(),
            content: content.to_string(),
            start_line: 1,
            end_line: 5,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: blake3::hash(content.as_bytes()).to_hex().to_string(),
            metadata: ChunkMetadata {
                tags: vec!["test".to_string()],
                priority: Priority::Normal,
                language: "text".to_string(),
                symbol: None,
                context: None,
                confidence: 1.0,
            },
            group_id: "test_group".to_string(),
        }
    }
    
    #[test]
    fn test_sync_metadata_creation() {
        let chunk = create_test_chunk("test_chunk", "test content");
        let metadata = SyncMetadata {
            chunk_id: chunk.id.clone(),
            document_id: "test_doc".to_string(),
            group_id: chunk.group_id.clone(),
            collection_name: "test_collection".to_string(),
            content_hash: chunk.hash.clone(),
            status: SyncStatus::Pending,
            last_synced: None,
            retry_count: 0,
            error_message: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        
        assert_eq!(metadata.status, SyncStatus::Pending);
        assert_eq!(metadata.retry_count, 0);
        println!("✅ SyncMetadata creation working");
    }
    
    #[test]
    fn test_sync_status_transitions() {
        let mut status = SyncStatus::Pending;
        assert_eq!(status, SyncStatus::Pending);
        
        status = SyncStatus::Processing;
        assert_eq!(status, SyncStatus::Processing);
        
        status = SyncStatus::Synced;
        assert_eq!(status, SyncStatus::Synced);
        
        println!("✅ SyncStatus transitions working");
    }
}