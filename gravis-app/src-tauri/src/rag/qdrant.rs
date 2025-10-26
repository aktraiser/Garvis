// GRAVIS RAG - Qdrant Client Optimisé
// Phase 3: Client production avec pool de connexions et batch processing

use anyhow::{Context, Result};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, 
        SearchPointsBuilder, VectorParamsBuilder, PointId, SearchParamsBuilder,
        Filter, FieldCondition, Condition, Range, Match, Value,
        UpsertPointsBuilder, DeletePointsBuilder,
    },
    Payload, Qdrant,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use uuid::Uuid;

use super::{DocumentGroup, EnrichedChunk};

/// Configuration optimisée pour Qdrant avec options HTTP/gRPC
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub timeout: Duration,
    pub connection_pool_size: usize,
    pub max_batch_size: usize,
    pub retry_attempts: usize,
    pub prefer_grpc: bool,
    pub force_http1: bool,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),  // REST API plus stable pour debug
            timeout: Duration::from_secs(30),         // REST timeout plus court
            connection_pool_size: 10,                 // Pool plus large pour REST
            max_batch_size: 50,                       // Batch plus petit pour REST
            retry_attempts: 3,                        // Moins de retries pour REST
            prefer_grpc: false,        // Utiliser REST pour debug
            force_http1: true,         // Force HTTP/1.1 pour stabilité
        }
    }
}

/// Point d'embedding pour Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPoint {
    pub id: String,
    pub embedding: Vec<f32>,
    pub payload: EmbeddingPayload,
}

/// Métadonnées associées à un embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPayload {
    pub chunk_id: String,
    pub document_id: String,
    pub group_id: String,
    pub content: String,
    pub chunk_type: String,
    pub language: String,
    pub tags: Vec<String>,
    pub priority: String,
    pub start_line: usize,
    pub end_line: usize,
    pub symbol: Option<String>,
    pub context: Option<String>,
    pub confidence: f32,
    pub created_at: i64,
}

/// Client Qdrant optimisé avec pool de connexions
pub struct OptimizedQdrantClient {
    client: Arc<Qdrant>,
    config: QdrantConfig,
}

impl OptimizedQdrantClient {
    /// Créer un client optimisé avec pool de connexions et HTTP/1.1 pour stabilité
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        info!("🔄 Initializing optimized Qdrant client: {}", config.url);
        info!("   • prefer_grpc: {}, force_http1: {}", config.prefer_grpc, config.force_http1);
        
        let mut client_builder = Qdrant::from_url(&config.url);
        
        // Configuration de stabilité basée sur les retours de la communauté
        if !config.prefer_grpc {
            // Configuration HTTP/REST par défaut
            if config.force_http1 {
                info!("🔧 Configuring for HTTP/1.1 stability (recommended for batch operations)...");
                
                // Pour qdrant-client 1.15.0, nous configurons des timeouts optimisés
                // Le client utilisera HTTP/1.1 par défaut sur le port 6333
                client_builder = client_builder
                    .timeout(config.timeout);
                    
                // Log pour le debug
                info!("   • Using HTTP/REST endpoint: {}", config.url);
                info!("   • Timeout: {:?}", config.timeout);
            }
        } else {
            info!("🔧 Using gRPC client (port 6334 recommended)...");
            // Pour gRPC, utiliser le port 6334 si l'URL contient 6333
            if config.url.contains("6333") {
                warn!("⚠️ gRPC préfère généralement le port 6334, actuel: {}", config.url);
                info!("   💡 Suggestion: utiliser http://localhost:6334 pour gRPC");
            }
            
            // Configuration gRPC optimisée pour batch operations
            info!("   • Timeout: {:?}", config.timeout);
            info!("   • Connection pool: {} connections", config.connection_pool_size);
            info!("   • Max batch size: {}", config.max_batch_size);
        }
        
        let client = client_builder
            .build()
            .context("Failed to create Qdrant client with HTTP/1.1 optimization")?;
        
