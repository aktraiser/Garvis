// Test standalone pour DocumentClassifier Phase 3A
use gravis_app_lib::rag::{DocumentClassifier, DocumentCategory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Test DocumentClassifier Phase 3A - Business Detection");
    
    let classifier = DocumentClassifier::new();
    
    // === Test 1: Business Document Classification ===
    println!("\n📊 Test 1: Business Document");
    let business_content = "
        Executive Summary
        
        Our company achieved strong financial performance in FY 2023.
        Revenue increased to $2.1 billion, with EBITDA of $450 million.
        Total Assets reached $3.2 billion.
        
        Management Discussion
        The Board of Directors approved the annual dividend.
    ";

    let doc_type = classifier.classify(business_content)?;
    println!("✅ Classified as: {:?}", doc_type);
    assert_eq!(doc_type, DocumentCategory::Business);

    let signals = classifier.extract_business_signals(business_content)?;
    println!("✅ Executive Summary detected: {}", signals.executive_summary);
    println!("✅ Confidence score: {:.3}", signals.confidence_score);
    println!("✅ Fiscal year: {:?}", signals.fiscal_year);
    println!("✅ Financial metrics: {:?}", signals.financial_metrics);
    
    assert!(signals.executive_summary);
    assert!(signals.confidence_score > 0.6);
    assert_eq!(signals.fiscal_year, Some(2023));
    assert!(!signals.financial_metrics.is_empty());

    // === Test 2: Academic Document Classification ===
    println!("\n📚 Test 2: Academic Document");
    let academic_content = "
        Abstract
        
        This study presents a novel approach to machine learning.
        Previous work by Smith et al. (2020) showed limitations.
        Our methodology improves upon [15] by 15%.
        
        References
        [1] Smith, J. et al. (2020). Machine Learning Advances.
    ";

    let doc_type = classifier.classify(academic_content)?;
    println!("✅ Classified as: {:?}", doc_type);
    assert_eq!(doc_type, DocumentCategory::Academic);

    // === Test 3: Mixed/Unknown Document ===
    println!("\n❓ Test 3: Mixed Document");
    let mixed_content = "
        This document contains various information.
        Some technical details but no clear pattern.
    ";

    let doc_type = classifier.classify(mixed_content)?;
    println!("✅ Classified as: {:?}", doc_type);
    assert_eq!(doc_type, DocumentCategory::Mixed);

    // === Test 4: Fiscal Year Extraction ===
    println!("\n📅 Test 4: Fiscal Year Extraction");
    let test_cases = vec![
        ("FY 2023", Some(2023)),
        ("Fiscal Year 2022", Some(2022)),
        ("Year Ended December 31, 2021", Some(2021)),
        ("Annual Report 2024", Some(2024)),
        ("No year here", None),
    ];

    for (content, expected) in test_cases {
        let result = classifier.business_patterns.extract_fiscal_year(content);
        println!("✅ '{}' → {:?}", content, result);
        assert_eq!(result, expected, "Failed for content: {}", content);
    }

    // === Test 5: Confidence Scoring ===
    println!("\n🎯 Test 5: Confidence Scoring");
    
    // Test score élevé
    let high_score = classifier.calculate_business_confidence_score(
        5,  // section_matches
        &vec!["Revenue".to_string(), "EBITDA".to_string()], // financial_metrics
        &vec!["Corp.".to_string()], // company_identifiers
        true, // has_executive_summary
        true, // has_fiscal_year
    );
    println!("✅ High confidence score: {:.3}", high_score);
    assert!(high_score > 0.8);

    // Test score faible
    let low_score = classifier.calculate_business_confidence_score(
        0, &vec![], &vec![], false, false
    );
    println!("✅ Low confidence score: {:.3}", low_score);
    assert!(low_score < 0.1);

    println!("\n🎉 Tous les tests DocumentClassifier Phase 3A passent !");
    println!("📈 Module prêt pour intégration Universal RAG Pipeline");
    
    Ok(())
}