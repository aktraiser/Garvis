// Test Business Metadata Enrichment - Phase 3A
use gravis_app_lib::rag::{BusinessMetadataEnricher, BusinessSection, FinancialKPI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💼 Test BusinessMetadata Enrichment Phase 3A");
    
    let enricher = BusinessMetadataEnricher::new();
    
    // === Test 1: Complete Business Document ===
    println!("\n📊 Test 1: Complete Business Document Enrichment");
    let business_content = "
        Executive Summary
        
        Microsoft Corporation achieved outstanding financial performance in FY 2023.
        Our technology company delivered revenue of $2.1 billion, up from $1.8 billion.
        EBITDA reached $450 million, representing strong operational efficiency.
        Total Assets increased to $3.2 billion, reflecting our growth strategy.
        
        Financial Highlights
        Net Income for the year was $320 million.
        Market Capitalization reached $1.5 billion as of year-end.
    ";

    let metadata = enricher.enrich_business_content(business_content, Some(2023), Some(1))?;
    
    println!("✅ Fiscal Year: {:?}", metadata.fiscal_year);
    println!("✅ Section Type: {:?}", metadata.section_type);
    println!("✅ Company Name: {:?}", metadata.company_name);
    println!("✅ Sector: {:?}", metadata.sector);
    println!("✅ Confidence Score: {:.3}", metadata.confidence_score);
    println!("✅ Financial KPIs extracted: {}", metadata.financial_kpis.len());
    
    for kpi in &metadata.financial_kpis {
        println!("  📈 {} = {:.0} {} ({})", kpi.name, kpi.value, kpi.currency, kpi.unit);
    }
    
    // Validations
    assert_eq!(metadata.fiscal_year, Some(2023));
    assert!(matches!(metadata.section_type, BusinessSection::ExecutiveSummary));
    assert!(metadata.company_name.is_some());
    assert!(metadata.sector.is_some());
    assert!(metadata.financial_kpis.len() >= 3); // Au moins 3 KPIs détectés
    assert!(metadata.confidence_score > 0.7);

    // === Test 2: Financial KPI Extraction Precision ===
    println!("\n💰 Test 2: Financial KPI Extraction");
    let kpi_content = "Revenue increased to $2.1 billion and EBITDA of $450 million";
    let kpis = enricher.kpi_extractor.extract_kpis(kpi_content, Some(2023))?;
    
    println!("✅ KPIs extracted: {}", kpis.len());
    assert!(kpis.len() >= 2);

    let revenue_kpi = kpis.iter().find(|k| k.name == "Revenue").unwrap();
    println!("✅ Revenue: {} {}", revenue_kpi.value, revenue_kpi.currency);
    assert_eq!(revenue_kpi.value, 2_100_000_000.0);
    assert_eq!(revenue_kpi.currency, "USD");

    // === Test 3: Section Classification ===
    println!("\n📋 Test 3: Section Classification");
    let test_sections = vec![
        ("Executive Summary: This year we achieved strong results", BusinessSection::ExecutiveSummary),
        ("Financial Highlights show record performance", BusinessSection::FinancialHighlights),
        ("Business Overview of our operations", BusinessSection::BusinessOverview),
        ("Risk Factors that may impact us", BusinessSection::RiskFactors),
        ("Market Analysis reveals opportunities", BusinessSection::MarketAnalysis),
        ("Some random content", BusinessSection::Unknown),
    ];

    for (content, expected) in test_sections {
        let section = enricher.section_classifier.classify_section(content)?;
        println!("✅ '{}' → {:?}", content.chars().take(30).collect::<String>(), section);
        assert!(matches!(section, expected));
    }

    // === Test 4: Company & Sector Extraction ===
    println!("\n🏢 Test 4: Company & Sector Extraction");
    let company_content = "Apple Inc. is a leading technology company in digital innovation";
    let company = enricher.company_extractor.extract_company_name(company_content);
    let sector = enricher.company_extractor.extract_sector(company_content);
    
    println!("✅ Company: {:?}", company);
    println!("✅ Sector: {:?}", sector);
    assert!(company.is_some());
    assert_eq!(sector, Some("Technology".to_string()));

    // === Test 5: Multiple Currency Detection ===
    println!("\n💱 Test 5: Currency Detection");
    let multi_currency_content = "Revenue €1.5 billion in Europe, $2.1 billion in USA";
    let currency = enricher.kpi_extractor.extract_currency(multi_currency_content);
    println!("✅ First currency detected: {:?}", currency);
    assert!(currency.is_some());

    // === Test 6: Confidence Scoring ===
    println!("\n🎯 Test 6: Confidence Scoring Validation");
    
    // High confidence: many KPIs + clear section + company
    let high_conf_metadata = enricher.enrich_business_content(
        "Executive Summary: Microsoft Corporation revenue $2B, EBITDA $500M, Net Income $300M",
        Some(2023),
        Some(1)
    )?;
    println!("✅ High confidence scenario: {:.3}", high_conf_metadata.confidence_score);
    assert!(high_conf_metadata.confidence_score > 0.8);
    
    // Low confidence: minimal info
    let low_conf_metadata = enricher.enrich_business_content(
        "Some general business information without specific details",
        None,
        None
    )?;
    println!("✅ Low confidence scenario: {:.3}", low_conf_metadata.confidence_score);
    assert!(low_conf_metadata.confidence_score < 0.3);

    println!("\n🎉 Tous les tests BusinessMetadata enrichment passent !");
    println!("📈 Module prêt pour intégration avec DocumentClassifier");
    
    Ok(())
}