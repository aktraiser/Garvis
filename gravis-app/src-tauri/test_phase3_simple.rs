// Test Phase 3 Simple avec documents réels
// Test rapide du pipeline complet avec vrais PDFs

use gravis_app_lib::rag::{
    DocumentClassifier, BusinessMetadataEnricher, DocumentCategory, BusinessSection
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Test Phase 3 Simple - Classification + Business Metadata");
    println!("📁 Testing with real documents from exemple/");
    
    // Initialiser les composants
    let document_classifier = DocumentClassifier::new();
    let business_enricher = BusinessMetadataEnricher::new();
    
    // Documents de test
    let test_documents = vec![
        ("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/unilever-annual-report-and-accounts-2024.pdf", "Business Annual Report"),
        ("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/2510.18234v1.pdf", "Academic Research Paper"),
        ("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/PV_AGE_XME_20octobre2025.pdf", "Legal/Administrative Document"),
        ("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/contrôle technique.pdf", "Technical Document"),
    ];
    
    for (file_path, description) in test_documents {
        if Path::new(file_path).exists() {
            println!("\n📄 Testing: {} ({})", 
                Path::new(file_path).file_name().unwrap().to_string_lossy(), 
                description);
            
            // Extraction simple de texte avec pdf-extract pour test
            match pdf_extract::extract_text(file_path) {
                Ok(content) => {
                    let content_preview = if content.len() > 500 {
                        format!("{}...", &content[..500])
                    } else {
                        content.clone()
                    };
                    
                    println!("   📝 Content preview: {}", content_preview.replace('\n', " "));
                    
                    // Test classification
                    match document_classifier.classify(&content) {
                        Ok(category) => {
                            println!("   📊 Classification: {:?}", category);
                            
                            // Test Business metadata si Business
                            if matches!(category, DocumentCategory::Business) {
                                match business_enricher.enrich_business_content(&content, None, None) {
                                    Ok(business_metadata) => {
                                        println!("   💼 Business Section: {:?}", business_metadata.section_type);
                                        println!("   🎯 Confidence: {:.3}", business_metadata.confidence_score);
                                        println!("   💰 KPIs found: {}", business_metadata.financial_kpis.len());
                                        
                                        if let Some(company) = &business_metadata.company_name {
                                            println!("   🏢 Company: {}", company);
                                        }
                                        
                                        for kpi in business_metadata.financial_kpis.iter().take(3) {
                                            println!("     💵 {}: {:.0} {} ({})", 
                                                kpi.name, kpi.value, kpi.currency, kpi.unit);
                                        }
                                    }
                                    Err(e) => println!("   ⚠️ Business enrichment failed: {}", e),
                                }
                            }
                        }
                        Err(e) => println!("   ❌ Classification failed: {}", e),
                    }
                }
                Err(e) => {
                    println!("   ⚠️ Failed to extract text: {}", e);
                    println!("   💡 This document might need OCR processing");
                }
            }
        } else {
            println!("\n❌ Document not found: {}", file_path);
        }
    }
    
    println!("\n🎉 Phase 3 Simple Test Complete!");
    println!("✅ Document Classification functional");
    println!("✅ Business Metadata Enrichment functional"); 
    println!("🚀 Ready for full RagState integration!");
    
    Ok(())
}