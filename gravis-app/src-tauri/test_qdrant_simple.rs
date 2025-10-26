use gravis_app_lib::rag::qdrant::{OptimizedQdrantClient, QdrantConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🔍 Test de connectivité Qdrant simple");
    
    // Test REST API (port 6333)
    let config = QdrantConfig::default();
    println!("📡 Configuration: {} (gRPC: {})", config.url, config.prefer_grpc);
    
    let client = OptimizedQdrantClient::new(config).await?;
    println!("✅ Client créé avec succès");
    
    // Test création collection
    let collection_name = "test_connectivity";
    println!("🔄 Test création collection: {}", collection_name);
    
    match client.create_optimized_collection(collection_name).await {
        Ok(_) => println!("✅ Collection créée avec succès"),
        Err(e) => println!("❌ Erreur création collection: {}", e),
    }
    
    // Test stats collection
    match client.get_collection_stats(collection_name).await {
        Ok(stats) => println!("📊 Collection stats: {} points", stats.points_count),
        Err(e) => println!("❌ Erreur stats collection: {}", e),
    }
    
    println!("🏁 Test terminé");
    Ok(())
}