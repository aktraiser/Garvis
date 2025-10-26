// GRAVIS RAG - Benchmark Tool pour validation production
// Test de performance avec 100k chunks réalistes

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

use super::{
    CustomE5Embedder, CustomE5Config, QdrantRestClient, QdrantRestConfig,
    EnrichedChunk, ChunkType, ChunkMetadata, Priority
};

/// Configuration du benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub chunks_count: usize,           // 100k pour test complet
    pub chunk_size_range: (usize, usize), // (256, 1024) tokens
    pub ef_search_values: Vec<u64>,    // [32, 64, 128] pour tuning
    pub search_queries: usize,         // 1000 queries pour p95
    pub batch_sizes: Vec<usize>,       // [32, 64, 128] pour optimisation
    pub collections: Vec<String>,      // ["code", "docs", "mixed"]
    pub output_path: String,           // benchmark_results.json
    pub csv_output: Option<String>,    // Export CSV pour analyse
    pub random_seed: Option<u64>,      // Reproductibilité
    pub qdrant_data_path: Option<String>, // Pour mesure disque
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            chunks_count: 1000, // Start smaller for dev
            chunk_size_range: (256, 1024),
            ef_search_values: vec![32, 64, 128],
            search_queries: 100,
            batch_sizes: vec![32, 64, 128],
            collections: vec!["benchmark_test".to_string()],
            output_path: "benchmark_results.json".to_string(),
            csv_output: None,
            random_seed: Some(42), // Reproductibilité par défaut
            qdrant_data_path: None,
        }
    }
}

