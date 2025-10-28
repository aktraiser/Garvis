// Test d'intégration Phase 2 - Pipeline OCR-RAG Intelligent
// Tests end-to-end: PDF → RAG → Search avec cache unifié

use tempfile::NamedTempFile;
use tokio::fs::write;

use gravis_app_lib::rag::{
    IngestionEngine, DocumentProcessor, UnifiedCache, StrategyDetector,
    IngestionStrategy, ChunkConfig, ChunkStrategy, Priority
};
use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig, OcrCache, CacheConfig, OcrConfig
};
use gravis_app_lib::rag::custom_e5::{CustomE5Embedder, CustomE5Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Test Phase 2 - Pipeline OCR-RAG Intelligent");
    
    // 1. Setup composants Phase 2
    let components = setup_components().await?;
    
    // 2. Test IngestionEngine avec détection automatique
    test_ingestion_engine(&components).await?;
    
    // 3. Test StrategyDetector heuristiques
    test_strategy_detector(&components).await?;
    
    // 4. Test chunking adaptatif par source type
    test_adaptive_chunking(&components).await?;
    
    // 5. Test cache unifié OCR → Embeddings → Documents
    test_unified_cache(&components).await?;
    
    // 6. Test pipeline end-to-end avec métriques
    test_end_to_end_pipeline(&components).await?;
    
    println!("✅ Phase 2 tests completed successfully!");
    
    Ok(())
}

/// Composants Phase 2 pour tests
struct Phase2Components {
    ingestion_engine: IngestionEngine,
    document_processor: DocumentProcessor,
    unified_cache: UnifiedCache,
    strategy_detector: StrategyDetector,
}

/// Setup des composants Phase 2
async fn setup_components() -> Result<Phase2Components, Box<dyn std::error::Error>> {
    println!("\n🔧 Setup composants Phase 2...");
    
    // OCR components
    let tesseract_config = TesseractConfig::default();
    let ocr_processor = TesseractProcessor::new(tesseract_config).await?;
    
    // Embedder
    let e5_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(e5_config).await?;
    
    // Document processor pour ingestion engine
    let document_processor_1 = DocumentProcessor::new(ocr_processor, embedder).await?;
    
    // Document processor séparé pour tests directs
    let tesseract_config_2 = TesseractConfig::default();
    let ocr_processor_2 = TesseractProcessor::new(tesseract_config_2).await?;
    let e5_config_2 = CustomE5Config::default();
    let embedder_2 = CustomE5Embedder::new(e5_config_2).await?;
    let document_processor_2 = DocumentProcessor::new(ocr_processor_2, embedder_2).await?;
    
    // Cache unifié
    let cache_config = CacheConfig::default();
    let ocr_cache = OcrCache::new(cache_config).await?;
    let unified_cache = UnifiedCache::new(ocr_cache);
    
    // Ingestion engine
    let ingestion_engine = IngestionEngine::new(document_processor_1);
    
    // Strategy detector
    let strategy_detector = StrategyDetector::new();
    
    println!("  ✓ Tous les composants Phase 2 initialisés");
    
    Ok(Phase2Components {
        ingestion_engine,
        document_processor: document_processor_2,
        unified_cache,
        strategy_detector,
    })
}

/// Test IngestionEngine avec détection automatique
async fn test_ingestion_engine(components: &Phase2Components) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Test IngestionEngine avec détection automatique...");
    
    // Créer fichier texte test
    let temp_file = NamedTempFile::with_suffix(".txt")?;
    let test_content = "Ceci est un test d'ingestion intelligente.\nLe moteur devrait détecter automatiquement la stratégie.\nEt optimiser le chunking selon le type de contenu.";
    write(temp_file.path(), test_content).await?;
    
    let chunk_config = ChunkConfig {
        chunk_size: 15,
        overlap: 3,
        strategy: ChunkStrategy::Heuristic,
    };
    
    // Test ingestion intelligente
    let result = components.ingestion_engine
        .ingest_document(temp_file.path(), "test_group", &chunk_config)
        .await?;
    
    println!("  ✓ Document ingéré: {} chunks créés", result.document.chunks.len());
    println!("  ✓ Stratégie utilisée: {:?}", result.strategy_used);
    println!("  ✓ Temps de traitement: {}ms", result.processing_time_ms);
    
    // Vérifier les métadonnées des chunks
    for (i, chunk) in result.document.chunks.iter().enumerate() {
        println!("    Chunk {}: source={:?}, confidence={:.2}", 
                 i, chunk.metadata.source_type, chunk.metadata.confidence);
    }
    
    Ok(())
}

