// Test complet Phase 2 - PDF OCR + Injection + Recherche
// Test end-to-end avec document réel : DeepSeek-OCR PDF

use std::path::Path;
use tempfile::NamedTempFile;
use tokio::fs;

use gravis_app_lib::rag::{
    IngestionEngine, DocumentProcessor, UnifiedCache, StrategyDetector,
    IngestionStrategy, ChunkConfig, ChunkStrategy, Priority, SourceType,
    CustomE5Embedder, CustomE5Config
};
use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig, OcrCache, CacheConfig
};
use gravis_app_lib::rag::qdrant_rest::{QdrantRestClient, QdrantRestConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Test Phase 2 Complet - PDF OCR + Injection + Recherche");
    
    // 1. Vérifier l'existence du PDF DeepSeek-OCR
    let pdf_path = Path::new("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/2510.18234v1.pdf");
    if !pdf_path.exists() {
        eprintln!("❌ PDF DeepSeek-OCR non trouvé: {:?}", pdf_path);
        eprintln!("   Placez le fichier 2510.18234v1.pdf dans le répertoire gravis-app/");
        return Ok(());
    }
    
    println!("✅ PDF DeepSeek-OCR trouvé: {:.1}MB", 
             fs::metadata(pdf_path).await?.len() as f64 / 1_000_000.0);
    
    // 2. Setup pipeline Phase 2 complet
    let components = setup_complete_pipeline().await?;
    
    // 3. Test extraction OCR complète du PDF
    test_pdf_ocr_extraction(&components, pdf_path).await?;
    
    // 4. Test injection dans RAG avec chunking intelligent
    test_rag_injection(&components, pdf_path).await?;
    
    // 5. Test recherche sémantique dans le contenu OCR
    test_semantic_search(&components).await?;
    
    // 6. Test métriques et performance complètes
    test_performance_metrics(&components).await?;
    
    println!("✅ Test Phase 2 complet terminé avec succès!");
    
    Ok(())
}

/// Composants Phase 2 complets pour test production
struct CompletePhase2Components {
    ingestion_engine: IngestionEngine,
    document_processor: DocumentProcessor,
    unified_cache: UnifiedCache,
    strategy_detector: StrategyDetector,
    embedder: CustomE5Embedder,
    qdrant_client: QdrantRestClient,
    collection_name: String,
}

/// Setup pipeline Phase 2 complet avec Qdrant
async fn setup_complete_pipeline() -> Result<CompletePhase2Components, Box<dyn std::error::Error>> {
    println!("\n🔧 Setup pipeline Phase 2 complet...");
    
    // OCR components
    let tesseract_config = TesseractConfig::default();
    let ocr_processor = TesseractProcessor::new(tesseract_config).await?;
    
    // Embedder
    let e5_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(e5_config).await?;
    
    // Document processor
    let document_processor = DocumentProcessor::new(ocr_processor, embedder.clone()).await?;
    
    // Cache unifié
    let cache_config = CacheConfig::default();
    let ocr_cache = OcrCache::new(cache_config).await?;
    let unified_cache = UnifiedCache::new(ocr_cache);
    
    // Ingestion engine
    let ingestion_engine = IngestionEngine::new(document_processor.clone());
    
    // Strategy detector
    let strategy_detector = StrategyDetector::new();
    
    // Qdrant client
    let qdrant_config = QdrantRestConfig {
        base_url: "http://localhost:6333".to_string(),
        timeout_seconds: 30,
        vector_size: 384, // E5-small-v2
        distance_metric: "Cosine".to_string(),
    };
    let qdrant_client = QdrantRestClient::new(qdrant_config).await?;
    
    // Collection pour test
    let collection_name = format!("test_phase2_ocr_{}", 
                                  std::time::SystemTime::now()
                                      .duration_since(std::time::UNIX_EPOCH)?
                                      .as_secs());
    
    println!("  ✓ Pipeline Phase 2 complet initialisé");
    println!("  ✓ Collection Qdrant: {}", collection_name);
    
    Ok(CompletePhase2Components {
        ingestion_engine,
        document_processor,
        unified_cache,
        strategy_detector,
        embedder,
        qdrant_client,
        collection_name,
    })
}

