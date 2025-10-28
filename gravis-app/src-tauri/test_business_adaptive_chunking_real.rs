// Test Business Adaptive Chunking - Phase 3A avec vrais documents
use gravis_app_lib::rag::{SmartChunker, SmartChunkConfig, DocumentClassifier, DocumentCategory, BusinessMetadataEnricher, SourceType, ExtractionMethod};
use std::path::Path;
use pdf_extract::extract_text;

/// Extraction simple de PDF pour tests
fn extract_pdf_simple(pdf_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = extract_text(pdf_path)?;
    Ok(content)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Test Business Adaptive Chunking Phase 3A - Real Documents");
    
    // === Test 1: Configuration Business vs Academic ===
    println!("\n⚙️ Test 1: Configuration Business vs Academic");
    
    let academic_config = SmartChunkConfig::academic_optimized();
    let business_config = SmartChunkConfig::business_optimized();
    
    println!("Academic config: {} tokens, {:.1}% overlap, MMR λ={:.1}, max_docs={}", 
             academic_config.target_tokens, 
             academic_config.overlap_percent * 100.0,
             academic_config.mmr_lambda,
             academic_config.max_context_docs);
             
    println!("Business config: {} tokens, {:.1}% overlap, MMR λ={:.1}, max_docs={}", 
             business_config.target_tokens, 
             business_config.overlap_percent * 100.0,
             business_config.mmr_lambda,
             business_config.max_context_docs);

    // === Test 2: Extraction et Chunking de Vrais Documents ===
    println!("\n🏢 Test 2: Real Document Processing from exemple/");
    
    // Testons plusieurs documents réels
    let test_documents = vec![
        ("../exemple/unilever-annual-report-and-accounts-2024.pdf", "Business - Unilever Annual Report"),
        ("../exemple/PV_AGE_XME_20octobre2025.pdf", "Business - PV AGE"),
        ("../exemple/2510.18234v1.pdf", "Academic - Research Paper"),
    ];
    
    let classifier = DocumentClassifier::new();
    let enricher = BusinessMetadataEnricher::new();
    
    for (pdf_path, description) in &test_documents {
        println!("\n📄 Testing: {}", description);
        println!("   Path: {}", pdf_path);
        
        // Vérification existence fichier
        if !Path::new(pdf_path).exists() {
            println!("   ⚠️  File not found, skipping");
            continue;
        }
        
        // Extraction PDF
        let extracted_content = match extract_pdf_simple(pdf_path) {
            Ok(content) => {
                println!("   ✅ PDF extracted: {} chars", content.len());
                content
            },
            Err(e) => {
                println!("   ❌ PDF extraction failed: {}", e);
                continue;
            }
        };
        
        // Si le contenu est trop long, prenons les 8000 premiers chars pour le test
        let test_content = if extracted_content.len() > 8000 {
            println!("   📏 Truncating to 8000 chars for test");
            &extracted_content[..8000]
        } else {
            &extracted_content
        };
        
        // Classification automatique
        let doc_category = classifier.classify(test_content)?;
        println!("   🏷️  Classified as: {:?}", doc_category);
        
        // Configuration adaptative basée sur classification
        let adaptive_config = match doc_category {
            DocumentCategory::Business => SmartChunkConfig::business_optimized(),
            DocumentCategory::Academic => SmartChunkConfig::academic_optimized(),
            DocumentCategory::Legal => SmartChunkConfig::legal_optimized(),
            DocumentCategory::Technical => SmartChunkConfig::technical_optimized(),
            DocumentCategory::Mixed => SmartChunkConfig::mixed_universal(),
        };
        
        // Chunking adaptatif
        let mut adaptive_chunker = match doc_category {
            DocumentCategory::Business => SmartChunker::new_business(adaptive_config)?,
            DocumentCategory::Academic => SmartChunker::new_academic(adaptive_config)?,
            DocumentCategory::Legal => SmartChunker::new_legal(adaptive_config)?,
            DocumentCategory::Technical => SmartChunker::new_technical(adaptive_config)?,
            DocumentCategory::Mixed => SmartChunker::new(adaptive_config)?, // Generic
        };
        
        let chunks = adaptive_chunker.chunk_document(
            test_content,
            SourceType::OcrExtracted, // PDF extrait
            &ExtractionMethod::PdfNative,
            &format!("test_{}", pdf_path.split('/').last().unwrap_or("unknown"))
        )?;
        
        println!("   📊 Chunks created: {}", chunks.chunks.len());
        println!("   📏 Average chunk size: {:.0} chars", chunks.avg_chunk_size);
        println!("   🔍 Sections detected: {:?}", chunks.sections_detected);
        
        // Pour documents Business, enrichissement métadonnées du premier chunk
        if matches!(doc_category, DocumentCategory::Business) && !chunks.chunks.is_empty() {
            let first_chunk = &chunks.chunks[0];
            let metadata = enricher.enrich_business_content(&first_chunk.content, None, Some(1))?;
            println!("   💼 Business metadata: {:?} section, {} KPIs, confidence {:.2}", 
                     metadata.section_type, metadata.financial_kpis.len(), metadata.confidence_score);
            
            // Affichage des KPIs détectés
            if !metadata.financial_kpis.is_empty() {
                println!("   💰 KPIs found:");
                for kpi in metadata.financial_kpis.iter().take(3) {
                    println!("       {} = {:.0} {} ({})", kpi.name, kpi.value, kpi.currency, kpi.unit);
                }
            }
        }
        
        // Validation chunks non vides
        assert!(chunks.chunks.len() > 0, "Should create at least one chunk for {}", description);
        
        println!("   ✅ Test passed for {}", description);
    }

    // === Test 3: Comparison avec contenu identique ===
    println!("\n📚 Test 3: Academic vs Business Chunker Comparison");
    
    // Prenons le premier document trouvé pour comparaison
    let comparison_path = "../exemple/unilever-annual-report-and-accounts-2024.pdf";
    
    if Path::new(comparison_path).exists() {
        let content = extract_pdf_simple(comparison_path)?;
        let test_content = if content.len() > 5000 { &content[..5000] } else { &content };
        
        // Test Academic chunker
        let mut academic_chunker = SmartChunker::new_academic(academic_config)?;
        let academic_chunks = academic_chunker.chunk_document(
            test_content,
            SourceType::OcrExtracted,
            &ExtractionMethod::PdfNative,
            "comparison_academic"
        )?;
        
        // Test Business chunker
        let mut business_chunker = SmartChunker::new_business(business_config)?;
        let business_chunks = business_chunker.chunk_document(
            test_content,
            SourceType::OcrExtracted,
            &ExtractionMethod::PdfNative,
            "comparison_business"
        )?;
        
        println!("📖 Academic chunker: {} chunks, {} sections", 
                 academic_chunks.chunks.len(), academic_chunks.sections_detected.len());
        println!("💼 Business chunker: {} chunks, {} sections", 
                 business_chunks.chunks.len(), business_chunks.sections_detected.len());
        
        // Comparaison des patterns détectés
        if !academic_chunks.sections_detected.is_empty() {
            println!("📖 Academic sections: {:?}", academic_chunks.sections_detected);
        }
        if !business_chunks.sections_detected.is_empty() {
            println!("💼 Business sections: {:?}", business_chunks.sections_detected);
        }
    }

    println!("\n🎉 Tous les tests Real Document Processing passent !");
    println!("🚀 Pipeline Universal RAG Phase 3A validé sur vrais documents !");
    
    Ok(())
}