// Test avec le client REST pur (sans gRPC)
use gravis_app_lib::rag::{
    E5Embedder, E5Config, 
    QdrantRestClient, QdrantRestConfig, RestPoint
};
use std::collections::HashMap;
use serde_json::Value;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🧪 Test RAG complet avec client REST pur");
    
    // 1. Init E5 embedder
    println!("🔄 Initializing E5 embedder...");
    let embedder_config = E5Config::default();
    let embedder = E5Embedder::new(embedder_config).await?;
    println!("✅ E5 embedder initialized");
    
    // 2. Init REST client (pas de gRPC)
    println!("🔄 Initializing REST client (port 6333)...");
    let rest_config = QdrantRestConfig::default();
    let rest_client = QdrantRestClient::new(rest_config)?;
    
    // Test health check
    match rest_client.health_check().await {
        Ok(true) => println!("✅ Qdrant REST API healthy"),
        Ok(false) => println!("❌ Qdrant REST API not responding"),
        Err(e) => println!("❌ Health check failed: {}", e),
    }
    
    // 3. Créer collection via REST
    let collection_name = "test_rest_only";
    println!("🔄 Creating collection via REST: {}", collection_name);
    
    match rest_client.create_collection(collection_name, 384, "Cosine").await {
        Ok(_) => println!("✅ Collection created successfully"),
        Err(e) => println!("⚠️ Collection creation: {} (maybe exists)", e),
    }
    
    // 4. Générer embeddings
    println!("🔄 Generating test embeddings...");
    let test_texts = vec![
        "This is a test document about machine learning",
        "Vector databases are essential for AI applications", 
        "Rust provides memory safety and performance",
    ];
    
    let mut rest_points = Vec::new();
    for (i, text) in test_texts.iter().enumerate() {
        let embedding = embedder.encode(text).await?;
        println!("  Generated embedding {}: {}D", i + 1, embedding.len());
        
        let mut payload = HashMap::new();
        payload.insert("text".to_string(), Value::String(text.to_string()));
        payload.insert("id".to_string(), Value::Number(i.into()));
        
        rest_points.push(RestPoint {
            id: Value::Number(i.into()),
            vector: embedding,
            payload: Some(payload),
        });
    }
    
    // 5. Debug du JSON avant upsert
    println!("🔄 Debug JSON structure...");
    let json_debug = serde_json::to_string_pretty(&rest_points[0])?;
    println!("JSON structure envoyé:\n{}", json_debug);
    
    // 5. Upsert via REST (le vrai test)
    println!("🔄 Upserting {} points via REST...", rest_points.len());
    match rest_client.upsert_points(collection_name, rest_points).await {
        Ok(_) => println!("✅ Points upserted successfully!"),
        Err(e) => {
            println!("❌ Upsert failed: {}", e);
            return Err(e.into());
        }
    }
    
    // 6. Test recherche
    println!("🔄 Testing search...");
    let query_embedding = embedder.encode("machine learning AI").await?;
    
    match rest_client.search_points(collection_name, query_embedding, 3, Some(32)).await {
        Ok(response) => {
            println!("✅ Search successful! Found {} results", response.result.len());
            for (i, result) in response.result.iter().enumerate() {
                println!("  Result {}: score={:.3}, id={:?}", i + 1, result.score, result.id);
            }
        }
        Err(e) => println!("❌ Search failed: {}", e),
    }
    
    // 7. Stats collection
    match rest_client.collection_info(collection_name).await {
        Ok(info) => {
            println!("📊 Collection info: {}", serde_json::to_string_pretty(&info)?);
        }
        Err(e) => println!("⚠️ Collection info failed: {}", e),
    }
    
    println!("🎯 Test REST complet terminé !");
    Ok(())
}