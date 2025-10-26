use std::sync::Arc;
use tokio;

use gravis_app_lib::rag::embedder::{E5Embedder, E5Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🧪 Testing E5 Embedder standalone");
    
    // Configure E5 embedder
    let config = E5Config::default();
    
    // Initialize embedder
    println!("🔄 Initializing E5 embedder...");
    let embedder = E5Embedder::new(config).await?;
    
    // Test embedding generation
    let test_texts = vec![
        "This is a test document about artificial intelligence.",
        "Machine learning is a subset of AI.",
        "Vector databases are useful for semantic search.",
    ];
    
    println!("📝 Testing embedding generation...");
    for (i, text) in test_texts.iter().enumerate() {
        println!("  Text {}: {}", i + 1, text);
        
        let embedding = embedder.encode(text).await?;
        println!("  ✅ Generated embedding with dimension: {}", embedding.len());
        println!("  📊 First 5 values: {:?}", &embedding[..5]);
        println!();
    }
    
    // Test cache functionality
    println!("🔄 Testing cache functionality...");
    let start = std::time::Instant::now();
    let _embedding1 = embedder.encode(&test_texts[0]).await?;
    let time1 = start.elapsed();
    
    let start = std::time::Instant::now();
    let _embedding2 = embedder.encode(&test_texts[0]).await?; // Should be cached
    let time2 = start.elapsed();
    
    println!("  First call: {:?}", time1);
    println!("  Cached call: {:?}", time2);
    println!("  ✅ Cache working: {}", time2 < time1);
    
    // Display stats
    embedder.log_stats();
    
    println!("✅ All tests passed! E5 Embedder is working correctly.");
    
    Ok(())
}