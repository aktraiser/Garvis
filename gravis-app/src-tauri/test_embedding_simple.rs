// Test simple pour voir les embeddings bruts
use gravis_app_lib::rag::embedder::{E5Embedder, E5Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🧪 Test embeddings bruts");
    
    let embedder_config = E5Config::default();
    let embedder = E5Embedder::new(embedder_config).await?;
    
    let text = "test simple";
    println!("🔄 Generating embedding for: '{}'", text);
    
    let embedding = embedder.encode(text).await?;
    
    println!("📊 Embedding info:");
    println!("  Dimension: {}", embedding.len());
    println!("  Premiers 10 valeurs: {:?}", &embedding[..10.min(embedding.len())]);
    println!("  Dernières 10 valeurs: {:?}", &embedding[embedding.len().saturating_sub(10)..]);
    
    // Vérifier si ce sont tous des null/NaN/zero
    let null_count = embedding.iter().filter(|&&x| x.is_nan() || x == 0.0).count();
    let non_null_count = embedding.len() - null_count;
    
    println!("  Valeurs valides: {}/{}", non_null_count, embedding.len());
    
    if non_null_count == 0 {
        println!("❌ PROBLÈME: Tous les embeddings sont null/zero/NaN!");
    } else {
        println!("✅ Embeddings contiennent des valeurs réelles");
    }
    
    Ok(())
}