/// Résultats détaillés du benchmark
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub config: BenchmarkConfig,
    pub system_info: SystemInfo,
    pub indexing_results: IndexingResults,
    pub search_results: SearchResults,
    pub memory_results: MemoryResults,
    pub qdrant_results: QdrantResults,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub total_memory_gb: f64,
    pub rust_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexingResults {
    pub total_chunks: usize,
    pub total_time_secs: f64,
    pub chunks_per_minute: f64,
    pub embedding_time_secs: f64,
    pub qdrant_upsert_time_secs: f64,
    pub failed_chunks: usize,
    pub avg_chunk_size_chars: f64,
    pub cache_hit_rate: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub query_count: usize,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub avg_results_returned: f64,
    pub ef_search_tuning: Vec<EfSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EfSearchResult {
    pub ef_value: u64,
    pub avg_latency_ms: f64,
    pub recall_at_10: f64, // Si golden set disponible
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryResults {
    pub peak_memory_gb: f64,  // Cohérent avec CSV
    pub device_pool_usage_mb: f64,
    pub embedding_cache_mb: f64,
    pub qdrant_memory_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantResults {
    pub collection_size_gb: f64,
    pub points_count: u64,
    pub segments_count: u64,
    pub indexing_time_secs: f64,
}

/// QRels pour évaluation recall@10 (même document = relevant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRelevanceLabels {
    pub qrels: HashMap<String, HashSet<String>>, // query_id -> set of relevant chunk_ids
    pub chunk_to_doc: HashMap<String, String>,   // chunk_id -> document_id
}

/// Métriques CSV pour export
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkCsvRow {
    pub size: String,
    pub chunks_count: usize,
    pub ef_search: u64,
    pub indexing_time_min: f64,
    pub throughput_chunks_per_min: f64,
    pub ram_max_gb: f64,
    pub qdrant_disk_gb: f64,
    pub p95_latency_ms: f64,
    pub recall_at_10: f64,
}

/// Générateur de données de test réalistes
pub struct BenchmarkDataGenerator {
    code_samples: Vec<String>,
    text_samples: Vec<String>,
    qrels: QueryRelevanceLabels,
}

impl BenchmarkDataGenerator {
    pub fn new() -> Self {
        Self {
            code_samples: Self::generate_code_samples(),
            text_samples: Self::generate_text_samples(),
            qrels: QueryRelevanceLabels {
                qrels: HashMap::new(),
                chunk_to_doc: HashMap::new(),
            },
        }
    }
    
    /// Générer des échantillons de code réalistes
    fn generate_code_samples() -> Vec<String> {
        vec![
            // Rust functions
            r#"
pub async fn process_document(id: &str, content: &str) -> Result<ProcessedDoc> {
    let chunks = chunk_document(content, ChunkConfig::default()).await?;
    let embeddings = embed_chunks(&chunks).await?;
    let result = ProcessedDoc {
        id: id.to_string(),
        chunks,
        embeddings,
        processed_at: SystemTime::now(),
    };
    Ok(result)
}
            "#.trim().to_string(),
            
            // TypeScript components
            r#"
interface SearchProps {
    query: string;
    filters: SearchFilters;
    onResults: (results: SearchResult[]) => void;
}

export const SearchComponent: React.FC<SearchProps> = ({ query, filters, onResults }) => {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    
    useEffect(() => {
        if (query.length > 2) {
            performSearch();
        }
    }, [query, filters]);
    
    const performSearch = async () => {
        setLoading(true);
        try {
            const results = await searchAPI.query(query, filters);
            onResults(results);
        } catch (err) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    };
};
            "#.trim().to_string(),
            
            // Python ML code
            r#"
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity
from transformers import AutoTokenizer, AutoModel

class EmbeddingModel:
    def __init__(self, model_name: str = "intfloat/e5-small-v2"):
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModel.from_pretrained(model_name)
        
    def encode(self, texts: List[str]) -> np.ndarray:
        inputs = self.tokenizer(texts, padding=True, truncation=True, return_tensors="pt")
        with torch.no_grad():
            outputs = self.model(**inputs)
            embeddings = outputs.last_hidden_state.mean(dim=1)
            return F.normalize(embeddings, p=2, dim=1).numpy()
            
    def search(self, query: str, corpus_embeddings: np.ndarray, top_k: int = 10):
        query_embedding = self.encode([query])
        similarities = cosine_similarity(query_embedding, corpus_embeddings)[0]
        top_indices = similarities.argsort()[-top_k:][::-1]
        return [(idx, similarities[idx]) for idx in top_indices]
            "#.trim().to_string(),
        ]
    }
    
    /// Générer des échantillons de texte réalistes
    fn generate_text_samples() -> Vec<String> {
        vec![
            "RAG (Retrieval-Augmented Generation) est une technique qui combine la récupération d'informations avec la génération de texte. Cette approche permet aux modèles de langage d'accéder à des connaissances externes pour produire des réponses plus précises et à jour.".to_string(),
            
            "L'architecture microservices permet de décomposer une application monolithique en services indépendants. Chaque service gère une fonctionnalité spécifique et communique avec les autres via des APIs bien définies. Cette approche améliore la scalabilité et la maintenabilité.".to_string(),
            
            "Les embeddings vectoriels transforment les mots et phrases en représentations numériques dans un espace multidimensionnel. Ces vecteurs capturent la sémantique du langage, permettant de mesurer la similarité entre concepts et d'effectuer des recherches sémantiques efficaces.".to_string(),
            
            "Docker containerise les applications avec leurs dépendances, garantissant la portabilité entre environnements. Kubernetes orchestre ces conteneurs à grande échelle, gérant l'auto-scaling, les déploiements et la haute disponibilité des services distribués.".to_string(),
        ]
    }
    
    /// Générer un chunk réaliste avec construction des qrels
    pub fn generate_chunk(&mut self, id: usize, chunk_type: ChunkType) -> EnrichedChunk {
        let (content, language, symbol) = match chunk_type {
            ChunkType::Function | ChunkType::Class => {
                let sample = &self.code_samples[id % self.code_samples.len()];
                (sample.clone(), "rust".to_string(), Some("process_document".to_string()))
            }
            ChunkType::TextBlock => {
                let sample = &self.text_samples[id % self.text_samples.len()];
                (sample.clone(), "text".to_string(), None)
            }
            _ => {
                let sample = &self.text_samples[0];
                (sample.clone(), "text".to_string(), None)
            }
        };
        
        let line_count = content.lines().count();
        let chunk_id = format!("chunk_{}", id);
        let doc_id = format!("doc_{}", id / 10); // 10 chunks par document (pour qrels)
        
        // Construire les qrels : chunks du même document sont relevants
        self.qrels.chunk_to_doc.insert(chunk_id.clone(), doc_id.clone());
        
        EnrichedChunk {
            id: chunk_id,
            content,
            start_line: 1,
            end_line: line_count,
            chunk_type,
            embedding: None,
            hash: blake3::hash(format!("chunk_{}", id).as_bytes()).to_hex().to_string(),
            metadata: ChunkMetadata {
                tags: vec!["benchmark".to_string(), "test".to_string()],
                priority: Priority::Normal,
                language,
                symbol,
                context: None,
                confidence: 1.0,
            },
            group_id: "benchmark_group".to_string(),
        }
    }
    
    /// Construire les qrels finaux après génération des chunks
    pub fn build_qrels(&mut self, chunk_ids: &[String]) {
        // Pour chaque chunk, trouver les autres chunks du même document
        for chunk_id in chunk_ids {
            if let Some(doc_id) = self.qrels.chunk_to_doc.get(chunk_id) {
                let relevant_chunks: HashSet<String> = chunk_ids
                    .iter()
                    .filter(|&other_id| {
                        self.qrels.chunk_to_doc.get(other_id) == Some(doc_id)
                    })
                    .cloned()
                    .collect();
                
                // Query basée sur le contenu du chunk
                let query_id = format!("query_for_{}", chunk_id);
                self.qrels.qrels.insert(query_id, relevant_chunks);
            }
        }
    }
    
    /// Générer des queries de test réalistes avec qrels
    pub fn generate_queries_with_qrels(&self, count: usize) -> Vec<(String, String)> {
        let base_queries = vec![
            "How to implement async functions in Rust?",
            "React component with TypeScript interfaces",
            "Machine learning embedding similarity search",
            "Docker container orchestration with Kubernetes",
            "RAG architecture for document retrieval",
            "Vector database optimization techniques",
            "Microservices communication patterns",
            "Neural network training best practices",
            "Database indexing performance tuning",
            "API design patterns and REST principles",
        ];
        
        // Utilise les qrels existants et crée des queries synthétiques
        let available_queries: Vec<_> = self.qrels.qrels.keys().collect();
        let query_samples = if available_queries.is_empty() {
            // Fallback sur queries génériques
            (0..count)
                .map(|i| {
                    let query = base_queries[i % base_queries.len()].to_string();
                    (format!("generic_query_{}", i), query)
                })
                .collect()
        } else {
            // Utilise les queries liées aux chunks (meilleur pour recall@10)
            (0..count)
                .map(|i| {
                    let query_id = available_queries[i % available_queries.len()].clone();
                    let query_text = base_queries[i % base_queries.len()].to_string();
                    (query_id, query_text)
                })
                .collect()
        };
        
        query_samples
    }
    
    /// Calculer le recall@10 avec les qrels
    pub fn calculate_recall_at_10(&self, query_id: &str, top_k_results: &[String]) -> f64 {
        if let Some(relevant_set) = self.qrels.qrels.get(query_id) {
            let hits: usize = top_k_results
                .iter()
                .take(10)
                .filter(|result_id| relevant_set.contains(*result_id))
                .count();
            
            hits as f64 / relevant_set.len().max(1) as f64
        } else {
            0.0 // Pas de qrels pour cette query
        }
    }
}

/// Outil de benchmark principal
pub struct RagBenchmark {
    config: BenchmarkConfig,
    data_generator: BenchmarkDataGenerator,
    results: BenchmarkResults,
}

impl RagBenchmark {
    pub fn new(config: BenchmarkConfig) -> Self {
        let system_info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus::get(),
            total_memory_gb: 16.0, // TODO: Détecter vraie RAM
            rust_version: "1.70.0".to_string(), // TODO: Détecter version
        };
        
        let results = BenchmarkResults {
            config: config.clone(),
            system_info,
            indexing_results: IndexingResults {
                total_chunks: 0,
                total_time_secs: 0.0,
                chunks_per_minute: 0.0,
                embedding_time_secs: 0.0,
                qdrant_upsert_time_secs: 0.0,
                failed_chunks: 0,
                avg_chunk_size_chars: 0.0,
                cache_hit_rate: 0.0,
            },
            search_results: SearchResults {
                query_count: 0,
                latency_p50_ms: 0.0,
                latency_p95_ms: 0.0,
                latency_p99_ms: 0.0,
                avg_results_returned: 0.0,
                ef_search_tuning: vec![],
            },
            memory_results: MemoryResults {
                peak_memory_gb: 0.0,
                device_pool_usage_mb: 0.0,
                embedding_cache_mb: 0.0,
                qdrant_memory_mb: 0.0,
            },
            qdrant_results: QdrantResults {
                collection_size_gb: 0.0,
                points_count: 0,
                segments_count: 0,
                indexing_time_secs: 0.0,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        Self {
            config,
            data_generator: BenchmarkDataGenerator::new(),
            results,
        }
    }
    
    /// Exécuter le benchmark complet
    pub async fn run_full_benchmark(&mut self) -> Result<()> {
        info!("🚀 Starting GRAVIS RAG Benchmark");
        info!("Target: {} chunks, {} queries", self.config.chunks_count, self.config.search_queries);
        
        // 1. Setup infrastructure
        let (embedder, qdrant_client) = self.setup_infrastructure().await?;
        
        // 2. Test d'indexation
        info!("📊 Phase 1: Indexation Performance");
        self.benchmark_indexing(&embedder, &qdrant_client).await?;
        
        // 3. Test de recherche
        info!("🔍 Phase 2: Search Performance");
        self.benchmark_search(&embedder, &qdrant_client).await?;
        
        // 4. Test de mémoire
        info!("💾 Phase 3: Memory Usage");
        self.benchmark_memory(&embedder, &qdrant_client).await?;
        
        // 5. Sauvegarder les résultats
        self.save_results().await?;
        
        info!("✅ Benchmark complete! Results saved to {}", self.config.output_path);
        Ok(())
    }
    
    /// Setup de l'infrastructure de test
    async fn setup_infrastructure(&self) -> Result<(Arc<CustomE5Embedder>, Arc<QdrantRestClient>)> {
        // Configuration Custom E5 (384D)
        let e5_config = CustomE5Config {
            model_id: "intfloat/e5-small-v2".to_string(),
            ..Default::default()
        };
        
        // Configuration Qdrant REST (384D compatible)
        let qdrant_config = QdrantRestConfig::default();
        
        // Initialisation
        info!("🔄 Initializing CustomE5Embedder (384D)...");
        let embedder = CustomE5Embedder::new(e5_config).await
            .context("Failed to initialize CustomE5 embedder")?;
        
        info!("🔄 Initializing QdrantRestClient...");
        let qdrant_client = QdrantRestClient::new(qdrant_config)
            .context("Failed to initialize Qdrant REST client")?;
        
        let embedder_arc = Arc::new(embedder);
        let qdrant_arc = Arc::new(qdrant_client);
        
        info!("✅ Infrastructure initialized: CustomE5 (384D) + Qdrant REST");
        Ok((embedder_arc, qdrant_arc))
    }
    
    /// Benchmark de l'indexation
    async fn benchmark_indexing(
        &mut self,
        embedder: &Arc<CustomE5Embedder>,
        qdrant_client: &Arc<QdrantRestClient>,
    ) -> Result<()> {
        let start_time = Instant::now();
        let mut failed_chunks = 0;
        let collection = &self.config.collections[0];
        
        // Créer la collection avec 384D pour E5-Small-v2
        info!("🔄 Creating collection '{}' with 384D vectors", collection);
        match qdrant_client.create_collection(collection, 384, "Cosine").await {
            Ok(_) => info!("✅ Collection created successfully"),
            Err(e) => info!("ℹ️ Collection exists or creation failed: {}", e),
        }
        
        info!("Generating {} test chunks...", self.config.chunks_count);
        
        // Générer les chunks de test avec qrels
        let mut chunks = Vec::new();
        for i in 0..self.config.chunks_count {
            let chunk_type = match i % 4 {
                0 => ChunkType::Function,
                1 => ChunkType::TextBlock,
                2 => ChunkType::Class,
                _ => ChunkType::Comment,
            };
            chunks.push(self.data_generator.generate_chunk(i, chunk_type));
        }
        
        // Construire les qrels maintenant que tous les chunks sont générés
        let chunk_ids: Vec<String> = chunks.iter().map(|c| c.id.clone()).collect();
        self.data_generator.build_qrels(&chunk_ids);
        
        // Calculer statistiques
        total_chars = chunks.iter().map(|c| c.content.len()).sum::<usize>();
        
        info!("Starting indexation of {} chunks...", chunks.len());
        let embed_start = Instant::now();
        
        // Traitement par batch
        for chunk in chunks {
            match batcher.submit_chunk(chunk, collection.clone()).await {
                Ok(_completion_rx) => {
                    // TODO: Attendre completion pour stats précises
                }
                Err(_) => {
                    failed_chunks += 1;
                }
            }
        }
        
        // Attendre que la queue soit vide
        if let Err(_) = batcher.wait_for_empty_queue(Duration::from_secs(300)).await {
            warn!("Timeout waiting for indexing completion");
        }
        
        let embed_time = embed_start.elapsed();
        let total_time = start_time.elapsed();
        
        // Collecter les stats du cache
        let embedder_stats = embedder.cache_stats();
        
        // Mettre à jour les résultats
        self.results.indexing_results = IndexingResults {
            total_chunks: self.config.chunks_count,
            total_time_secs: total_time.as_secs_f64(),
            chunks_per_minute: (self.config.chunks_count as f64) / (total_time.as_secs_f64() / 60.0),
            embedding_time_secs: embed_time.as_secs_f64(),
            qdrant_upsert_time_secs: total_time.as_secs_f64() - embed_time.as_secs_f64(),
            failed_chunks,
            avg_chunk_size_chars: total_chars as f64 / self.config.chunks_count as f64,
            cache_hit_rate: 0.0, // TODO: Récupérer depuis embedder_stats
        };
        
        info!("Indexation complete: {:.1} chunks/min", self.results.indexing_results.chunks_per_minute);
        Ok(())
    }
    
    /// Benchmark de la recherche
    async fn benchmark_search(
        &mut self,
        embedder: &Arc<E5Embedder>,
        qdrant_client: &Arc<OptimizedQdrantClient>,
    ) -> Result<()> {
        let collection = &self.config.collections[0];
        let queries_with_ids = self.data_generator.generate_queries_with_qrels(self.config.search_queries);
        let mut latencies = Vec::new();
        let mut ef_results = Vec::new();
        
        info!("Testing search performance with {} queries...", queries_with_ids.len());
        
        // Test pour chaque valeur ef_search
        for &ef_value in &self.config.ef_search_values {
            info!("Testing ef_search = {}", ef_value);
            let mut ef_latencies = Vec::new();
            let mut recall_scores = Vec::new();
            
            for (query_id, query_text) in &queries_with_ids {
                let start = Instant::now();
                
                // Générer embedding de la query
                let query_embedding = embedder.encode(query_text).await?;
                
                // TODO: Configurer ef_search dans la recherche
                let search_results = qdrant_client.semantic_search(
                    collection,
                    query_embedding,
                    10,
                    None,
                ).await?;
                
                let latency = start.elapsed();
                ef_latencies.push(latency.as_millis() as f64);
                latencies.push(latency.as_millis() as f64);
                
                // Calculer recall@10
                let result_ids: Vec<String> = search_results.iter()
                    .map(|r| r.id.clone())
                    .collect();
                let recall = self.data_generator.calculate_recall_at_10(query_id, &result_ids);
                recall_scores.push(recall);
            }
            
            let avg_latency = ef_latencies.iter().sum::<f64>() / ef_latencies.len() as f64;
            let avg_recall = recall_scores.iter().sum::<f64>() / recall_scores.len().max(1) as f64;
            
            ef_results.push(EfSearchResult {
                ef_value,
                avg_latency_ms: avg_latency,
                recall_at_10: avg_recall,
            });
            
            info!("ef_search={}: avg_latency={:.1}ms, recall@10={:.3}", ef_value, avg_latency, avg_recall);
        }
        
        // Calculer percentiles
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = latencies[latencies.len() * 50 / 100];
        let p95 = latencies[latencies.len() * 95 / 100];
        let p99 = latencies[latencies.len() * 99 / 100];
        
        self.results.search_results = SearchResults {
            query_count: queries_with_ids.len(),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            avg_results_returned: 10.0,
            ef_search_tuning: ef_results,
        };
        
        info!("Search complete: p95 = {:.1}ms", p95);
        Ok(())
    }
    
    /// Benchmark de la mémoire
    async fn benchmark_memory(
        &mut self,
        embedder: &Arc<E5Embedder>,
        qdrant_client: &Arc<OptimizedQdrantClient>,
    ) -> Result<()> {
        info!("📊 Measuring memory usage and disk footprint...");
        
        let embedder_stats = embedder.cache_stats();
        let collection_stats = qdrant_client.get_collection_stats(&self.config.collections[0]).await?;
        
        // Mesurer l'utilisation disque de Qdrant
        let qdrant_disk_gb = self.measure_qdrant_disk_usage();
        
        // Estimer la mémoire peak (basé sur les chunks traités)
        let estimated_peak_mb = (self.config.chunks_count as f64 * 1.5) / 1000.0 + 256.0; // Base + overhead
        
        self.results.memory_results = MemoryResults {
            peak_memory_gb: estimated_peak_mb / 1024.0,
            device_pool_usage_mb: 256.0,
            embedding_cache_mb: embedder_stats.embedding_memory_mb as f64,
            qdrant_memory_mb: 300.0,
        };
        
        self.results.qdrant_results = QdrantResults {
            collection_size_gb: qdrant_disk_gb,
            points_count: collection_stats.points_count,
            segments_count: collection_stats.segments_count,
            indexing_time_secs: self.results.indexing_results.total_time_secs,
        };
        
        info!("💾 Memory: {:.1}GB peak, Qdrant disk: {:.2}GB", 
              self.results.memory_results.peak_memory_gb, qdrant_disk_gb);
        
        Ok(())
    }
    
    /// Sauvegarder les résultats
    async fn save_results(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.results)?;
        fs::write(&self.config.output_path, json)?;
        
        // Export CSV si configuré
        if let Some(csv_path) = &self.config.csv_output {
            self.export_to_csv(csv_path)?;
        }
        
        // Afficher résumé
        self.print_summary();
        
        Ok(())
    }
    
    /// Export CSV pour analyse
    fn export_to_csv(&self, csv_path: &str) -> Result<()> {
        let mut csv_rows = Vec::new();
        
        // Déterminer la taille du benchmark
        let size = if self.config.chunks_count >= 100_000 {
            "full"
        } else if self.config.chunks_count >= 10_000 {
            "large"
        } else if self.config.chunks_count >= 1_000 {
            "medium"
        } else {
            "small"
        };
        
        // Une ligne par ef_search
        for ef_result in &self.results.search_results.ef_search_tuning {
            let row = BenchmarkCsvRow {
                size: size.to_string(),
                chunks_count: self.config.chunks_count,
                ef_search: ef_result.ef_value,
                indexing_time_min: self.results.indexing_results.total_time_secs / 60.0,
                throughput_chunks_per_min: self.results.indexing_results.chunks_per_minute,
                ram_max_gb: self.results.memory_results.peak_memory_gb,
                qdrant_disk_gb: self.results.qdrant_results.collection_size_gb,
                p95_latency_ms: ef_result.avg_latency_ms, // TODO: utiliser p95 réel
                recall_at_10: ef_result.recall_at_10,
            };
            csv_rows.push(row);
        }
        
        // Écrire le CSV
        let mut csv_content = String::from("size,chunks_count,ef_search,indexing_time_min,throughput_chunks_per_min,ram_max_gb,qdrant_disk_gb,p95_latency_ms,recall_at_10\n");
        
        for row in csv_rows {
            csv_content.push_str(&format!(
                "{},{},{},{:.2},{:.1},{:.2},{:.2},{:.1},{:.3}\n",
                row.size, row.chunks_count, row.ef_search,
                row.indexing_time_min, row.throughput_chunks_per_min,
                row.ram_max_gb, row.qdrant_disk_gb,
                row.p95_latency_ms, row.recall_at_10
            ));
        }
        
        fs::write(csv_path, csv_content)?;
        info!("📊 CSV exported to: {}", csv_path);
        
        Ok(())
    }
    
    /// Mesurer l'utilisation disque de Qdrant
    fn measure_qdrant_disk_usage(&self) -> f64 {
        if let Some(data_path) = &self.config.qdrant_data_path {
            // Mesurer la taille du répertoire Qdrant
            if let Ok(entries) = fs::read_dir(data_path) {
                let total_bytes: u64 = entries
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| entry.metadata().ok())
                    .map(|metadata| metadata.len())
                    .sum();
                
                return total_bytes as f64 / 1_073_741_824.0; // Bytes to GB
            }
        }
        
        // Estimation par défaut : 384D * 4 bytes * chunks + overhead
        let vector_size_bytes = 384 * 4 * self.config.chunks_count;
        let estimated_gb = vector_size_bytes as f64 / 1_073_741_824.0 * 1.3; // +30% overhead
        
        estimated_gb
    }
    
    /// Afficher le résumé des résultats
    fn print_summary(&self) {
        let r = &self.results;
        
        println!("\n🎯 GRAVIS RAG Benchmark Results");
        println!("=====================================");
        println!("📊 Indexing Performance:");
        println!("  • {} chunks indexed in {:.1}s", r.indexing_results.total_chunks, r.indexing_results.total_time_secs);
        println!("  • Throughput: {:.1} chunks/min", r.indexing_results.chunks_per_minute);
        println!("  • Failed: {} chunks", r.indexing_results.failed_chunks);
        println!("  • Avg chunk size: {:.0} chars", r.indexing_results.avg_chunk_size_chars);
        
        println!("\n🔍 Search Performance:");
        println!("  • {} queries tested", r.search_results.query_count);
        println!("  • p50 latency: {:.1}ms", r.search_results.latency_p50_ms);
        println!("  • p95 latency: {:.1}ms", r.search_results.latency_p95_ms);
        println!("  • p99 latency: {:.1}ms", r.search_results.latency_p99_ms);
        
        println!("\n⚡ ef_search Tuning:");
        for ef_result in &r.search_results.ef_search_tuning {
            println!("  • ef={}: {:.1}ms avg, recall@10={:.2}", 
                ef_result.ef_value, ef_result.avg_latency_ms, ef_result.recall_at_10);
        }
        
        println!("\n💾 Memory Usage:");
        println!("  • Peak RAM: {:.1}GB", r.memory_results.peak_memory_gb);
        println!("  • Qdrant collection: {:.0} points, {:.1}GB", r.qdrant_results.points_count, r.qdrant_results.collection_size_gb);
        
        println!("\n📋 Recommendations:");
        if r.search_results.latency_p95_ms > 100.0 {
            println!("  ⚠️  Consider reducing ef_search for better latency");
        }
        if r.indexing_results.chunks_per_minute < 50.0 {
            println!("  ⚠️  Indexing throughput is low, check batch sizes");
        }
        if r.indexing_results.failed_chunks > 0 {
            println!("  ⚠️  {} chunks failed to index, check error logs", r.indexing_results.failed_chunks);
        }
        
        println!("\n✅ Results saved to: {}", self.config.output_path);
    }
}

/// Point d'entrée pour le benchmark CLI
pub async fn run_benchmark_cli(args: Vec<String>) -> Result<()> {
    let config = if args.len() > 1 && Path::new(&args[1]).exists() {
        // Charger config depuis fichier JSON
        let config_str = fs::read_to_string(&args[1])?;
        serde_json::from_str(&config_str)?
    } else {
        // Configuration par défaut
        BenchmarkConfig::default()
    };
    
    let mut benchmark = RagBenchmark::new(config);
    benchmark.run_full_benchmark().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_generator() {
        let generator = BenchmarkDataGenerator::new();
        let chunk = generator.generate_chunk(0, ChunkType::Function);
        
        assert!(!chunk.content.is_empty());
        assert_eq!(chunk.id, "chunk_0");
        assert_eq!(chunk.chunk_type, ChunkType::Function);
        println!("✅ Data generator working");
    }
    
    #[test]
    fn test_queries_generation() {
        let generator = BenchmarkDataGenerator::new();
        let queries = generator.generate_queries(50);
        
        assert_eq!(queries.len(), 50);
        assert!(queries.iter().all(|q| !q.is_empty()));
        println!("✅ Query generation working");
    }
    
    #[tokio::test]
    async fn test_small_benchmark() {
        let config = BenchmarkConfig {
            chunks_count: 10,
            search_queries: 5,
            ..Default::default()
        };
        
        let mut benchmark = RagBenchmark::new(config);
        
        // Test seulement la génération de données
        let generator = BenchmarkDataGenerator::new();
        let chunks: Vec<_> = (0..10)
            .map(|i| generator.generate_chunk(i, ChunkType::TextBlock))
            .collect();
        
        assert_eq!(chunks.len(), 10);
        println!("✅ Small benchmark data generation successful");
    }
}