// GRAVIS RAG - Qdrant REST Client (Fallback pour problèmes gRPC/HTTP/2)
// Solution alternative utilisant l'API REST de Qdrant

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

/// Configuration pour le client REST Qdrant
#[derive(Debug, Clone)]
pub struct QdrantRestConfig {
    pub url: String,
    pub timeout_secs: u64,
}

impl Default for QdrantRestConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Point pour l'API REST Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestPoint {
    pub id: Value,
    pub vector: Vec<f32>,
    pub payload: Option<HashMap<String, Value>>,
}

/// Réponse de recherche REST
#[derive(Debug, Serialize, Deserialize)]
pub struct RestSearchResponse {
    pub result: Vec<RestSearchResult>,
    pub time: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestSearchResult {
    pub id: Value,
    pub version: u64,
    pub score: f32,
    pub payload: Option<HashMap<String, Value>>,
    pub vector: Option<Vec<f32>>,
}

/// Client REST simple pour Qdrant
pub struct QdrantRestClient {
    client: Client,
    base_url: String,
}

impl QdrantRestClient {
    /// Créer un nouveau client REST
    pub fn new(config: QdrantRestConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: config.url,
        })
    }

    /// Supprimer une collection pour garantir l'isolation des benchmarks
    pub async fn delete_collection(&self, collection_name: &str) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        info!("🗑️ Deleting collection for clean benchmark: {}", collection_name);
        
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send delete collection request")?;

        if response.status().is_success() || response.status() == 404 {
            info!("✅ Collection deleted (or didn't exist): {}", collection_name);
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to delete collection: {} - {}",
                status,
                text
            ))
        }
    }

    /// Créer une collection
    pub async fn create_collection(
        &self,
        collection_name: &str,
        vector_size: u64,
        distance: &str,
    ) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let payload = json!({
            "vectors": {
                "size": vector_size,
                "distance": distance,
                "hnsw_config": {
                    "m": 32,
                    "ef_construct": 256,
                    "full_scan_threshold": 10000
                }
            },
            "optimizers_config": {
                "default_segment_number": 2,
                "indexing_threshold": 20000,
                "flush_interval_sec": 1
            },
            "replication_factor": 1
        });

        info!("🔄 Creating collection via REST: {}", collection_name);
        
        let response = self
            .client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send create collection request")?;

        if response.status().is_success() {
            info!("✅ Collection created successfully: {}", collection_name);
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            
            // Collection existe déjà = OK
            if status == 409 {
                info!("ℹ️ Collection already exists: {}", collection_name);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Failed to create collection: {} - {}",
                    status,
                    text
                ))
            }
        }
    }

    /// Mettre à jour la configuration de collection pour forcer l'indexation HNSW
    pub async fn update_collection_config(
        &self,
        collection_name: &str,
        indexing_threshold: Option<usize>,
        hnsw_ef_construct: Option<usize>,
    ) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let mut payload = json!({});
        
        if let Some(threshold) = indexing_threshold {
            payload["optimizers_config"] = json!({
                "indexing_threshold": threshold,
                "default_segment_number": 2
            });
        }
        
        if let Some(ef_construct) = hnsw_ef_construct {
            payload["hnsw_config"] = json!({
                "m": 16,
                "ef_construct": ef_construct,
                "full_scan_threshold": 10000,
                "on_disk": false
            });
        }

        info!("🔄 Updating collection config: {} (threshold: {:?})", collection_name, indexing_threshold);
        
        let response = self
            .client
            .patch(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to update collection config")?;

        if response.status().is_success() {
            info!("✅ Collection config updated: {}", collection_name);
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to update collection config: {} - {}",
                status,
                text
            ))
        }
    }

    /// Attendre que l'optimiseur termine et que l'index HNSW soit construit
    pub async fn wait_for_indexing(&self, collection_name: &str, timeout_secs: u64) -> Result<(usize, usize)> {
        use tokio::time::{sleep, Duration, timeout};
        
        info!("⏳ Waiting for HNSW indexing to complete...");
        
        let wait_operation = async {
            loop {
                let info = self.collection_info(collection_name).await?;
                
                if let Some(result) = info.get("result") {
                    let indexed_count = result
                        .get("indexed_vectors_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    
                    let points_count = result
                        .get("points_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    
                    let optimizer_status = result
                        .get("optimizer_status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    
                    println!("  ⏳ optimizer_status={}, indexed={}/{} vectors", 
                             optimizer_status, indexed_count, points_count);
                    
                    if optimizer_status == "ok" && indexed_count > 0 {
                        info!("✅ HNSW ready with {} indexed vectors", indexed_count);
                        return Ok((indexed_count, points_count));
                    }
                }
                
                sleep(Duration::from_secs(1)).await;
            }
        };
        
        timeout(Duration::from_secs(timeout_secs), wait_operation)
            .await
            .context("Timeout waiting for indexing")?
    }

    /// Upserter des points
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<RestPoint>,
    ) -> Result<()> {
        let url = format!("{}/collections/{}/points", self.base_url, collection_name);
        
        let payload = json!({
            "points": points
        });

        let response = self
            .client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send upsert request")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to upsert points: {} - {}",
                status,
                text
            ))
        }
    }

    /// Chercher des points similaires
    pub async fn search_points(
        &self,
        collection_name: &str,
        vector: Vec<f32>,
        limit: u64,
        ef: Option<u64>,
    ) -> Result<RestSearchResponse> {
        let url = format!("{}/collections/{}/points/search", self.base_url, collection_name);
        
        let mut payload = json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false
        });

        // Ajouter paramètres de recherche si spécifiés
        if let Some(ef_value) = ef {
            payload["params"] = json!({
                "ef": ef_value
            });
        }

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send search request")?;

        if response.status().is_success() {
            let search_response: RestSearchResponse = response
                .json()
                .await
                .context("Failed to parse search response")?;
            Ok(search_response)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to search points: {} - {}",
                status,
                text
            ))
        }
    }

    /// Vérifier le statut du serveur
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Obtenir les informations de la collection
    pub async fn collection_info(&self, collection_name: &str) -> Result<Value> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get collection info")?;

        if response.status().is_success() {
            let info: Value = response
                .json()
                .await
                .context("Failed to parse collection info")?;
            Ok(info)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to get collection info: {} - {}",
                status,
                text
            ))
        }
    }
}

/// Tests pour le client REST
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rest_client_health() {
        let config = QdrantRestConfig::default();
        let client = QdrantRestClient::new(config).unwrap();
        
        // Test health check (peut échouer si Qdrant n'est pas démarré)
        match client.health_check().await {
            Ok(healthy) => {
                if healthy {
                    println!("✅ Qdrant REST API is healthy");
                } else {
                    println!("⚠️ Qdrant REST API not responding");
                }
            }
            Err(e) => {
                println!("⚠️ Health check failed: {}", e);
            }
        }
    }
}