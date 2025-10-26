// Benchmark nettoyé pour CustomE5Embedder + QdrantRestClient
use gravis_app_lib::rag::{
    CustomE5Embedder, CustomE5Config,
    QdrantRestClient, QdrantRestConfig, RestPoint
};
use std::collections::HashMap;
use std::time::Instant;
use serde_json::Value;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser)]
#[command(name = "gravis-rag-bench")]
#[command(about = "GRAVIS RAG Benchmark - E5 + Qdrant")]
struct Args {
    /// Transport protocol
    #[arg(long, default_value = "rest")]
    transport: Transport,
    
    /// Number of chunks to index
    #[arg(long, default_value = "1000")]
    chunks: usize,
    
    /// Number of search queries
    #[arg(long, default_value = "50")]
    queries: usize,
    
    /// Batch size for upsert
    #[arg(long, default_value = "256")]
    batch_size: usize,
    
    /// Force HNSW index construction (lower indexing threshold)
    #[arg(long, default_value = "false")]
    force_index: bool,
    
    /// EF search parameter for HNSW (32, 64, 128)
    #[arg(long, default_value = "32")]
    ef_search: u64,
    
    /// Export results to JSON file
    #[arg(long)]
    export_json: Option<String>,
    
    /// Run recall test with semantic similarity scoring
    #[arg(long, default_value = "false")]
    recall_test: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Transport {
    Rest,
    Grpc,  // Pour tests futurs
}

/// Résultats de benchmark pour export JSON
#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResults {
    // Configuration
    pub config: BenchmarkConfig,
    
    // Métriques d'indexation
    pub indexing: IndexingMetrics,
    
    // Métriques de recherche
    pub search: SearchMetrics,
    
    // Statut de l'index
    pub index_status: IndexStatus,
    
    // Métriques système
    pub system: SystemMetrics,
    