/// Test extraction OCR complète du PDF DeepSeek-OCR
async fn test_pdf_ocr_extraction(
    components: &CompletePhase2Components, 
    pdf_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📄 Test extraction OCR complète du PDF...");
    
    // 1. Détection de stratégie pour le PDF
    let strategy = components.strategy_detector.detect_strategy(pdf_path).await?;
    println!("  ✓ Stratégie détectée: {:?}", strategy);
    
    // 2. Extraction avec IngestionEngine
    let chunk_config = ChunkConfig {
        chunk_size: 200,  // Chunks plus grands pour PDF académique
        overlap: 40,
        strategy: ChunkStrategy::Heuristic,
    };
    
    let start_time = std::time::Instant::now();
    let ingestion_result = components.ingestion_engine
        .ingest_document(pdf_path, "deepseek_ocr_group", &chunk_config)
        .await?;
    let extraction_time = start_time.elapsed();
    
    println!("  ✅ Extraction terminée en {:?}", extraction_time);
    println!("  ✓ Document ID: {}", ingestion_result.document.id);
    println!("  ✓ Chunks créés: {}", ingestion_result.document.chunks.len());
    println!("  ✓ Contenu total: {} caractères", ingestion_result.document.content.len());
    
    // 3. Analyse qualité des chunks OCR
    let mut ocr_chunks = 0;
    let mut native_chunks = 0;
    let mut total_confidence = 0.0;
    
    for (i, chunk) in ingestion_result.document.chunks.iter().take(5).enumerate() {
        match chunk.metadata.source_type {
            SourceType::OcrExtracted => {
                ocr_chunks += 1;
                println!("    Chunk OCR {}: confidence={:.2}, taille={} chars", 
                         i, chunk.metadata.confidence, chunk.content.len());
            }
            SourceType::NativeText => {
                native_chunks += 1;
                println!("    Chunk Natif {}: confidence={:.2}, taille={} chars", 
                         i, chunk.metadata.confidence, chunk.content.len());
            }
            _ => {}
        }
        total_confidence += chunk.metadata.confidence;
        
        // Afficher un extrait du contenu
        let preview = if chunk.content.len() > 100 {
            format!("{}...", &chunk.content[..100])
        } else {
            chunk.content.clone()
        };
        println!("      Contenu: \"{}\"", preview);
    }
    
    let avg_confidence = total_confidence / ingestion_result.document.chunks.len() as f32;
    println!("  ✓ Chunks OCR: {}, Chunks natifs: {}", ocr_chunks, native_chunks);
    println!("  ✓ Confidence moyenne: {:.2}", avg_confidence);
    
    // 4. Vérifier métadonnées document
    match &ingestion_result.document.document_type {
        gravis_app_lib::rag::DocumentType::PDF { extraction_strategy, native_text_ratio, ocr_pages, total_pages } => {
            println!("  ✓ Type: PDF");
            println!("    - Stratégie: {:?}", extraction_strategy);
            println!("    - Ratio texte natif: {:.2}", native_text_ratio);
            println!("    - Pages OCR: {} sur {}", ocr_pages.len(), total_pages);
        }
        _ => println!("  ⚠️  Type document inattendu: {:?}", ingestion_result.document.document_type),
    }
    
    Ok(())
}

/// Test injection dans RAG avec Qdrant
async fn test_rag_injection(
    components: &CompletePhase2Components,
    pdf_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Test injection RAG avec Qdrant...");
    
    // 1. Créer collection Qdrant
    println!("  ⏳ Création collection Qdrant: {}", components.collection_name);
    components.qdrant_client
        .create_collection(&components.collection_name)
        .await
        .map_err(|e| format!("Erreur création collection: {}", e))?;
    
    // 2. Traitement document avec chunking
    let chunk_config = ChunkConfig {
        chunk_size: 150,
        overlap: 30,
        strategy: ChunkStrategy::Heuristic,
    };
    
    let document = components.document_processor
        .process_document(pdf_path, "rag_test_group", &chunk_config)
        .await?;
    
    println!("  ✓ Document traité: {} chunks", document.chunks.len());
    
    // 3. Génération embeddings pour échantillon de chunks
    let mut embedded_chunks = 0;
    let sample_size = std::cmp::min(10, document.chunks.len());
    
    for (i, chunk) in document.chunks.iter().take(sample_size).enumerate() {
        // Générer embedding
        let embedding = components.embedder.embed(&chunk.content).await
            .map_err(|e| format!("Erreur embedding chunk {}: {}", i, e))?;
        
        // Cache l'embedding
        components.unified_cache.cache_embedding(&chunk.hash, embedding.clone());
        
        // Créer point Qdrant
        let point = gravis_app_lib::rag::qdrant_rest::RestPoint {
            id: chunk.id.clone(),
            vector: embedding,
            payload: serde_json::json!({
                "content": chunk.content,
                "source_type": format!("{:?}", chunk.metadata.source_type),
                "confidence": chunk.metadata.confidence,
                "chunk_type": format!("{:?}", chunk.chunk_type),
                "group_id": chunk.group_id,
                "hash": chunk.hash
            }),
        };
        
        // Insérer dans Qdrant
        components.qdrant_client
            .upsert_points(&components.collection_name, vec![point])
            .await
            .map_err(|e| format!("Erreur insertion point {}: {}", i, e))?;
        
        embedded_chunks += 1;
        
        if i % 5 == 0 {
            println!("    Chunk {} injecté: confidence={:.2}, source={:?}", 
                     i, chunk.metadata.confidence, chunk.metadata.source_type);
        }
    }
    
    println!("  ✅ Injection terminée: {} chunks intégrés dans Qdrant", embedded_chunks);
    
    // 4. Vérifier l'état de la collection
    let collection_info = components.qdrant_client
        .get_collection_info(&components.collection_name)
        .await
        .map_err(|e| format!("Erreur info collection: {}", e))?;
    
    println!("  ✓ Collection: {} points indexés", collection_info.points_count);
    
    Ok(())
}

