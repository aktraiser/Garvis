// Test Unicode Normalization - Phase 3A
// Test des fonctionnalités de sanitization des ligatures PDF

use gravis_app_lib::rag::{sanitize_pdf_text, detect_ligatures, clean_extracted_text};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 Test Unicode Normalization Phase 3A - Ligature Sanitization");
    
    // === Test 1: Détection et remplacement des ligatures communes ===
    println!("\n📝 Test 1: Common Ligatures Detection & Replacement");
    
    let ligature_text = "The ﬁrst ﬂoor oﬃce has ﬀ and ﬃ ligatures in the ﬁle.pdf";
    println!("   Original: {}", ligature_text);
    
    // Détection des ligatures
    let detections = detect_ligatures(ligature_text);
    println!("   Ligatures detected: {}", detections.len());
    for (ligature, position, replacement) in &detections {
        println!("     Position {}: '{}' → '{}'", position, ligature, replacement);
    }
    
    // Sanitization complète
    let (sanitized, stats) = sanitize_pdf_text(ligature_text)?;
    println!("   Sanitized: {}", sanitized);
    println!("   Stats: {} total chars, {} ligatures replaced", stats.total_chars, stats.ligatures_replaced);
    
    // Vérifications
    assert_eq!(sanitized, "The first floor office has ff and ffi ligatures in the file.pdf");
    assert_eq!(stats.ligatures_replaced, 6);
    assert!(stats.unicode_normalized);

    // === Test 2: Ligatures françaises ===
    println!("\n🇫🇷 Test 2: French Ligatures");
    
    let french_text = "Œuvre complète avec des ﬁnitions et ﬂux";
    println!("   Original: {}", french_text);
    
    let (french_sanitized, french_stats) = sanitize_pdf_text(french_text)?;
    println!("   Sanitized: {}", french_sanitized);
    
    assert_eq!(french_sanitized, "OEuvre complète avec des finitions et flux");
    assert_eq!(french_stats.ligatures_replaced, 3);

    // === Test 3: Nettoyage texte OCR/PDF complet ===
    println!("\n🧹 Test 3: Complete OCR/PDF Text Cleaning");
    
    let messy_ocr_text = "  The   ﬁrst  ﬂoor   has  \"strange\"  characters–—  and   multiple   spaces  ";
    println!("   Messy OCR: '{}'", messy_ocr_text);
    
    let cleaned = clean_extracted_text(messy_ocr_text)?;
    println!("   Cleaned: '{}'", cleaned);
    
    assert_eq!(cleaned, "The first floor has \"strange\" characters-- and multiple spaces");

    // === Test 4: Cas limites ===
    println!("\n⚡ Test 4: Edge Cases");
    
    // Texte vide
    let (empty_result, empty_stats) = sanitize_pdf_text("")?;
    assert_eq!(empty_result, "");
    assert_eq!(empty_stats.ligatures_replaced, 0);
    println!("   ✅ Empty string handled");
    
    // Texte sans ligatures
    let normal_text = "Normal text without ligatures";
    let (normal_result, normal_stats) = sanitize_pdf_text(normal_text)?;
    assert_eq!(normal_result, normal_text);
    assert_eq!(normal_stats.ligatures_replaced, 0);
    println!("   ✅ Normal text unchanged");
    
    // Ligatures uniquement
    let ligatures_only = "ﬁﬂﬃﬄﬀ";
    let (ligatures_result, ligatures_stats) = sanitize_pdf_text(ligatures_only)?;
    assert_eq!(ligatures_result, "fiflffifflff");
    assert_eq!(ligatures_stats.ligatures_replaced, 5);
    println!("   ✅ Ligatures-only text processed");

    // === Test 5: Test avec des PDFs réels (si disponibles) ===
    println!("\n📄 Test 5: Real PDF Content Simulation");
    
    let academic_pdf_text = "
        Abstract
        
        This paper presents a novel approach to machine learning classification.
        The methodology involves sophisticated algorithms with finalized parameters.
        Our findings show significant improvements in efficiency.
        
        References
        [1] Smith, J. et al. (2020). Advanced Classification Methods.
    ";
    
    // Le texte académique ne devrait pas avoir de ligatures (pas d'OCR)
    let (academic_clean, academic_stats) = sanitize_pdf_text(academic_pdf_text)?;
    println!("   Academic PDF cleaned ({} chars → {} chars)", 
             academic_pdf_text.len(), academic_clean.len());
    assert_eq!(academic_stats.ligatures_replaced, 0);
    
    let business_ocr_text = "
        Executive Summary
        
        Our company achieved strong ﬁnancial performance in ﬁscal year 2023.
        Revenue increased signiﬁcantly with proﬁtable operations across all divisions.
        The Board of Directors conﬁrmed the ﬁnal dividend payment.
    ";
    
    let (business_clean, business_stats) = sanitize_pdf_text(business_ocr_text)?;
    println!("   Business OCR cleaned: {} ligatures replaced", business_stats.ligatures_replaced);
    assert!(business_stats.ligatures_replaced > 0);
    assert!(business_clean.contains("financial"));
    assert!(business_clean.contains("fiscal"));
    assert!(business_clean.contains("significantly"));

    // === Test 6: Performance sur gros texte ===
    println!("\n⚡ Test 6: Performance on Large Text");
    
    let large_text = "The ﬁrst ﬂoor oﬃce ".repeat(1000); // 20,000 chars avec ligatures
    let start = std::time::Instant::now();
    let (large_result, large_stats) = sanitize_pdf_text(&large_text)?;
    let duration = start.elapsed();
    
    println!("   Large text: {} chars processed in {:?}", large_text.len(), duration);
    println!("   Ligatures replaced: {} (expected: {})", large_stats.ligatures_replaced, 3000);
    assert_eq!(large_stats.ligatures_replaced, 3000); // 3 ligatures × 1000 répétitions
    assert!(duration.as_millis() < 100); // Devrait être rapide
    
    println!("\n🎉 Tous les tests Unicode Normalization Phase 3A passent !");
    println!("🚀 Module sanitize_pdf_text prêt pour intégration Universal RAG Pipeline !");
    
    Ok(())
}