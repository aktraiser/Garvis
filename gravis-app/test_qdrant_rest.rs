
use gravis_app_lib::rag::qdrant_rest::{QdrantRestClient, QdrantRestConfig, RestPoint};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Qdrant REST API");
    
    // Configuration du client REST
    let config = QdrantRestConfig::default();
    let client = QdrantRestClient::new(config)?;
    
    // Test 1: Health check
    println!("1. Testing health check...");
    match client.health_check().await {
        Ok(healthy) => {
            if healthy {
                println!("   ✅ Qdrant REST API is healthy");
            } else {
                println!("   ❌ Qdrant REST API not responding");
                return Ok(());
            }
        }
        Err(e) => {
            println!("   ❌ Health check failed: {}", e);
            return Ok(());
        }
    }
    
    // Test 2: Créer une collection de test
    println!("2. Creating test collection...");
    let collection_name = "test_rest_collection";
    match client.create_collection(collection_name, 768, "Cosine").await {
        Ok(_) => println!("   ✅ Collection created/exists"),
        Err(e) => println!("   ⚠️ Collection creation failed: {}", e),
    }
    
    // Test 3: Insérer des points de test
    println!("3. Inserting test points...");
    let test_points = vec![
        RestPoint {
            id: json!(1),
            vector: vec![0.1; 768],  // Vector de test
            payload: Some({
                let mut payload = HashMap::new();
                payload.insert("text".to_string(), json!("Test document 1"));
                payload.insert("category".to_string(), json!("test"));
                payload
            }),
        },
        RestPoint {
            id: json!(2),
            vector: vec![0.2; 768],  // Vector de test différent
            payload: Some({
                let mut payload = HashMap::new();
                payload.insert("text".to_string(), json!("Test document 2"));
                payload.insert("category".to_string(), json!("test"));
                payload
            }),
        },
    ];
    
    match client.upsert_points(collection_name, test_points).await {
        Ok(_) => println!("   ✅ Points inserted successfully"),
        Err(e) => {
            println!("   ❌ Point insertion failed: {}", e);
            return Ok(());
        }
    }
    
    // Attendre un peu pour l'indexation
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Test 4: Recherche de similarité  
    println!("4. Testing similarity search...");
    let query_vector = vec![0.15; 768];  // Entre les deux points de test
    
    match client.search_points(collection_name, query_vector, 2, Some(32)).await {
        Ok(response) => {
            println!("   ✅ Search successful!");
            println!("   📊 Found {} results in {:.2}ms", 
                     response.result.len(), response.time * 1000.0);
            
            for (i, result) in response.result.iter().enumerate() {
                println!("   {}. ID: {:?}, Score: {:.4}", 
                         i + 1, result.id, result.score);
                if let Some(payload) = &result.payload {
                    if let Some(text) = payload.get("text") {
                        println!("      Text: {}", text);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Search failed: {}", e);
            return Ok(());
        }
    }
    
    // Test 5: Info collection
    println!("5. Getting collection info...");
    match client.collection_info(collection_name).await {
        Ok(info) => {
            println!("   ✅ Collection info retrieved");
            if let Some(vectors_count) = info.get("result")
                .and_then(|r| r.get("vectors_count")) {
                println!("   📊 Vectors count: {}", vectors_count);
            }
        }
        Err(e) => println!("   ⚠️ Collection info failed: {}", e),
    }
    
    println!("\n🎉 All REST API tests completed!");
    println!("✅ The embedding system can now use REST API as fallback for gRPC issues");
    
    Ok(())
}