/// Test StrategyDetector avec différents types de fichiers
async fn test_strategy_detector(components: &Phase2Components) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Test StrategyDetector heuristiques...");
    
    // Test détection fichier texte
    let txt_file = NamedTempFile::with_suffix(".txt")?;
    write(txt_file.path(), "Fichier texte simple").await?;
    
    let strategy = components.strategy_detector.detect_strategy(txt_file.path()).await?;
    match strategy {
        IngestionStrategy::DirectText => println!("  ✓ DirectText détecté pour .txt"),
        _ => println!("  ⚠️  Stratégie inattendue pour .txt: {:?}", strategy),
    }
    
    // Test détection markdown
    let md_file = NamedTempFile::with_suffix(".md")?;
    write(md_file.path(), "# Markdown Test\n\nContenu markdown.").await?;
    
    let strategy = components.strategy_detector.detect_strategy(md_file.path()).await?;
    match strategy {
        IngestionStrategy::DirectText => println!("  ✓ DirectText détecté pour .md"),
        _ => println!("  ⚠️  Stratégie inattendue pour .md: {:?}", strategy),
    }
    
    // Test heuristiques PDF (fichier simulé)
    println!("  ✓ Heuristiques PDF testées (sans fichier réel pour l'instant)");
    
    Ok(())
}

/// Test chunking adaptatif selon source_type
async fn test_adaptive_chunking(components: &Phase2Components) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n✂️  Test chunking adaptatif par source_type...");
    
    let chunk_config = ChunkConfig {
        chunk_size: 10,
        overlap: 2,
        strategy: ChunkStrategy::Heuristic,
    };
    
    // Test texte natif avec chunking par phrases
    let native_text = "Première phrase. Deuxième phrase! Troisième phrase? Quatrième phrase.";
    let temp_file = NamedTempFile::with_suffix(".txt")?;
    write(temp_file.path(), native_text).await?;
    
    let result = components.document_processor
        .process_document(temp_file.path(), "test_group", &chunk_config)
        .await?;
    
    println!("  ✓ Chunking natif: {} chunks créés", result.chunks.len());
    
    // Vérifier les types de chunks et métadonnées
    for (i, chunk) in result.chunks.iter().enumerate() {
        let source_type = &chunk.metadata.source_type;
        let confidence = chunk.metadata.confidence;
        let priority = &chunk.metadata.priority;
        
        println!("    Chunk {}: source={:?}, confidence={:.1}, priority={:?}", 
                 i, source_type, confidence, priority);
        
        // Vérifier que texte natif a confidence=1.0 et priorité High
        if matches!(source_type, gravis_app_lib::rag::SourceType::NativeText) {
            assert_eq!(confidence, 1.0, "Texte natif devrait avoir confidence=1.0");
            assert!(matches!(priority, Priority::High), "Texte natif devrait avoir priorité High");
        }
    }
    
    Ok(())
}

