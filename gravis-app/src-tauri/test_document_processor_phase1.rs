// Test d'intégration DocumentProcessor - Phase 1
// Validation des structures étendues et pipeline de base

use tempfile::NamedTempFile;
use tokio::fs::write;

use gravis_app_lib::rag::{
    DocumentProcessor, SourceType, ExtractionMethod, PdfStrategy, DocumentType,
    ChunkMetadata, ChunkConfig, Priority, ChunkStrategy
};
use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig
};
use gravis_app_lib::rag::custom_e5::{CustomE5Embedder, CustomE5Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Test DocumentProcessor Phase 1 - Intégration OCR-RAG");
    
    // 1. Setup composants
    let tesseract_config = TesseractConfig::default();
    let ocr_processor = TesseractProcessor::new(tesseract_config).await?;
    
    let e5_config = CustomE5Config::default();
    let embedder = CustomE5Embedder::new(e5_config).await?;
    
    let processor = DocumentProcessor::new(ocr_processor, embedder).await?;
    
    // 2. Test structures étendues
    test_chunk_metadata_extended().await?;
    test_document_type_strategies().await?;
    
    // 3. Test pipeline de base avec fichier texte
    test_document_processing_text(&processor).await?;
    
    // 4. Test détection format (sans OCR complet pour l'instant)
    test_format_detection(&processor).await?;
    
    println!("✅ Phase 1 tests completed successfully!");
    
    Ok(())
}

/// Test des nouvelles structures ChunkMetadata avec OCR
async fn test_chunk_metadata_extended() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Test ChunkMetadata étendu avec OCR...");
    
    // Test SourceType enum
    let source_types = vec![
        SourceType::NativeText,
        SourceType::OcrExtracted,
        SourceType::HybridPdfNative,
        SourceType::HybridPdfOcr,
    ];
    
    for source_type in source_types {
        println!("  ✓ SourceType: {:?}", source_type);
    }
    
    // Test ExtractionMethod enum
    let extraction_methods = vec![
        ExtractionMethod::DirectRead,
        ExtractionMethod::TesseractOcr { 
            confidence: 0.85, 
            language: "fra".to_string() 
        },
        ExtractionMethod::PdfNative,
        ExtractionMethod::PdfOcrFallback,
        ExtractionMethod::HybridIntelligent,
    ];
    
    for method in extraction_methods {
        println!("  ✓ ExtractionMethod: {:?}", method);
    }
    
    // Test ChunkMetadata avec nouveaux champs
    let metadata = ChunkMetadata {
        tags: vec!["test".to_string()],
        priority: Priority::Normal,
        language: "fra".to_string(),
        symbol: None,
        context: None,
        confidence: 0.9,
        // Nouveaux champs OCR
        ocr_metadata: None,
        source_type: SourceType::NativeText,
        extraction_method: ExtractionMethod::DirectRead,
    };
    
    println!("  ✓ ChunkMetadata étendu créé: confidence={}, source={:?}", 
             metadata.confidence, metadata.source_type);
    
    Ok(())
}

/// Test des nouvelles stratégies DocumentType
async fn test_document_type_strategies() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📄 Test DocumentType avec stratégies PDF...");
    
    // Test PDF avec différentes stratégies
    let pdf_strategies = vec![
        DocumentType::PDF {
            extraction_strategy: PdfStrategy::NativeOnly,
            native_text_ratio: 0.95,
            ocr_pages: vec![],
            total_pages: 5,
        },
        DocumentType::PDF {
            extraction_strategy: PdfStrategy::OcrOnly,
            native_text_ratio: 0.0,
            ocr_pages: vec![0, 1, 2, 3, 4],
            total_pages: 5,
        },
        DocumentType::PDF {
            extraction_strategy: PdfStrategy::HybridIntelligent,
            native_text_ratio: 0.6,
            ocr_pages: vec![2, 4],
            total_pages: 5,
        },
    ];
    
    for (i, doc_type) in pdf_strategies.iter().enumerate() {
        println!("  ✓ Stratégie PDF {}: {:?}", i+1, doc_type);
    }
    
    Ok(())
}

/// Test traitement document texte simple
async fn test_document_processing_text(processor: &DocumentProcessor) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Test traitement document texte...");
    
    // Créer fichier texte temporaire avec extension
    let temp_file = NamedTempFile::with_suffix(".txt")?;
    let temp_path = temp_file.path();
    
    let test_content = "Ceci est un document de test.\nIl contient plusieurs lignes.\nPour tester le chunking adaptatif.";
    write(temp_path, test_content).await?;
    
    // Configuration chunking
    let chunk_config = ChunkConfig {
        chunk_size: 10, // Petite taille pour tester
        overlap: 2,
        strategy: ChunkStrategy::Heuristic,
    };
    
    // Traitement du document
    let result = processor.process_document(temp_path, "test_group", &chunk_config).await?;
    
    println!("  ✓ Document traité: {} chunks créés", result.chunks.len());
    println!("  ✓ Type document: {:?}", result.document_type);
    println!("  ✓ Contenu normalisé: {} caractères", result.content.len());
    
    // Vérifier les chunks
    for (i, chunk) in result.chunks.iter().enumerate() {
        println!("    Chunk {}: source={:?}, method={:?}, confidence={}", 
                 i, 
                 chunk.metadata.source_type,
                 chunk.metadata.extraction_method,
                 chunk.metadata.confidence);
    }
    
    Ok(())
}

/// Test détection de format de fichier
async fn test_format_detection(processor: &DocumentProcessor) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Test détection format fichier...");
    
    // Test fichier markdown
    let md_file = NamedTempFile::with_suffix(".md")?;
    write(md_file.path(), "# Test Markdown\n\nContenu test.").await?;
    
    let chunk_config = ChunkConfig::default();
    let result = processor.process_document(md_file.path(), "test_group", &chunk_config).await?;
    
    match result.document_type {
        DocumentType::Markdown => println!("  ✓ Markdown détecté correctement"),
        _ => println!("  ⚠️  Markdown non détecté: {:?}", result.document_type),
    }
    
    // Test fichier texte simple
    let txt_file = NamedTempFile::with_suffix(".txt")?;
    write(txt_file.path(), "Fichier texte simple.").await?;
    
    let result = processor.process_document(txt_file.path(), "test_group", &chunk_config).await?;
    
    match result.document_type {
        DocumentType::PlainText => println!("  ✓ PlainText détecté correctement"),
        _ => println!("  ⚠️  PlainText non détecté: {:?}", result.document_type),
    }
    
    println!("  ✓ Tests de détection format terminés");
    
    Ok(())
}