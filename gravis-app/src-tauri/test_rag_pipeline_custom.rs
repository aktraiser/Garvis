// Test pipeline RAG complet avec CustomE5Embedder + Qdrant REST
use gravis_app_lib::rag::{
    CustomE5Embedder, CustomE5Config,
    QdrantRestClient, QdrantRestConfig, RestPoint
};
use std::collections::HashMap;
use serde_json::Value;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🧪 Test pipeline RAG complet avec CustomE5Embedder + Qdrant REST");
    
    // 1. Init Custom E5 embedder (384D)
    println!("🔄 Initializing Custom E5 embedder...");
    let embedder_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(embedder_config).await?;
    println!("✅ Custom E5 embedder initialized (384D)");
    
    // 2. Init REST client
    println!("🔄 Initializing REST client (port 6333)...");
    let rest_config = QdrantRestConfig::default();
    let rest_client = QdrantRestClient::new(rest_config)?;
    
    // Test health check
    match rest_client.health_check().await {
        Ok(true) => println!("✅ Qdrant REST API healthy"),
        Ok(false) => println!("❌ Qdrant REST API not responding"),
        Err(e) => println!("❌ Health check failed: {}", e),
    }
    
    // 3. Créer collection via REST (384D pour E5-Small-v2)
    let collection_name = "test_custom_e5_pipeline";
    println!("🔄 Creating collection with 384D vectors: {}", collection_name);
    
    match rest_client.create_collection(collection_name, 384, "Cosine").await {
        Ok(_) => println!("✅ Collection created successfully (384D)"),
        Err(e) => println!("⚠️ Collection creation: {} (maybe exists)", e),
    }
    
    // 4. Générer embeddings avec Custom E5
    println!("🔄 Generating 384D embeddings with Custom E5...");
    let test_texts = vec![
        "This is a test document about machine learning and AI applications",
        "Vector databases are essential for RAG systems with semantic search", 
        "Rust provides memory safety and high performance for ML workloads",
        "E5-Small-v2 produces high quality 384 dimensional embeddings",
    ];
    
    let mut rest_points = Vec::new();
    for (i, text) in test_texts.iter().enumerate() {
        let embedding = embedder.encode(text).await?;
        println!("  Generated embedding {}: {}D (first 5: {:?})", 
                 i + 1, embedding.len(), &embedding[..5]);
        
        let mut payload = HashMap::new();
        payload.insert("text".to_string(), Value::String(text.to_string()));
        payload.insert("id".to_string(), Value::Number(i.into()));
        payload.insert("model".to_string(), Value::String("custom-e5-small-v2".to_string()));
        
        rest_points.push(RestPoint {
            id: Value::Number(i.into()),
            vector: embedding,
            payload: Some(payload),
        });
    }
    
    // 5. Upsert via REST avec vraies 384D embeddings
    println!("🔄 Upserting {} points with 384D embeddings...", rest_points.len());
    match rest_client.upsert_points(collection_name, rest_points).await {
        Ok(_) => println!("✅ Points upserted successfully with 384D embeddings!"),
        Err(e) => {
            println!("❌ Upsert failed: {}", e);
            return Err(e.into());
        }
    }
    
    // 6. Test recherche sémantique
    println!("🔄 Testing semantic search...");
    let query_text = "machine learning and artificial intelligence";
    let query_embedding = embedder.encode(query_text).await?;
    println!("Query: '{}' -> {}D embedding", query_text, query_embedding.len());
    
    match rest_client.search_points(collection_name, query_embedding, 4, Some(32)).await {
        Ok(response) => {
            println!("✅ Search successful! Found {} results", response.result.len());
            for (i, result) in response.result.iter().enumerate() {
                if let Some(payload) = &result.payload {
                    if let Some(text) = payload.get("text") {
                        println!("  Result {}: score={:.3}, text: {}", 
                                i + 1, result.score, text.as_str().unwrap_or("N/A"));
                    }
                }
            }
        }
        Err(e) => println!("❌ Search failed: {}", e),
    }
    
    // 7. Stats collection
    match rest_client.collection_info(collection_name).await {
        Ok(info) => {
            println!("📊 Collection info:");
            if let Some(result) = info.get("result") {
                if let Some(vectors_count) = result.get("vectors_count") {
                    println!("  Vectors count: {}", vectors_count);
                }
                if let Some(config) = result.get("config") {
                    if let Some(params) = config.get("params") {
                        if let Some(vectors) = params.get("vectors") {
                            if let Some(size) = vectors.get("size") {
                                println!("  Vector size: {}D", size);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("⚠️ Collection info failed: {}", e),
    }
    
    // 8. Stats embedder
    let (cache_size, memory_mb) = embedder.cache_stats();
    println!("📈 Embedder cache: {} entries, {}MB", cache_size, memory_mb);
    
    println!("🎯 Pipeline RAG complet terminé avec CustomE5 (384D) + Qdrant REST !");
    Ok(())
}