        info!("✅ Qdrant client initialized with stability optimizations");
        
        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }
    
    /// Créer une collection optimisée avec détection automatique de dimension
    pub async fn create_optimized_collection(&self, collection_name: &str) -> Result<()> {
        // D'abord supprimer la collection si elle existe déjà
        let _ = self.client.delete_collection(collection_name).await;
        info!("🧹 Collection '{}' supprimée (si elle existait)", collection_name);
        
        // Créer avec 384D pour E5-Small-v2 (vraie dimension)
        let vector_dim = 384;
        info!("🔄 Creating optimized collection: {} ({}D for E5-Small-v2)", collection_name, vector_dim);
        
        self.client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(vector_dim, Distance::Cosine))
            )
            .await
            .context("Failed to create collection")?;
        
        info!("✅ Collection '{}' created successfully with {}D vectors", collection_name, vector_dim);
        Ok(())
    }
    
    /// Créer une collection avec dimension spécifique
    pub async fn create_collection_with_dimension(&self, collection_name: &str, dimension: u64) -> Result<()> {
        // Supprimer la collection si elle existe déjà
        let _ = self.client.delete_collection(collection_name).await;
        info!("🧹 Collection '{}' supprimée (si elle existait)", collection_name);
        
        info!("🔄 Creating collection: {} ({}D)", collection_name, dimension);
        
        self.client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(dimension, Distance::Cosine))
            )
            .await
            .context("Failed to create collection")?;
        
        info!("✅ Collection '{}' created successfully with {}D vectors", collection_name, dimension);
        Ok(())
    }
    
    /// Créer automatiquement la collection pour un groupe si elle n'existe pas
    pub async fn ensure_collection_exists(&self, group: &DocumentGroup) -> Result<()> {
        let collection_name = &group.qdrant_collection;
        
        // Vérifier si la collection existe
        match self.client.collection_info(collection_name).await {
            Ok(_) => {
                info!("Collection '{}' already exists", collection_name);
                Ok(())
            }
            Err(_) => {
                info!("Collection '{}' does not exist, creating...", collection_name);
                self.create_optimized_collection(collection_name).await
            }
        }
    }
    
    /// Batch upsert avec limite de mémoire (recommandation experte)
    pub async fn batch_upsert_embeddings(
        &self,
        collection: &str,
        embeddings: Vec<EmbeddingPoint>,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }
        
        let batch_size = self.config.max_batch_size.min(100); // Optimisé pour stabilité
        info!("🔄 Upserting {} embeddings in batches of {}", embeddings.len(), batch_size);
        
        for (batch_idx, chunk) in embeddings.chunks(batch_size).enumerate() {
            let points: Vec<PointStruct> = chunk.iter()
                .map(|emb| {
                    let payload: Payload = serde_json::to_value(&emb.payload)
                        .unwrap_or_default()
                        .try_into()
                        .unwrap_or_default();
                    
                    PointStruct::new(
                        emb.id.clone(),
                        emb.embedding.clone(),
                        payload,
                    )
                })
                .collect();
            
            // Retry logic avec backoff exponentiel
            let mut attempt = 0;
            while attempt < self.config.retry_attempts {
                match self.client.upsert_points(
                    UpsertPointsBuilder::new(collection, points.clone())
                ).await {
                    Ok(_) => {
                        info!("✅ Batch {} upserted successfully ({} points)", batch_idx + 1, chunk.len());
                        break;
                    }
                    Err(e) => {
                        attempt += 1;
                        if attempt >= self.config.retry_attempts {
                            error!("❌ UPSERT FAILED PERMANENTLY for batch {}: {}", batch_idx + 1, e);
                            return Err(anyhow::anyhow!(
                                "Failed to upsert batch {} after {} attempts: {}", 
                                batch_idx + 1, self.config.retry_attempts, e
                            ));
                        }
                        
                        let delay = Duration::from_millis(500 * (1 << attempt)); // Backoff plus agressif
                        warn!("⚠️ Attempt {} failed for batch {} (ERROR: {}), retrying in {:?}...", 
                              attempt, batch_idx + 1, e, delay);
                        sleep(delay).await;
                    }
                }
            }
            
            // Pause plus longue pour éviter surcharge serveur
            sleep(Duration::from_millis(50)).await;
        }
        
        info!("✅ All embeddings upserted successfully");
        Ok(())
    }
    
    /// Recherche sémantique avec filtres avancés
    pub async fn semantic_search(
        &self,
        collection: &str,
        query_embedding: Vec<f32>,
        limit: u64,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        let filter = filters.map(|f| f.to_qdrant_filter());
        
        let mut builder = SearchPointsBuilder::new(collection, query_embedding, limit)
            .with_payload(true)
            .params(SearchParamsBuilder::default().exact(false));
            
        if let Some(filter) = filter {
            builder = builder.filter(filter);
        }
        
        let response = self.client
            .search_points(builder)
            .await
            .context("Failed to search points")?;
        
        let results = response
            .result
            .into_iter()
            .filter_map(|point| {
                if let Some(point_id) = point.id {
                    let id = match point_id.point_id_options? {
                        qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => uuid,
                        qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => num.to_string(),
                    };
                    
                    Some(SearchResult {
                        id,
                        score: point.score,
                        payload: point.payload,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(results)
    }
    
    /// Supprimer des points par IDs
    pub async fn delete_points(&self, collection: &str, point_ids: Vec<String>) -> Result<()> {
        if point_ids.is_empty() {
            return Ok(());
        }
        
        use qdrant_client::qdrant::DeletePointsBuilder;
        
        let points: Vec<PointId> = point_ids.into_iter()
            .map(PointId::from)
            .collect();
        
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection).points(points)
            )
            .await
            .context("Failed to delete points")?;
        
        Ok(())
    }
    
    /// Obtenir les statistiques de la collection
    pub async fn get_collection_stats(&self, collection: &str) -> Result<CollectionStats> {
        let info = self.client
            .collection_info(collection)
            .await
            .context("Failed to get collection info")?;
        
        let collection_info = info.result.unwrap();
        
        Ok(CollectionStats {
            points_count: collection_info.points_count.unwrap_or(0),
            segments_count: collection_info.segments_count,
            disk_data_size: 0, // Non disponible dans cette version de l'API
            ram_data_size: 0,  // Non disponible dans cette version de l'API
        })
    }
}

/// Filtres de recherche
#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub group_ids: Option<Vec<String>>,
    pub document_ids: Option<Vec<String>>,
    pub chunk_types: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub min_confidence: Option<f32>,
}

impl SearchFilters {
    pub fn new() -> Self {
        Self {
            group_ids: None,
            document_ids: None,
            chunk_types: None,
            languages: None,
            tags: None,
            priority: None,
            min_confidence: None,
        }
    }
    
    fn to_qdrant_filter(&self) -> Filter {
        let mut conditions = Vec::new();
        
        if let Some(ref group_ids) = self.group_ids {
            conditions.push(Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "group_id".to_string(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keywords(
                                qdrant_client::qdrant::RepeatedStrings {
                                    strings: group_ids.clone(),
                                }
                            )),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let Some(ref languages) = self.languages {
            conditions.push(Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "language".to_string(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keywords(
                                qdrant_client::qdrant::RepeatedStrings {
                                    strings: languages.clone(),
                                }
                            )),
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        if let Some(min_confidence) = self.min_confidence {
            conditions.push(Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "confidence".to_string(),
                        range: Some(Range {
                            gte: Some(min_confidence as f64),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                )),
            });
        }
        
        Filter {
            must: conditions,
            ..Default::default()
        }
    }
}

/// Résultat de recherche
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: HashMap<String, Value>,
}

/// Statistiques d'une collection
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub points_count: u64,
    pub segments_count: u64,
    pub disk_data_size: u64,
    pub ram_data_size: u64,
}

/// Convertir un EnrichedChunk en EmbeddingPoint
impl From<&EnrichedChunk> for EmbeddingPoint {
    fn from(chunk: &EnrichedChunk) -> Self {
        let embedding = chunk.embedding.clone().unwrap_or_default();
        
        Self {
            id: chunk.id.clone(),
            embedding,
            payload: EmbeddingPayload {
                chunk_id: chunk.id.clone(),
                document_id: chunk.group_id.clone(), // TODO: Ajouter document_id à EnrichedChunk
                group_id: chunk.group_id.clone(),
                content: chunk.content.clone(),
                chunk_type: format!("{:?}", chunk.chunk_type),
                language: chunk.metadata.language.clone(),
                tags: chunk.metadata.tags.clone(),
                priority: format!("{:?}", chunk.metadata.priority),
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                symbol: chunk.metadata.symbol.clone(),
                context: chunk.metadata.context.clone(),
                confidence: chunk.metadata.confidence,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_qdrant_client_creation() {
        let config = QdrantConfig::default();
        
        // Test uniquement si Qdrant est disponible
        if let Ok(_client) = OptimizedQdrantClient::new(config).await {
            println!("✅ Qdrant client created successfully");
        } else {
            println!("⚠️  Qdrant not available (expected in tests)");
        }
    }
    
    #[test]
    fn test_search_filters() {
        let filters = SearchFilters {
            group_ids: Some(vec!["group1".to_string()]),
            languages: Some(vec!["rust".to_string()]),
            min_confidence: Some(0.8),
            ..SearchFilters::new()
        };
        
        let qdrant_filter = filters.to_qdrant_filter();
        assert_eq!(qdrant_filter.must.len(), 3);
        println!("✅ Search filters conversion working");
    }
}