/// Test recherche sémantique dans le contenu OCR
async fn test_semantic_search(
    components: &CompletePhase2Components
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Test recherche sémantique...");
    
    // Requêtes de test liées au contenu DeepSeek-OCR
    let test_queries = vec![
        "optical character recognition",
        "deep learning vision",
        "text detection accuracy",
        "benchmark performance",
        "neural network architecture",
    ];
    
    for (i, query) in test_queries.iter().enumerate() {
        println!("\n  📝 Requête {}: \"{}\"", i + 1, query);
        
        // 1. Générer embedding de la requête
        let query_embedding = components.embedder.embed(query).await
            .map_err(|e| format!("Erreur embedding requête: {}", e))?;
        
        // 2. Recherche dans Qdrant
        let search_response = components.qdrant_client
            .search_points(&components.collection_name, query_embedding, 3)
            .await
            .map_err(|e| format!("Erreur recherche: {}", e))?;
        
        println!("    ✓ {} résultats trouvés", search_response.result.len());
        
        // 3. Analyser les résultats
        for (j, result) in search_response.result.iter().enumerate() {
            let content = result.payload.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            let source_type = result.payload.get("source_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let confidence = result.payload.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            
            // Extrait du contenu
            let preview = if content.len() > 80 {
                format!("{}...", &content[..80])
            } else {
                content.to_string()
            };
            
            println!("      {}. Score={:.3}, Source={}, Conf={:.2}", 
                     j + 1, result.score, source_type, confidence);
            println!("         \"{}\"", preview);
        }
    }
    
    Ok(())
}

/// Test métriques et performance complètes
async fn test_performance_metrics(
    components: &CompletePhase2Components
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Test métriques et performance...");
    
    // 1. Métriques cache
    let cache_metrics = components.unified_cache.get_cache_metrics();
    println!("  💾 Cache Unifié:");
    println!("    - Documents: {}", cache_metrics.document_cache_size);
    println!("    - Embeddings: {}", cache_metrics.embedding_cache_size);
    println!("    - OCR: {}", cache_metrics.ocr_cache_size);
    println!("    - Mémoire estimée: {}KB", cache_metrics.memory_usage_estimate / 1024);
    
    let global_stats = components.unified_cache.get_global_stats();
    println!("    - Hit ratio: {:.1}%", global_stats.hit_ratio() * 100.0);
    println!("    - Requêtes totales: {}", global_stats.total_cache_requests);
    
    // 2. Test nettoyage cache
    let cleanup_result = components.unified_cache.cleanup_cache(100, 1).await?;
    println!("  🧹 Nettoyage cache:");
    println!("    - Documents supprimés: {}", cleanup_result.removed_documents);
    println!("    - Embeddings supprimés: {}", cleanup_result.removed_embeddings);
    println!("    - Temps nettoyage: {}ms", cleanup_result.cleanup_time_ms);
    
    // 3. Benchmark génération embeddings
    println!("  ⚡ Benchmark embeddings:");
    let test_texts = vec![
        "Short text for embedding generation benchmark.",
        "This is a longer text with multiple sentences. It contains more content to test the embedding generation performance on varied input lengths. The goal is to measure the time taken for processing different text sizes.",
        "Single word",
    ];
    
    for (i, text) in test_texts.iter().enumerate() {
        let start = std::time::Instant::now();
        let _embedding = components.embedder.embed(text).await?;
        let duration = start.elapsed();
        
        println!("    Text {}: {}ms ({} chars)", 
                 i + 1, duration.as_millis(), text.len());
    }
    
    // 4. Statistiques finales collection Qdrant
    let collection_info = components.qdrant_client
        .get_collection_info(&components.collection_name)
        .await
        .map_err(|e| format!("Erreur info collection finale: {}", e))?;
    
    println!("  🎯 Collection Qdrant finale:");
    println!("    - Points indexés: {}", collection_info.points_count);
    println!("    - Statut: {}", collection_info.status);
    
    // 5. Nettoyage final
    println!("  🧽 Nettoyage final...");
    components.qdrant_client
        .delete_collection(&components.collection_name)
        .await
        .map_err(|e| format!("Erreur suppression collection: {}", e))?;
    
    println!("  ✅ Collection {} supprimée", components.collection_name);
    
    Ok(())
}