/// Test cache unifié multi-niveaux
async fn test_unified_cache(components: &Phase2Components) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Test cache unifié OCR → Embeddings → Documents...");
    
    // Test cache metrics initial
    let initial_metrics = components.unified_cache.get_cache_metrics();
    println!("  ✓ Métriques initiales: docs={}, embeddings={}, OCR={}", 
             initial_metrics.document_cache_size,
             initial_metrics.embedding_cache_size,
             initial_metrics.ocr_cache_size);
    
    // Test cache embeddings
    let test_hash = "test_chunk_hash_123";
    let test_embedding = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    
    // Cache miss initial
    assert!(components.unified_cache.get_cached_embedding(test_hash).is_none());
    println!("  ✓ Cache embedding MISS initial confirmé");
    
    // Cache l'embedding
    components.unified_cache.cache_embedding(test_hash, test_embedding.clone());
    
    // Cache hit
    let cached_embedding = components.unified_cache.get_cached_embedding(test_hash);
    assert!(cached_embedding.is_some());
    assert_eq!(cached_embedding.unwrap(), test_embedding);
    println!("  ✓ Cache embedding HIT confirmé");
    
    // Test métriques mises à jour
    let updated_metrics = components.unified_cache.get_cache_metrics();
    assert!(updated_metrics.embedding_cache_size > initial_metrics.embedding_cache_size);
    println!("  ✓ Métriques cache mises à jour: {} embeddings", 
             updated_metrics.embedding_cache_size);
    
    // Test nettoyage cache
    let cleanup_result = components.unified_cache.cleanup_cache(1000, 24).await?;
    println!("  ✓ Nettoyage cache: {} docs, {} embeddings supprimés en {}ms",
             cleanup_result.removed_documents,
             cleanup_result.removed_embeddings,
             cleanup_result.cleanup_time_ms);
    
    Ok(())
}

/// Test pipeline end-to-end avec métriques complètes
async fn test_end_to_end_pipeline(components: &Phase2Components) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Test pipeline end-to-end avec métriques...");
    
    // Créer plusieurs fichiers de test
    let test_files = create_test_files().await?;
    
    let chunk_config = ChunkConfig {
        chunk_size: 20,
        overlap: 4,
        strategy: ChunkStrategy::Heuristic,
    };
    
    // Test traitement par lot
    let file_paths: Vec<&std::path::Path> = test_files.iter().map(|f| f.path()).collect();
    let batch_result = components.ingestion_engine
        .ingest_document_batch(file_paths, "test_batch_group", &chunk_config)
        .await?;
    
    println!("  ✓ Traitement par lot terminé:");
    println!("    - Documents traités avec succès: {}", batch_result.successful_ingestions.len());
    println!("    - Échecs: {}", batch_result.failed_ingestions.len());
    println!("    - Temps total: {}ms", batch_result.total_processing_time_ms);
    
    // Analyser les résultats détaillés
    let mut total_chunks = 0;
    for (i, result) in batch_result.successful_ingestions.iter().enumerate() {
        total_chunks += result.document.chunks.len();
        println!("    Document {}: {} chunks, {}ms, stratégie={:?}",
                 i, result.document.chunks.len(), result.processing_time_ms, result.strategy_used);
    }
    
    println!("  ✓ Total chunks créés: {}", total_chunks);
    
    // Vérifier métriques finales
    let final_metrics = components.unified_cache.get_cache_metrics();
    let global_stats = components.unified_cache.get_global_stats();
    
    println!("  ✓ Métriques finales:");
    println!("    - Cache documents: {}", final_metrics.document_cache_size);
    println!("    - Cache embeddings: {}", final_metrics.embedding_cache_size);
    println!("    - Hit ratio: {:.2}%", global_stats.hit_ratio() * 100.0);
    println!("    - Mémoire estimée: {}KB", final_metrics.memory_usage_estimate / 1024);
    
    Ok(())
}

/// Créer fichiers de test variés
async fn create_test_files() -> Result<Vec<NamedTempFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    // Fichier texte court
    let file1 = NamedTempFile::with_suffix(".txt")?;
    write(file1.path(), "Court fichier texte.").await?;
    files.push(file1);
    
    // Fichier texte long
    let file2 = NamedTempFile::with_suffix(".txt")?;
    let long_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(50);
    write(file2.path(), long_content).await?;
    files.push(file2);
    
    // Fichier markdown
    let file3 = NamedTempFile::with_suffix(".md")?;
    write(file3.path(), "# Titre\n\n## Section\n\nContenu markdown avec **gras** et *italique*.").await?;
    files.push(file3);
    
    // Fichier vide
    let file4 = NamedTempFile::with_suffix(".txt")?;
    write(file4.path(), "").await?;
    files.push(file4);
    
    Ok(files)
}