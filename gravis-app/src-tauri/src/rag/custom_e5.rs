// Custom E5-Small-v2 Embedder - Solution directe pour 384D
// Contourne les limitations de BertConfig en chargeant directement les weights

use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use dashmap::DashMap;
use hf_hub::api::tokio::Api;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;
use tracing::info;

/// Configuration pour l'embedder E5 personnalisé
#[derive(Debug, Clone)]
pub struct CustomE5Config {
    pub model_id: String,
    pub revision: String,
    pub cache_dir: Option<PathBuf>,
    pub max_sequence_length: usize,
    pub device: Device,
}

impl Default for CustomE5Config {
    fn default() -> Self {
        Self {
            model_id: "intfloat/e5-small-v2".to_string(),
            revision: "main".to_string(),
            cache_dir: None,
            max_sequence_length: 512,
            device: Device::Cpu,
        }
    }
}

/// Cache des embeddings
type EmbeddingCache = DashMap<String, Vec<f32>>;

/// E5 Embedder personnalisé - charge directement les tensors sans BertModel
pub struct CustomE5Embedder {
    tokenizer: Tokenizer,
    embeddings: Tensor,         // word embeddings weights [vocab_size, 384]
    cache: Arc<EmbeddingCache>,
    config: CustomE5Config,
}

impl CustomE5Embedder {
    /// Initialise l'embedder E5 personnalisé
    pub async fn new(config: CustomE5Config) -> Result<Self> {
        info!("🔄 Initializing Custom E5 embedder: {}", config.model_id);
        
        // Setup HF Hub API
        std::env::set_var("HF_TOKEN", "");
        
        let api = Api::new()
            .context("Failed to initialize HF Hub API")?;
        let repo = api.model(config.model_id.clone());
        
        // Télécharger le tokenizer
        info!("📥 Downloading tokenizer...");
        let tokenizer_path = repo
            .get("tokenizer.json")
            .await
            .context("Failed to download tokenizer")?;
        
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
        
        // Télécharger les poids du modèle
        info!("📥 Downloading model weights...");
        let weights_path = repo.get("model.safetensors").await
            .context("Failed to download model weights")?;
        
        // Charger les weights safetensors
        info!("🔧 Loading safetensors weights...");
        let vs = unsafe { 
            VarBuilder::from_mmaped_safetensors(&[&weights_path], candle_core::DType::F32, &config.device)?
        };
        
        // Extraire directement les embeddings word weights
        info!("📊 Extracting word embeddings from model...");
        let embeddings = vs.get((30522, 384), "embeddings.word_embeddings.weight")
            .context("Failed to load word embeddings tensor")?;
        
        info!("✅ Custom E5 embedder initialized with 384D embeddings");
        
        Ok(Self {
            tokenizer,
            embeddings,
            cache: Arc::new(DashMap::new()),
            config,
        })
    }
    
    /// Encode un texte en embedding 384D
    pub async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        // Cache key
        let cache_key = blake3::hash(text.as_bytes()).to_hex().to_string();
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Préfixer le texte selon E5
        let prefixed_text = format!("query: {}", text);
        
        // Tokenisation
        let encoding = self.tokenizer
            .encode(prefixed_text, true)
            .map_err(|e| anyhow::anyhow!("Failed to tokenize text: {}", e))?;
        
        let tokens = encoding.get_ids();
        
        // Pour une implémentation simple, faisons juste la moyenne des embeddings des tokens
        let mut embedding_sum = vec![0.0f32; 384];
        let mut token_count = 0;
        
        for &token_id in tokens {
            if token_id < 30522 {
                // Extraire l'embedding pour ce token
                let token_embedding = self.embeddings.get(token_id as usize)
                    .context("Failed to get token embedding")?
                    .to_vec1::<f32>()?;
                
                // Additionner
                for (i, &val) in token_embedding.iter().enumerate() {
                    embedding_sum[i] += val;
                }
                token_count += 1;
            }
        }
        
        // Moyenne
        if token_count > 0 {
            for val in &mut embedding_sum {
                *val /= token_count as f32;
            }
        }
        
        // Normalisation L2
        let norm: f32 = embedding_sum.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding_sum {
                *val /= norm;
            }
        }
        
        // Mise en cache
        self.cache.insert(cache_key, embedding_sum.clone());
        
        Ok(embedding_sum)
    }
    
    /// Stats du cache
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache_size = self.cache.len();
        let memory_mb = (cache_size * 384 * 4) / (1024 * 1024);
        (cache_size, memory_mb)
    }
    
    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}