    // Timestamp
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkConfig {
    pub chunks: usize,
    pub queries: usize,
    pub batch_size: usize,
    pub force_index: bool,
    pub ef_search: u64,
    pub transport: String,
    pub recall_test: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexingMetrics {
    pub total_time_secs: f64,
    pub upsert_time_secs: f64,
    pub throughput_chunks_per_sec: f64,
    pub points_stored: usize,
    pub points_expected: usize,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchMetrics {
    pub total_time_secs: f64,
    pub queries_per_second: f64,
    pub latency_ms: LatencyMetrics,
    pub total_results: usize,
    pub avg_results_per_query: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatencyMetrics {
    pub min: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexStatus {
    pub hnsw_enabled: bool,
    pub indexed_vectors: usize,
    pub total_vectors: usize,
    pub optimizer_status: String,
    pub indexing_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemMetrics {
    pub embedding_cache_entries: usize,
    pub embedding_cache_mb: usize,
}

// RecallMetrics removed - data included directly in output display

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    println!("🚀 GRAVIS RAG Benchmark - CustomE5 (384D) + Qdrant {:?}", args.transport);
    println!("📊 Config: {} chunks, {} queries, batch {}, force_index={}, ef_search={}, recall_test={}", 
             args.chunks, args.queries, args.batch_size, args.force_index, args.ef_search, args.recall_test);
    
    if let Some(ref json_file) = args.export_json {
        println!("📁 Will export results to: {}", json_file);
    }
    
    let collection_name = "benchmark_custom_e5";
    
    // 1. Setup infrastructure
    println!("🔄 Phase 1: Infrastructure Setup");
    let start_setup = Instant::now();
    
    let embedder_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(embedder_config).await?;
    
    let rest_config = QdrantRestConfig::default();
    let rest_client = QdrantRestClient::new(rest_config)?;
    
    // Health check
    if !rest_client.health_check().await? {
        panic!("❌ Qdrant server not available");
    }
    
    println!("✅ Infrastructure ready in {:.2}s", start_setup.elapsed().as_secs_f64());
    
    // 2. Collection setup (CLEAN SLATE - delete puis recréer pour isolation)
    println!("🔄 Phase 2: Collection Setup (384D) - Clean Isolation");
    
    // Supprimer la collection existante pour garantir l'isolation
    match rest_client.delete_collection(collection_name).await {
        Ok(_) => println!("✅ Previous collection deleted"),
        Err(_) => {}, // OK si n'existe pas
    }
    
    // Attendre un peu pour que la suppression soit effective
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Créer une nouvelle collection propre
    match rest_client.create_collection(collection_name, 384, "Cosine").await {
        Ok(_) => println!("✅ Clean collection created (isolated run)"),
        Err(e) => {
            println!("❌ Collection creation failed: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. Indexing benchmark
    println!("🔄 Phase 3: Indexing {} chunks (batch {})", args.chunks, args.batch_size);
    let start_indexing = Instant::now();
    
    // ID de run pour traçabilité (collection propre = pas de doublons)
    let run_id = chrono::Utc::now().timestamp_millis();
    
    // Collection propre = 0 points au départ
    let points_before = 0;
    
    let test_chunks = generate_test_chunks(args.chunks);
    let mut points = Vec::new();
    
    for (i, chunk) in test_chunks.iter().enumerate() {
        let embedding = embedder.encode(chunk).await?;
        
        // Assert dimension pour sécurité
        assert_eq!(embedding.len(), 384, "Embedding dimension mismatch: expected 384, got {}", embedding.len());
        
        let mut payload = HashMap::new();
        payload.insert("text".to_string(), Value::String(chunk.clone()));
        payload.insert("chunk_id".to_string(), Value::Number(i.into()));
        payload.insert("run_id".to_string(), Value::Number(run_id.into()));
        payload.insert("timestamp".to_string(), Value::String(chrono::Utc::now().to_rfc3339()));
        
        // ID simple : collection propre = pas de conflits
        let point_id = i as u64;
        
        points.push(RestPoint {
            id: Value::Number(point_id.into()),
            vector: embedding,
            payload: Some(payload),
        });
        
        if (i + 1) % 100 == 0 {
            println!("  Processed {}/{} chunks", i + 1, args.chunks);
        }
    }
    
    // Upsert par batch pour éviter la limite 32MB JSON
    let upsert_start = Instant::now();
    println!("  Debug: Upserting {} points in batches of {}...", points.len(), args.batch_size);
    
    // Debug: vérifier le premier point
    if !points.is_empty() {
        println!("  Debug: First point - ID: {:?}, vector_len: {}", 
                 points[0].id, points[0].vector.len());
    }
    
    let mut total_upserted = 0;
    for (batch_idx, batch) in points.chunks(args.batch_size).enumerate() {
        println!("  Batch {}: upserting {} points...", batch_idx + 1, batch.len());
        
        match rest_client.upsert_points(collection_name, batch.to_vec()).await {
            Ok(_) => {
                total_upserted += batch.len();
                println!("    ✅ Batch {} successful ({} points)", batch_idx + 1, batch.len());
            }
            Err(e) => {
                println!("    ❌ Batch {} failed: {}", batch_idx + 1, e);
                return Err(e.into());
            }
        }
        
        // Pause entre les batches pour éviter la surcharge
        if batch_idx + 1 < (points.len() + args.batch_size - 1) / args.batch_size {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    let upsert_time = upsert_start.elapsed();
    println!("  ✅ All batches complete: {}/{} points upserted", total_upserted, points.len());
    
    let total_indexing_time = start_indexing.elapsed();
    
    // Phase 3.5: Force HNSW indexing si demandé
    let mut indexed_vectors = 0;
    let mut total_vectors = 0;
    let mut optimizer_status = "unknown".to_string();
    
    if args.force_index {
        println!("🔄 Phase 3.5: Forcing HNSW index construction");
        
        // Abaisser le seuil d'indexation pour forcer la construction HNSW
        let indexing_threshold = (args.chunks / 2).max(100); // Moitié des points ou min 100
        match rest_client.update_collection_config(collection_name, Some(indexing_threshold), Some(128)).await {
            Ok(_) => println!("  ✅ Collection config updated (threshold: {})", indexing_threshold),
            Err(e) => println!("  ⚠️ Config update failed: {}", e),
        }
        
        // Attendre que l'index soit construit
        match rest_client.wait_for_indexing(collection_name, 30).await {
            Ok((indexed, total)) => {
                indexed_vectors = indexed;
                total_vectors = total;
                println!("  ✅ HNSW indexing complete: {}/{} vectors indexed", indexed, total);
            }
            Err(e) => println!("  ⚠️ Indexing wait failed: {}", e),
        }
    } else {
        // Vérification du count Qdrant (sanity check avec délai)
        println!("  Debug: Waiting 1s for indexing to settle...");
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    // Récupérer les métriques finales d'indexation (collection isolée)
    let mut points_stored = 0;
    match rest_client.collection_info(collection_name).await {
        Ok(info) => {
            // Collection propre : total_vectors = points ajoutés dans ce run
            points_stored = extract_points_count(&info);
            
            // Extraire les vraies métriques HNSW depuis l'API Qdrant
            if let Some(result) = info.get("result") {
                indexed_vectors = result.get("indexed_vectors_count")
                    .and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                total_vectors = result.get("points_count")
                    .and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                optimizer_status = result.get("optimizer_status")
                    .and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                
                println!("  📊 Index metrics: indexed_vectors={}, total_vectors={}, optimizer_status={}", 
                         indexed_vectors, total_vectors, optimizer_status);
                println!("  📊 Isolated run: expected={}, stored={} (clean collection)", 
                         args.chunks, points_stored);
                
                // Vérification d'isolation
                if total_vectors == args.chunks {
                    println!("  ✅ Perfect isolation: collection contains exactly this run's data");
                } else {
                    println!("  ⚠️ Isolation issue: expected {}, got {} total vectors", 
                             args.chunks, total_vectors);
                }
            }
            
            if !args.force_index {
                println!("  Debug: Collection info response: {}", 
                         serde_json::to_string_pretty(&info).unwrap_or_default());
            }
        }
        Err(e) => println!("⚠️ Could not verify points count: {}", e),
    }
    
    println!("✅ Indexing complete:");
    println!("  Total time: {:.2}s", total_indexing_time.as_secs_f64());
    println!("  Upsert time: {:.2}s", upsert_time.as_secs_f64());
    let success_rate = ((points_stored as f64 / args.chunks as f64) * 100.0).min(100.0);
    println!("  Points stored: {}/{} ({:.1}%)", points_stored, args.chunks, success_rate);
    println!("  Throughput: {:.1} chunks/sec", args.chunks as f64 / total_indexing_time.as_secs_f64());
    
    // 4. Search benchmark with warm-up
    println!("🔄 Phase 4: Search Performance ({} queries)", args.queries);
    let search_queries_list = generate_search_queries(args.queries);
    
    // Warm-up (3 queries)
    println!("  Warm-up (ef={})...", args.ef_search);
    for i in 0..3.min(search_queries_list.len()) {
        let query_embedding = embedder.encode(&search_queries_list[i]).await?;
        let _ = rest_client.search_points(collection_name, query_embedding, 10, Some(args.ef_search)).await?;
    }
    
    let mut search_times = Vec::new();
    let mut total_results = 0;
    let mut recall_similarities = Vec::new();
    
    let search_start = Instant::now();
    for (i, query) in search_queries_list.iter().enumerate() {
        let query_embedding = embedder.encode(query).await?;
        
        // Chronométrage précis (juste la recherche)
        let query_start = Instant::now();
        let results = rest_client.search_points(collection_name, query_embedding, 10, Some(args.ef_search)).await?;
        let query_time = query_start.elapsed();
        
        search_times.push(query_time.as_micros() as f64 / 1000.0); // ms avec précision µs
        total_results += results.result.len();
        
        // Test de recall si activé (ATTENTION: LECTURE SEULE - aucun upsert autorisé)
        if args.recall_test && !results.result.is_empty() {
            // Utiliser le score Qdrant (cosine similarity) du premier résultat
            let top_score = results.result[0].score;
            recall_similarities.push(top_score as f64);
            
            // Assert: vérifier qu'aucun point n'a été ajouté accidentellement (collection isolée)
            if i == 0 {
                let current_count = match rest_client.collection_info(collection_name).await {
                    Ok(info) => extract_points_count(&info),
                    Err(_) => 0,
                };
                assert_eq!(current_count, args.chunks, 
                    "RECALL TEST ERROR: Points count changed during search phase! Expected: {}, Now: {}",
                    args.chunks, current_count);
            }
        }
        
        if (i + 1) % 10 == 0 {
            println!("  Completed {}/{} queries", i + 1, args.queries);
        }
    }
    
    let total_search_time = search_start.elapsed();
    
    // Statistiques de recherche fiables
    search_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_search_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
    let p50_search_time = search_times[search_times.len() / 2];
    let p95_search_time = search_times[(search_times.len() as f64 * 0.95) as usize];
    let p99_search_time = search_times[(search_times.len() as f64 * 0.99) as usize];
    let min_search_time = search_times[0];
    let max_search_time = search_times[search_times.len() - 1];
    
    println!("✅ Search complete:");
    println!("  Total time: {:.3}s", total_search_time.as_secs_f64());
    println!("  Latency min/avg/p50/p95/p99/max: {:.2}/{:.2}/{:.2}/{:.2}/{:.2}/{:.2}ms", 
             min_search_time, avg_search_time, p50_search_time, p95_search_time, p99_search_time, max_search_time);
    println!("  QPS: {:.1}", args.queries as f64 / total_search_time.as_secs_f64());
    println!("  Total results: {}", total_results);
    println!("  Avg results/query: {:.1}", total_results as f64 / args.queries as f64);
    
    // Afficher les métriques de recall si activées
    if args.recall_test && !recall_similarities.is_empty() {
        let avg_similarity = recall_similarities.iter().sum::<f64>() / recall_similarities.len() as f64;
        let min_similarity = recall_similarities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_similarity = recall_similarities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("📊 Recall Test Results:");
        println!("  Avg similarity: {:.3}", avg_similarity);
        println!("  Min similarity: {:.3}", min_similarity);
        println!("  Max similarity: {:.3}", max_similarity);
        println!("  Queries tested: {}", recall_similarities.len());
    }
    
    // 5. Final stats
    println!("🔄 Phase 5: Final Statistics");
    let (cache_entries, cache_mb) = embedder.cache_stats();
    println!("  Embedding cache: {} entries, {}MB", cache_entries, cache_mb);
    
    // Calculer le statut HNSW correct
    let hnsw_enabled = indexed_vectors > 0 && optimizer_status == "ok";
    let indexing_percentage = if total_vectors > 0 {
        (indexed_vectors as f64 / total_vectors as f64) * 100.0
    } else {
        0.0
    };
    
    // Summary line for easy parsing
    println!("\n📊 SUMMARY: {} chunks indexed in {:.2}s ({:.1} ch/s), {} queries in {:.3}s (p50: {:.2}ms, p95: {:.2}ms)", 
             args.chunks, total_indexing_time.as_secs_f64(), 
             args.chunks as f64 / total_indexing_time.as_secs_f64(),
             args.queries, total_search_time.as_secs_f64(), p50_search_time, p95_search_time);
    
    println!("📈 Index status: HNSW={}, indexed_vectors={}/{} ({:.1}%), ef_search={}", 
             hnsw_enabled, indexed_vectors, total_vectors, indexing_percentage, args.ef_search);
    
    // Export JSON si demandé
    if let Some(json_file) = args.export_json {
        let results = BenchmarkResults {
            config: BenchmarkConfig {
                chunks: args.chunks,
                queries: args.queries,
                batch_size: args.batch_size,
                force_index: args.force_index,
                ef_search: args.ef_search,
                transport: format!("{:?}", args.transport),
                recall_test: args.recall_test,
            },
            indexing: IndexingMetrics {
                total_time_secs: total_indexing_time.as_secs_f64(),
                upsert_time_secs: upsert_time.as_secs_f64(),
                throughput_chunks_per_sec: args.chunks as f64 / total_indexing_time.as_secs_f64(),
                points_stored,
                points_expected: args.chunks,
                success_rate: ((points_stored as f64 / args.chunks as f64) * 100.0).min(100.0),
            },
            search: SearchMetrics {
                total_time_secs: total_search_time.as_secs_f64(),
                queries_per_second: args.queries as f64 / total_search_time.as_secs_f64(),
                latency_ms: LatencyMetrics {
                    min: min_search_time,
                    avg: avg_search_time,
                    p50: p50_search_time,
                    p95: p95_search_time,
                    p99: p99_search_time,
                    max: max_search_time,
                },
                total_results,
                avg_results_per_query: total_results as f64 / args.queries as f64,
            },
            index_status: IndexStatus {
                hnsw_enabled,
                indexed_vectors,
                total_vectors,
                optimizer_status: optimizer_status.clone(),
                indexing_percentage,
            },
            system: SystemMetrics {
                embedding_cache_entries: cache_entries,
                embedding_cache_mb: cache_mb,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        match fs::write(&json_file, serde_json::to_string_pretty(&results)?) {
            Ok(_) => println!("📁 Results exported to: {}", json_file),
            Err(e) => println!("❌ Failed to export results: {}", e),
        }
    }
    
    println!("🎯 Benchmark complete! CustomE5 (384D) + Qdrant {:?} validated ✅", args.transport);
    
    Ok(())
}

fn extract_points_count(info: &Value) -> usize {
    if let Some(result) = info.get("result") {
        // Qdrant API uses "points_count", not "vectors_count"
        if let Some(count) = result.get("points_count") {
            return count.as_u64().unwrap_or(0) as usize;
        }
    }
    0
}

fn generate_test_chunks(count: usize) -> Vec<String> {
    let templates = vec![
        "This is a test document about {}. It contains important information for RAG systems.",
        "The {} functionality provides essential capabilities for modern applications.",
        "Performance optimization in {} requires careful consideration of memory usage.",
        "When implementing {} in Rust, memory safety is automatically guaranteed.",
        "The E5-Small-v2 model excels at understanding {} related content.",
        "Vector databases like Qdrant are essential for {} search applications.",
        "Machine learning models for {} have improved significantly in recent years.",
        "The {} architecture enables scalable and efficient processing.",
    ];
    
    let topics = vec![
        "machine learning", "artificial intelligence", "natural language processing",
        "computer vision", "data science", "software engineering", "rust programming",
        "vector databases", "semantic search", "embeddings", "transformers", "neural networks",
        "deep learning", "information retrieval", "knowledge graphs", "web development",
    ];
    
    let mut chunks = Vec::new();
    for i in 0..count {
        let template = &templates[i % templates.len()];
        let topic = &topics[i % topics.len()];
        chunks.push(template.replace("{}", topic));
    }
    chunks
}

fn generate_search_queries(count: usize) -> Vec<String> {
    let queries = vec![
        "machine learning algorithms",
        "vector database performance",
        "Rust memory safety",
        "semantic search implementation",
        "natural language processing",
        "AI model optimization",
        "embeddings generation",
        "information retrieval systems",
        "deep learning architectures",
        "knowledge representation",
    ];
    
    let mut search_queries = Vec::new();
    for i in 0..count {
        search_queries.push(queries[i % queries.len()].to_string());
    }
    search_queries
}