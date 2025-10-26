// GRAVIS RAG - EmbeddingBatcher pour traitement par lots optimisé
// Phase 3: Traitement asynchrone avec back-pressure et monitoring

use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{info, warn, debug, error};

use super::{E5Embedder, EnrichedChunk, OptimizedQdrantClient, EmbeddingPoint};

/// Configuration pour le batcher d'embeddings
#[derive(Debug, Clone)]
pub struct EmbeddingBatcherConfig {
    pub max_batch_size: usize,
    pub max_queue_size: usize,
    pub batch_timeout: Duration,
    pub max_concurrent_batches: usize,
    pub retry_attempts: usize,
    pub retry_delay: Duration,
}

impl Default for EmbeddingBatcherConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 64,                        // Limite mémoire optimisée
            max_queue_size: 1000,                      // Buffer pour pics de charge
            batch_timeout: Duration::from_millis(500), // Latence acceptable
            max_concurrent_batches: 4,                 // Parallélisme contrôlé
            retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
        }
    }
}

/// Job d'embedding avec callback de completion
#[derive(Debug)]
pub struct EmbeddingJob {
    pub chunk: EnrichedChunk,
    pub collection: String,
    pub completion_tx: Option<mpsc::UnboundedSender<Result<String>>>,
    pub created_at: Instant,
}

/// Statistiques du batcher
#[derive(Debug, Clone)]
pub struct BatcherStats {
    pub queue_size: usize,
    pub processed_total: u64,
    pub failed_total: u64,
    pub avg_batch_size: f32,
    pub avg_processing_time_ms: f32,
    pub active_batches: usize,
}

/// Batcher d'embeddings avec traitement asynchrone optimisé
pub struct EmbeddingBatcher {
    config: EmbeddingBatcherConfig,
    embedder: Arc<E5Embedder>,
    qdrant_client: Arc<OptimizedQdrantClient>,
    
    // Queue thread-safe pour les jobs
    job_queue: Arc<Mutex<VecDeque<EmbeddingJob>>>,
    
    // Contrôle de concurrence
    semaphore: Arc<Semaphore>,
    
    // Statistiques
    stats: Arc<Mutex<BatcherStats>>,
    
    // Contrôle du lifecycle
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

impl EmbeddingBatcher {
    /// Créer un nouveau batcher avec configuration optimisée
    pub fn new(
        config: EmbeddingBatcherConfig,
        embedder: Arc<E5Embedder>,
        qdrant_client: Arc<OptimizedQdrantClient>,
    ) -> Self {
        info!("🔄 Creating EmbeddingBatcher with batch size: {}", config.max_batch_size);
        
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_batches));
        
        let queue_capacity = config.max_queue_size;
        
