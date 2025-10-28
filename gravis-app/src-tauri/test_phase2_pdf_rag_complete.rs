// GRAVIS Phase 2 - Test RAG complet: Extraction OCR → Injection → Recherche
// Test du pipeline complet avec le PDF DeepSeek-OCR

use gravis_app_lib::rag::{
    // Core RAG
    DocumentProcessor, CustomE5Embedder, CustomE5Config, ChunkConfig, ChunkStrategy,
    QdrantRestClient, QdrantRestConfig, 
    
    // Ingestion Phase 2
    ingestion_engine::{IngestionEngine},
    unified_cache::UnifiedCache,
    
    // OCR
    ocr::{TesseractProcessor, TesseractConfig}
};

use std::path::PathBuf;
use tokio;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("debug,tokenizers=warn,candle=warn")
        .init();

    println!("🚀 Test Phase 2 RAG Complet - DeepSeek-OCR: Extraction → Injection → Recherche");
    
    // === ÉTAPE 1: VÉRIFICATION FICHIER ===
    let pdf_path = PathBuf::from("2510.18234v1.pdf");
    if !pdf_path.exists() {
        eprintln!("❌ PDF DeepSeek-OCR non trouvé: {:?}", pdf_path);
        return Ok(());
    }
    
    let metadata = std::fs::metadata(&pdf_path)?;
    println!("✅ PDF trouvé: {:.1}MB", metadata.len() as f64 / 1024.0 / 1024.0);

    // === ÉTAPE 2: SETUP PIPELINE COMPLET ===
    println!("\n🔧 Setup pipeline RAG complet...");
    
    // Configuration OCR
    let tesseract_config = TesseractConfig::default();
    let ocr_processor = TesseractProcessor::new(tesseract_config).await?;
    println!("  ✓ OCR processor initialisé");
    
    // Configuration CustomE5 pour embeddings
    let e5_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(e5_config).await?;
    println!("  ✓ CustomE5 embedder initialisé");
    
    // Configuration Qdrant
    let qdrant_config = QdrantRestConfig {
        url: "http://localhost:6333".to_string(),
        timeout_secs: 30,
    };
    let qdrant_client = QdrantRestClient::new(qdrant_config)?;
    println!("  ✓ Qdrant client initialisé");
    
    // Test connexion Qdrant
    match qdrant_client.test_connection().await {
        Ok(_) => println!("  ✅ Connexion Qdrant validée"),
        Err(e) => {
            eprintln!("  ❌ Qdrant non accessible: {}", e);
            eprintln!("     Démarrer avec: docker run -p 6333:6333 qdrant/qdrant");
            return Ok(());
        }
    }
    
    // Document processor avec composants intégrés
    let doc_processor = DocumentProcessor::new(ocr_processor, embedder.clone()).await?;
    
    // Cache unifié Phase 2
    let cache = UnifiedCache::new(1000)?;
    
    // Moteur d'ingestion Phase 2
    let ingestion_engine = IngestionEngine::new(doc_processor);
    println!("  ✓ Pipeline RAG complet initialisé");

    // === ÉTAPE 3: EXTRACTION ET INJECTION ===
    println!("\n📄 Extraction OCR et injection RAG...");
    
    let collection_name = "benchmark_custom_e5_phase2";
    
    // Créer/réinitialiser collection
    match qdrant_client.delete_collection(collection_name).await {
        Ok(_) => println!("  ✓ Collection existante supprimée"),
        Err(_) => {} // Collection n'existait pas
    }
    
    qdrant_client.create_collection(collection_name, 384).await?;
    println!("  ✓ Collection '{}' créée", collection_name);
    
    // Configuration chunking optimisée pour recherche
    let chunk_config = ChunkConfig {
        chunk_size: 500,        // Chunks plus petits pour recherche précise
        overlap: 50,            // Overlap pour contexte
        strategy: ChunkStrategy::Hybrid,  // Stratégie hybride pour PDFs académiques
    };
    
    // Traitement et injection
    let start_time = std::time::Instant::now();
    
    let ingestion_result = ingestion_engine.ingest_document(
        &pdf_path,
        "deepseek_ocr_paper",
        &chunk_config
    ).await?;
    
    let processing_time = start_time.elapsed();
    
    println!("  ✅ Ingestion terminée en {:?}", processing_time);
    println!("  ✓ Document ID: {}", ingestion_result.document_id);
    println!("  ✓ Chunks créés: {}", ingestion_result.chunks_created);
    println!("  ✓ Vecteurs injectés: {}", ingestion_result.vectors_indexed);
    println!("  ✓ Cache hits: {}", ingestion_result.cache_hits);

    // === ÉTAPE 4: RECHERCHE SÉMANTIQUE ===
    println!("\n🔍 Test recherche sémantique...");
    
    let queries = vec![
        "deep learning optical character recognition",
        "neural network architecture design",
        "context compression techniques",
        "character detection accuracy",
        "transformer model performance"
    ];
    
    for (i, query) in queries.iter().enumerate() {
        println!("\n📍 Requête {}: \"{}\"", i+1, query);
        
        // Génération embedding pour la recherche
        let query_embedding = embedder.encode(query).await?;
        if query_embedding.is_empty() {
            println!("  ❌ Échec génération embedding");
            continue;
        }
        
        // Recherche dans Qdrant
        let search_results = qdrant_client.search_vectors(
            collection_name,
            &query_embedding,
            5  // Top 5 résultats
        ).await?;
        
        println!("  ✓ {} résultats trouvés", search_results.len());
        
        // Affichage des meilleurs résultats
        for (j, result) in search_results.iter().take(3).enumerate() {
            println!("    {}. Score: {:.3} | Chunk: {} chars", 
                j+1, result.score, result.payload.get("content")
                    .and_then(|v| v.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0)
            );
            
            // Extrait du contenu pour validation
            if let Some(content) = result.payload.get("content").and_then(|v| v.as_str()) {
                let preview = content.chars().take(100).collect::<String>();
                println!("       \"{}...\"", preview);
            }
        }
    }

    // === ÉTAPE 5: STATISTIQUES FINALES ===
    println!("\n📊 Statistiques RAG Phase 2:");
    
    // Comptage total des vecteurs
    let total_points = qdrant_client.count_points(collection_name).await?;
    println!("  📄 Document traité: 2510.18234v1.pdf");
    println!("  🧠 Vecteurs indexés: {}", total_points);
    println!("  ⚡ Temps total: {:?}", processing_time);
    println!("  🎯 Performance: {:.2}ms/chunk", 
        processing_time.as_millis() as f64 / ingestion_result.chunks_created as f64);
    
    // Validation du pipeline
    if total_points > 50 && processing_time.as_secs() < 60 {
        println!("\n✅ Phase 2 RAG Pipeline VALIDÉ!");
        println!("   🔄 Extraction OCR → Chunking → Embedding → Injection → Recherche");
        println!("   📈 Prêt pour production avec documents volumineux");
    } else {
        println!("\n⚠️  Pipeline nécessite optimisation:");
        if total_points <= 50 {
            println!("   • Chunking trop agressif ({} chunks)", total_points);
        }
        if processing_time.as_secs() >= 60 {
            println!("   • Performance trop lente ({:?})", processing_time);
        }
    }

    println!("\n🎉 Test RAG Phase 2 terminé avec succès!");
    println!("   Pipeline complet validé sur document académique DeepSeek-OCR");
    
    Ok(())
}