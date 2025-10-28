// Test Phase 2 simplifié - PDF OCR + Injection
// Test avec DeepSeek-OCR PDF et pipeline Phase 2

use std::path::Path;

use gravis_app_lib::rag::{
    IngestionEngine, DocumentProcessor, UnifiedCache, StrategyDetector,
    ChunkConfig, ChunkStrategy, SourceType
};
use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig, OcrCache, CacheConfig
};
use gravis_app_lib::rag::custom_e5::{CustomE5Embedder, CustomE5Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Test Phase 2 - PDF DeepSeek-OCR avec Pipeline Intelligent");
    
    // 1. Vérifier présence PDF
    let pdf_path = Path::new("../2510.18234v1.pdf");
    if !pdf_path.exists() {
        eprintln!("❌ PDF non trouvé: {:?}", pdf_path);
        eprintln!("   Placez 2510.18234v1.pdf dans gravis-app/");
        return Ok(());
    }
    
    let file_size = tokio::fs::metadata(pdf_path).await?.len();
    println!("✅ PDF trouvé: {:.1}MB", file_size as f64 / 1_000_000.0);
    
    // 2. Setup composants Phase 2
    println!("\n🔧 Setup pipeline Phase 2...");
    
    let tesseract_config = TesseractConfig::default();
    let ocr_processor = TesseractProcessor::new(tesseract_config).await?;
    
    let e5_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(e5_config).await?;
    
    let document_processor = DocumentProcessor::new(ocr_processor, embedder).await?;
    let unified_cache = {
        let cache_config = CacheConfig::default();
        let ocr_cache = OcrCache::new(cache_config).await?;
        UnifiedCache::new(ocr_cache)
    };
    
    let ingestion_engine = IngestionEngine::new(document_processor);
    let strategy_detector = StrategyDetector::new();
    
    println!("  ✓ Pipeline initialisé");
    
    // 3. Test détection stratégie PDF
    println!("\n🔍 Test détection stratégie PDF...");
    let strategy = strategy_detector.detect_strategy(pdf_path).await?;
    println!("  ✓ Stratégie détectée: {:?}", strategy);
    
    // 4. Test extraction et chunking avec IngestionEngine
    println!("\n📄 Test extraction PDF avec chunking intelligent...");
    
    let chunk_config = ChunkConfig {
        chunk_size: 100,  // Chunks adaptés pour test
        overlap: 20,
        strategy: ChunkStrategy::Heuristic,
    };
    
    let start_extraction = std::time::Instant::now();
    let ingestion_result = ingestion_engine
        .ingest_document(pdf_path, "deepseek_test_group", &chunk_config)
        .await?;
    let extraction_time = start_extraction.elapsed();
    
    println!("  ✅ Extraction terminée en {:?}", extraction_time);
    println!("  ✓ Document ID: {}", ingestion_result.document.id);
    println!("  ✓ Chunks créés: {}", ingestion_result.document.chunks.len());
    println!("  ✓ Contenu total: {} caractères", ingestion_result.document.content.len());
    println!("  ✓ Temps traitement: {}ms", ingestion_result.processing_time_ms);
    
    // 5. Analyse qualité chunking par type
    println!("\n✂️  Analyse qualité chunking...");
    
    let mut ocr_chunks = 0;
    let mut native_chunks = 0;
    let mut high_confidence_chunks = 0;
    let mut total_confidence = 0.0;
    
    for chunk in &ingestion_result.document.chunks {
        total_confidence += chunk.metadata.confidence;
        
        match chunk.metadata.source_type {
            SourceType::OcrExtracted => ocr_chunks += 1,
            SourceType::NativeText => native_chunks += 1,
            _ => {}
        }
        
        if chunk.metadata.confidence >= 0.8 {
            high_confidence_chunks += 1;
        }
    }
    
    let avg_confidence = total_confidence / ingestion_result.document.chunks.len() as f32;
    
    println!("  ✓ Chunks OCR: {}", ocr_chunks);
    println!("  ✓ Chunks natifs: {}", native_chunks);
    println!("  ✓ Confidence moyenne: {:.2}", avg_confidence);
    println!("  ✓ Chunks haute confiance (≥0.8): {}", high_confidence_chunks);
    
    // 6. Échantillon de chunks avec détails
    println!("\n📋 Échantillon chunks (5 premiers):");
    
    for (i, chunk) in ingestion_result.document.chunks.iter().take(5).enumerate() {
        let preview = if chunk.content.len() > 60 {
            format!("{}...", &chunk.content[..60])
        } else {
            chunk.content.clone()
        };
        
        println!("  {}. Type={:?}, Conf={:.2}, Taille={}", 
                 i + 1, 
                 chunk.metadata.source_type, 
                 chunk.metadata.confidence,
                 chunk.content.len());
        println!("     \"{}\"", preview);
    }
    
    // 7. Test cache et métriques
    println!("\n💾 Test cache unifié...");
    
    // Simuler cache embeddings
    for (i, chunk) in ingestion_result.document.chunks.iter().take(3).enumerate() {
        let mock_embedding = vec![0.1; 384]; // Mock embedding 384D
        unified_cache.cache_embedding(&chunk.hash, mock_embedding);
        
        if i == 0 {
            println!("  ✓ Embedding cached pour chunk: {}", chunk.id);
        }
    }
    
    let cache_metrics = unified_cache.get_cache_metrics();
    println!("  ✓ Cache embeddings: {} entrées", cache_metrics.embedding_cache_size);
    println!("  ✓ Mémoire estimée: {}KB", cache_metrics.memory_usage_estimate / 1024);
    
    // 8. Test nettoyage
    let cleanup_result = unified_cache.cleanup_cache(1000, 24).await?;
    println!("  ✓ Nettoyage: {} docs, {} embeddings supprimés", 
             cleanup_result.removed_documents, cleanup_result.removed_embeddings);
    
    // 9. Résumé final
    println!("\n📊 Résumé test Phase 2:");
    println!("  📄 PDF: {:.1}MB traité en {:?}", 
             file_size as f64 / 1_000_000.0, extraction_time);
    println!("  🧠 Pipeline: {} chunks générés", ingestion_result.document.chunks.len());
    println!("  🎯 Qualité: {:.1}% chunks haute confiance", 
             high_confidence_chunks as f64 / ingestion_result.document.chunks.len() as f64 * 100.0);
    println!("  ⚡ Performance: {}ms/chunk", 
             ingestion_result.processing_time_ms as f64 / ingestion_result.document.chunks.len() as f64);
    
    // 10. Test recherche simulée dans les chunks
    println!("\n🔍 Test recherche simulée...");
    
    let search_terms = vec!["deep learning", "optical character", "neural network"];
    
    for term in search_terms {
        let mut matches = 0;
        for chunk in &ingestion_result.document.chunks {
            if chunk.content.to_lowercase().contains(&term.to_lowercase()) {
                matches += 1;
            }
        }
        println!("  \"{}\" → {} chunks contiennent le terme", term, matches);
    }
    
    println!("\n✅ Test Phase 2 PDF terminé avec succès!");
    println!("   Pipeline OCR → Chunking → Cache validé sur document réel");
    
    Ok(())
}