        Self {
            job_queue: Arc::new(Mutex::new(VecDeque::with_capacity(queue_capacity))),
            config,
            embedder,
            qdrant_client,
            semaphore,
            stats: Arc::new(Mutex::new(BatcherStats {
                queue_size: 0,
                processed_total: 0,
                failed_total: 0,
                avg_batch_size: 0.0,
                avg_processing_time_ms: 0.0,
                active_batches: 0,
            })),
            shutdown_tx: None,
        }
    }
    
    /// Démarrer le traitement asynchrone par lots
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting EmbeddingBatcher background processing");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);
        
        // Cloner les composants pour les tâches async
        let job_queue = Arc::clone(&self.job_queue);
        let embedder = Arc::clone(&self.embedder);
        let qdrant_client = Arc::clone(&self.qdrant_client);
        let semaphore = Arc::clone(&self.semaphore);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();
        
        // Tâche de traitement principal
        tokio::spawn(async move {
            let mut batch_interval = interval(config.batch_timeout);
            
            loop {
                tokio::select! {
                    _ = batch_interval.tick() => {
                        if let Err(e) = Self::process_batch_if_ready(
                            &job_queue,
                            &embedder,
                            &qdrant_client,
                            &semaphore,
                            &stats,
                            &config,
                        ).await {
                            error!("Batch processing error: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("🛑 Shutting down EmbeddingBatcher");
                        break;
                    }
                }
            }
        });
        
        // Tâche de monitoring des stats
        let stats_clone = Arc::clone(&self.stats);
        tokio::spawn(async move {
            let mut stats_interval = interval(Duration::from_secs(30));
            
            loop {
                stats_interval.tick().await;
                Self::log_stats(&stats_clone).await;
            }
        });
        
        info!("✅ EmbeddingBatcher started successfully");
        Ok(())
    }
    
    /// Ajouter un chunk à traiter (non-bloquant)
    pub async fn submit_chunk(
        &self,
        chunk: EnrichedChunk,
        collection: String,
    ) -> Result<mpsc::UnboundedReceiver<Result<String>>> {
        let (completion_tx, completion_rx) = mpsc::unbounded_channel();
        
        let job = EmbeddingJob {
            chunk,
            collection,
            completion_tx: Some(completion_tx),
            created_at: Instant::now(),
        };
        
        // Vérifier la limite de queue
        {
            let mut queue = self.job_queue.lock().await;
            if queue.len() >= self.config.max_queue_size {
                return Err(anyhow::anyhow!(
                    "Embedding queue is full ({}), dropping job", 
                    self.config.max_queue_size
                ));
            }
            
            queue.push_back(job);
            
            // Mettre à jour les stats
            if let Ok(mut stats) = self.stats.try_lock() {
                stats.queue_size = queue.len();
            }
        }
        
        debug!("📥 Job submitted to embedding queue");
        Ok(completion_rx)
    }
    
    /// Traiter un lot si prêt (logique interne)
    async fn process_batch_if_ready(
        job_queue: &Arc<Mutex<VecDeque<EmbeddingJob>>>,
        embedder: &Arc<E5Embedder>,
        qdrant_client: &Arc<OptimizedQdrantClient>,
        semaphore: &Arc<Semaphore>,
        stats: &Arc<Mutex<BatcherStats>>,
        config: &EmbeddingBatcherConfig,
    ) -> Result<()> {
        // Extraire un batch de la queue
        let batch = {
            let mut queue = job_queue.lock().await;
            if queue.is_empty() {
                return Ok(());
            }
            
            let batch_size = config.max_batch_size.min(queue.len());
            let mut batch = Vec::with_capacity(batch_size);
            
            for _ in 0..batch_size {
                if let Some(job) = queue.pop_front() {
                    batch.push(job);
                }
            }
            
            // Mettre à jour la taille de queue
            if let Ok(mut stats) = stats.try_lock() {
                stats.queue_size = queue.len();
            }
            
            batch
        };
        
        if !batch.is_empty() {
            // Acquérir un slot de concurrence
            let _permit = semaphore.acquire().await.context("Failed to acquire semaphore")?;
            
            // Traiter le batch en arrière-plan
            let embedder_clone = Arc::clone(embedder);
            let qdrant_client_clone = Arc::clone(qdrant_client);
            let stats_clone = Arc::clone(stats);
            let config_clone = config.clone();
            
            tokio::spawn(async move {
                Self::process_batch(
                    batch,
                    &embedder_clone,
                    &qdrant_client_clone,
                    &stats_clone,
                    &config_clone,
                ).await;
            });
        }
        
        Ok(())
    }
    
    /// Traiter un batch complet (avec retry et monitoring)
    async fn process_batch(
        batch: Vec<EmbeddingJob>,
        embedder: &E5Embedder,
        qdrant_client: &OptimizedQdrantClient,
        stats: &Arc<Mutex<BatcherStats>>,
        config: &EmbeddingBatcherConfig,
    ) {
        let batch_start = Instant::now();
        let batch_size = batch.len();
        
        debug!("🔄 Processing batch of {} chunks", batch_size);
        
        // Incrémenter les batches actifs
        if let Ok(mut stats) = stats.try_lock() {
            stats.active_batches += 1;
        }
        
        // Grouper par collection pour optimiser
        let mut collections: std::collections::HashMap<String, Vec<EmbeddingJob>> = 
            std::collections::HashMap::new();
        
        for job in batch {
            collections.entry(job.collection.clone()).or_default().push(job);
        }
        
        // Traiter chaque collection
        for (collection, mut jobs) in collections {
            let mut retry_count = 0;
            let jobs_len = jobs.len();
            
            loop {
                match Self::process_collection_batch(&collection, &jobs, embedder, qdrant_client).await {
                    Ok(_) => {
                        // Notifier le succès pour tous les jobs
                        for job in jobs.drain(..) {
                            if let Some(tx) = job.completion_tx {
                                let _ = tx.send(Ok(job.chunk.id));
                            }
                        }
                        
                        // Mettre à jour les stats de succès
                        if let Ok(mut stats) = stats.try_lock() {
                            stats.processed_total += jobs_len as u64;
                        }
                        
                        break;
                    }
                    Err(e) => {
                        retry_count += 1;
                        
                        if retry_count >= config.retry_attempts {
                            error!("❌ Batch failed after {} retries: {}", config.retry_attempts, e);
                            
                            // Notifier l'échec pour tous les jobs
                            for job in jobs.drain(..) {
                                if let Some(tx) = job.completion_tx {
                                    let _ = tx.send(Err(anyhow::anyhow!("Batch processing failed: {}", e)));
                                }
                            }
                            
                            // Mettre à jour les stats d'échec
                            if let Ok(mut stats) = stats.try_lock() {
                                stats.failed_total += jobs_len as u64;
                            }
                            
                            break;
                        } else {
                            warn!("⚠️ Batch failed, retrying ({}/{}): {}", retry_count, config.retry_attempts, e);
                            sleep(config.retry_delay * retry_count as u32).await;
                        }
                    }
                }
            }
        }
        
        // Mettre à jour les statistiques de performance
        let processing_time = batch_start.elapsed();
        if let Ok(mut stats) = stats.try_lock() {
            stats.active_batches = stats.active_batches.saturating_sub(1);
            
            // Moyennes mobiles simplifiées
            let total_processed = stats.processed_total + stats.failed_total;
            if total_processed > 0 {
                stats.avg_batch_size = (stats.avg_batch_size * (total_processed - batch_size as u64) as f32 
                    + batch_size as f32) / total_processed as f32;
                stats.avg_processing_time_ms = (stats.avg_processing_time_ms * (total_processed - batch_size as u64) as f32 
                    + processing_time.as_millis() as f32) / total_processed as f32;
            }
        }
        
        debug!("✅ Batch processed in {:?}", processing_time);
    }
    
    /// Traiter les chunks d'une collection spécifique
    async fn process_collection_batch(
        collection: &str,
        jobs: &[EmbeddingJob],
        embedder: &E5Embedder,
        qdrant_client: &OptimizedQdrantClient,
    ) -> Result<()> {
        // Générer les embeddings en batch
        let texts: Vec<String> = jobs.iter().map(|job| job.chunk.content.clone()).collect();
        let embeddings = embedder.encode_batch(&texts).await
            .context("Failed to generate embeddings for batch")?;
        
        // Créer les points d'embedding
        let embedding_points: Vec<EmbeddingPoint> = jobs.iter()
            .zip(embeddings.iter())
            .map(|(job, embedding)| {
                let mut chunk = job.chunk.clone();
                chunk.embedding = Some(embedding.clone());
                EmbeddingPoint::from(&chunk)
            })
            .collect();
        
        // Upsert en Qdrant
        qdrant_client.batch_upsert_embeddings(collection, embedding_points).await
            .context("Failed to upsert embeddings to Qdrant")?;
        
        Ok(())
    }
    
    /// Logger les statistiques périodiquement
    async fn log_stats(stats: &Arc<Mutex<BatcherStats>>) {
        if let Ok(stats) = stats.try_lock() {
            info!(
                "📊 EmbeddingBatcher Stats - Queue: {}, Processed: {}, Failed: {}, Avg Batch: {:.1}, Avg Time: {:.1}ms, Active: {}",
                stats.queue_size,
                stats.processed_total,
                stats.failed_total,
                stats.avg_batch_size,
                stats.avg_processing_time_ms,
                stats.active_batches
            );
        }
    }
    
    /// Obtenir les statistiques actuelles
    pub async fn get_stats(&self) -> BatcherStats {
        let stats = self.stats.lock().await;
        stats.clone()
    }
    
    /// Arrêter le batcher proprement
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
            info!("🛑 EmbeddingBatcher shutdown signal sent");
        }
        Ok(())
    }
    
    /// Attendre que la queue soit vide (pour tests)
    pub async fn wait_for_empty_queue(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            let queue_size = {
                let queue = self.job_queue.lock().await;
                queue.len()
            };
            
            if queue_size == 0 {
                return Ok(());
            }
            
            sleep(Duration::from_millis(50)).await;
        }
        
        Err(anyhow::anyhow!("Timeout waiting for empty queue"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{ChunkType, ChunkMetadata, Priority};
    
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
    fn test_batcher_config() {
        let config = EmbeddingBatcherConfig::default();
        assert_eq!(config.max_batch_size, 64);
        assert_eq!(config.max_queue_size, 1000);
        println!("✅ EmbeddingBatcher config validation passed");
    }
    
    #[test]
    fn test_embedding_job_creation() {
        let chunk = create_test_chunk("test_id", "test content");
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let job = EmbeddingJob {
            chunk,
            collection: "test_collection".to_string(),
            completion_tx: Some(tx),
            created_at: Instant::now(),
        };
        
        assert_eq!(job.collection, "test_collection");
        assert_eq!(job.chunk.id, "test_id");
        println!("✅ EmbeddingJob creation working");
    }
}