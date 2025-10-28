// GRAVIS Business Metadata Enrichment - Phase 3A
// Module d'enrichissement des métadonnées pour documents Business

use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::Result;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Métadonnées enrichies spécifiques aux documents Business
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetadata {
    pub fiscal_year: Option<i32>,
    pub company_name: Option<String>,
    pub sector: Option<String>,
    pub financial_kpis: Vec<FinancialKPI>,
    pub section_type: BusinessSection,
    pub page_number: Option<u32>,
    pub bbox: Option<BoundingBox>,
    pub confidence_score: f32,
}

/// Types de sections Business identifiées
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BusinessSection {
    ExecutiveSummary,
    FinancialHighlights,
    BusinessOverview,
    RiskFactors,
    MarketAnalysis,
    Governance,
    Sustainability,
    Unknown,
}

/// KPI financier extrait avec valeur et contexte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialKPI {
    pub name: String,        // "Revenue", "EBITDA", "Net Income"
    pub value: f64,
    pub currency: String,
    pub period: String,      // "2023", "Q3 2023"
    pub growth_rate: Option<f32>,
    pub unit: String,        // "million", "billion", "%"
}

/// Bounding box pour localisation dans le PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub page: u32,
}

/// Extracteur de métadonnées Business
pub struct BusinessMetadataEnricher {
    pub kpi_extractor: FinancialKPIExtractor,
    pub section_classifier: BusinessSectionClassifier,
    pub company_extractor: CompanyExtractor,
}

/// Extracteur de KPIs financiers avec patterns avancés
pub struct FinancialKPIExtractor {
    pub value_patterns: HashMap<String, Regex>,
    pub currency_patterns: Regex,
    pub unit_patterns: Regex,
}

/// Classificateur de sections Business
pub struct BusinessSectionClassifier {
    pub section_patterns: HashMap<BusinessSection, Regex>,
}

/// Extracteur d'informations entreprise
pub struct CompanyExtractor {
    pub company_name_patterns: Regex,
    pub sector_patterns: HashMap<String, Regex>,
}

// === Patterns regex compilés ===

static KPI_VALUE_PATTERNS: Lazy<HashMap<String, Regex>> = Lazy::new(|| {
    let mut patterns = HashMap::new();
    
    // Patterns pour différents types de KPIs
    patterns.insert(
        "revenue".to_string(),
        Regex::new(r"(?i)(revenue[s]?|chiffre\s+d'affaires|ca)\s*(?:of|was|reached|increased|to|de|:|at|a\s+atteint)?\s*(?:to|à)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")
            .expect("Invalid revenue pattern")
    );
    
    patterns.insert(
        "ebitda".to_string(),
        Regex::new(r"(?i)(EBITDA[s]?|résultat\s+opérationnel)\s*(?:of|was|reached|de|:|at|était\s+de)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")
            .expect("Invalid EBITDA pattern")
    );
    
    patterns.insert(
        "net_income".to_string(),
        Regex::new(r"(?i)(net\s+income[s]?|résultat\s+net|bénéfice\s+net)\s*(?:of|was|reached|de|:|at|s'élève\s+à)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")
            .expect("Invalid net income pattern")
    );
    
    patterns.insert(
        "total_assets".to_string(),
        Regex::new(r"(?i)(total\s+assets?|actif\s+total|total\s+du\s+bilan)\s*(?:of|was|reached|de|:|at|a\s+augmenté\s+à)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")
            .expect("Invalid total assets pattern")
    );
    
    patterns.insert(
        "market_cap".to_string(),
        Regex::new(r"(?i)(market\s+cap(?:italization)?|capitalisation\s+boursière|valeur\s+de\s+marché)\s*(?:of|was|reached|de|:|at|atteint)?\s*(?:\$|€|USD|EUR)?\s*([0-9]+(?:[,.]\s*[0-9]{3})*(?:[,.]?[0-9]+)?)\s*(million[s]?|billion[s]?|milliard[s]?|M|B|Md)?")
            .expect("Invalid market cap pattern")
    );
    
    patterns
});

static CURRENCY_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\$|USD|EUR|€|GBP|£|JPY|¥)")
        .expect("Invalid currency pattern")
});

static UNIT_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(million|billion|thousand|M|B|K|%)")
        .expect("Invalid unit pattern")
});

static COMPANY_NAME_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([A-Z][a-zA-Z\s&]{1,30}(?:Inc\.|Corp\.|Corporation|Company|Ltd\.|Limited|SA|SAS|Group|Holdings))\b")
        .expect("Invalid company name pattern")
});

impl BusinessMetadataEnricher {
    pub fn new() -> Self {
        Self {
            kpi_extractor: FinancialKPIExtractor::new(),
            section_classifier: BusinessSectionClassifier::new(),
            company_extractor: CompanyExtractor::new(),
        }
    }

    /// Enrichissement complet des métadonnées Business
    pub fn enrich_business_content(
        &self,
        content: &str,
        fiscal_year: Option<i32>,
        page_number: Option<u32>,
    ) -> Result<BusinessMetadata> {
        // Extraction des KPIs financiers
        let financial_kpis = self.kpi_extractor.extract_kpis(content, fiscal_year)?;
        
        // Classification de section
        let section_type = self.section_classifier.classify_section(content)?;
        
        // Extraction informations entreprise
        let company_name = self.company_extractor.extract_company_name(content);
        let sector = self.company_extractor.extract_sector(content);
        
        // Calcul score de confiance basé sur la richesse des métadonnées
        let confidence_score = self.calculate_metadata_confidence(&financial_kpis, &section_type, &company_name);

        Ok(BusinessMetadata {
            fiscal_year,
            company_name,
            sector,
            financial_kpis,
            section_type,
            page_number,
            bbox: None, // À implémenter avec détection layout
            confidence_score,
        })
    }

    /// Calcul du score de confiance des métadonnées
    fn calculate_metadata_confidence(
        &self,
        kpis: &[FinancialKPI],
        section_type: &BusinessSection,
        company_name: &Option<String>,
    ) -> f32 {
        let mut score = 0.0;

        // Score basé sur le nombre de KPIs extraits
        score += (kpis.len() as f32 * 0.2).min(0.8);

        // Score basé sur la précision de classification section
        match section_type {
            BusinessSection::ExecutiveSummary => score += 0.3,
            BusinessSection::FinancialHighlights => score += 0.3,
            BusinessSection::BusinessOverview => score += 0.2,
            BusinessSection::Unknown => score += 0.0,
            _ => score += 0.1,
        }

        // Score basé sur la détection d'entreprise
        if company_name.is_some() {
            score += 0.2;
        }

        score.min(1.0)
    }
}

impl FinancialKPIExtractor {
    pub fn new() -> Self {
        Self {
            value_patterns: KPI_VALUE_PATTERNS.clone(),
            currency_patterns: CURRENCY_PATTERNS.clone(),
            unit_patterns: UNIT_PATTERNS.clone(),
        }
    }

    /// Extraction de tous les KPIs financiers du contenu
    pub fn extract_kpis(&self, content: &str, fiscal_year: Option<i32>) -> Result<Vec<FinancialKPI>> {
        let mut kpis = Vec::new();

        for (kpi_name, pattern) in &self.value_patterns {
            if let Some(captures) = pattern.captures(content) {
                if let Some(value_str) = captures.get(2) { // Groupe 2 car groupe 1 = nom KPI maintenant
                    // Utiliser le parsing robuste pour nombres EU/US
                    if let Some(value) = self.parse_financial_number(value_str.as_str()) {
                        let unit = captures.get(3)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "".to_string());

                        // Normalisation des unités (million, billion)
                        let normalized_value = self.normalize_value(value, &unit);

                        // Détection de devise
                        let currency = self.extract_currency(content).unwrap_or_else(|| "USD".to_string());

                        // Période (fiscal year ou détection dans le texte)
                        let period = fiscal_year
                            .map(|y| y.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        kpis.push(FinancialKPI {
                            name: self.format_kpi_name(kpi_name),
                            value: normalized_value,
                            currency,
                            period,
                            growth_rate: None, // À implémenter
                            unit: self.normalize_unit(&unit),
                        });
                    }
                }
            }
        }

        Ok(kpis)
    }

    /// Normalisation des valeurs avec unités (million, billion)
    fn normalize_value(&self, value: f64, unit: &str) -> f64 {
        match unit.to_lowercase().as_str() {
            "million" | "millions" | "m" => value * 1_000_000.0,
            "billion" | "billions" | "milliard" | "milliards" | "b" | "md" => value * 1_000_000_000.0,
            "thousand" | "k" => value * 1_000.0,
            _ => value,
        }
    }

    /// Parse les nombres FR/EN avec virgules et points
    pub fn parse_financial_number(&self, number_str: &str) -> Option<f64> {
        let cleaned = number_str
            .replace(" ", "")  // Supprimer espaces
            .replace("\u{00A0}", ""); // Supprimer espaces insécables
            
        // Détecter format EU (1.234.567,89) vs US (1,234,567.89)
        if let Some(last_comma) = cleaned.rfind(',') {
            let after_comma = &cleaned[last_comma + 1..];
            if after_comma.len() <= 2 && after_comma.chars().all(|c| c.is_ascii_digit()) {
                // Format EU: comma = decimal separator
                let cleaned_eu = cleaned.replace(".", "").replace(",", ".");
                return cleaned_eu.parse::<f64>().ok();
            }
        }
        
        if let Some(last_dot) = cleaned.rfind('.') {
            let after_dot = &cleaned[last_dot + 1..];
            if after_dot.len() <= 2 && after_dot.chars().all(|c| c.is_ascii_digit()) {
                // Format US: dot = decimal separator
                let cleaned_us = cleaned.replace(",", "");
                return cleaned_us.parse::<f64>().ok();
            }
        }
        
        // Fallback: essayer de parser directement
        cleaned.replace(",", "").replace(".", "").parse::<f64>().ok()
    }

    /// Extraction de devise du contexte
    pub fn extract_currency(&self, content: &str) -> Option<String> {
        self.currency_patterns
            .find(content)
            .map(|m| match m.as_str() {
                "$" => "USD".to_string(),
                "€" => "EUR".to_string(),
                "£" => "GBP".to_string(),
                "¥" => "JPY".to_string(),
                other => other.to_string(),
            })
    }

    /// Formatage du nom de KPI
    fn format_kpi_name(&self, kpi_name: &str) -> String {
        match kpi_name {
            "revenue" => "Revenue".to_string(),
            "ebitda" => "EBITDA".to_string(),
            "net_income" => "Net Income".to_string(),
            "total_assets" => "Total Assets".to_string(),
            "market_cap" => "Market Capitalization".to_string(),
            _ => kpi_name.to_string(),
        }
    }

    /// Normalisation des unités
    fn normalize_unit(&self, unit: &str) -> String {
        match unit.to_lowercase().as_str() {
            "million" | "m" => "Million".to_string(),
            "billion" | "b" => "Billion".to_string(),
            "thousand" | "k" => "Thousand".to_string(),
            "%" => "Percent".to_string(),
            _ => "Units".to_string(),
        }
    }
}

impl BusinessSectionClassifier {
    pub fn new() -> Self {
        let mut section_patterns = HashMap::new();

        section_patterns.insert(
            BusinessSection::ExecutiveSummary,
            Regex::new(r"(?i)(executive\s+summary|management\s+summary|résumé\s+exécutif|synthèse\s+direction|summary\s+of\s+operations|aperçu\s+général)")
                .expect("Invalid executive summary pattern")
        );

        section_patterns.insert(
            BusinessSection::FinancialHighlights,
            Regex::new(r"(?i)(financial\s+highlights|financial\s+performance|key\s+financial|faits\s+saillants\s+financiers|performance\s+financière|résultats\s+financiers|principales\s+données\s+financières|chiffres\s+clés)")
                .expect("Invalid financial highlights pattern")
        );

        section_patterns.insert(
            BusinessSection::BusinessOverview,
            Regex::new(r"(?i)(business\s+overview|company\s+overview|business\s+description|aperçu\s+des\s+activités|présentation\s+du\s+groupe|description\s+de\s+l'entreprise|activités\s+du\s+groupe|notre\s+entreprise)")
                .expect("Invalid business overview pattern")
        );

        section_patterns.insert(
            BusinessSection::RiskFactors,
            Regex::new(r"(?i)(risk\s+factors|risks\s+and|business\s+risks|facteurs\s+de\s+risque|risques\s+et\s+incertitudes|principaux\s+risques|gestion\s+des\s+risques)")
                .expect("Invalid risk factors pattern")
        );

        section_patterns.insert(
            BusinessSection::MarketAnalysis,
            Regex::new(r"(?i)(market\s+analysis|industry\s+analysis|competitive\s+landscape|analyse\s+du\s+marché|étude\s+de\s+marché|environnement\s+concurrentiel|secteur\s+d'activité|positionnement\s+concurrentiel)")
                .expect("Invalid market analysis pattern")
        );

        Self { section_patterns }
    }

    /// Classification de section avec score de confiance
    pub fn classify_section(&self, content: &str) -> Result<BusinessSection> {
        // Priorité par ordre d'apparition dans le document
        let priority_order = vec![
            BusinessSection::ExecutiveSummary,
            BusinessSection::FinancialHighlights,
            BusinessSection::BusinessOverview,
            BusinessSection::RiskFactors,
            BusinessSection::MarketAnalysis,
        ];

        for section_type in priority_order {
            if let Some(pattern) = self.section_patterns.get(&section_type) {
                if pattern.is_match(content) {
                    return Ok(section_type);
                }
            }
        }

        Ok(BusinessSection::Unknown)
    }
}

impl CompanyExtractor {
    pub fn new() -> Self {
        let mut sector_patterns = HashMap::new();

        sector_patterns.insert(
            "Technology".to_string(),
            Regex::new(r"(?i)(technology|software|digital|tech|IT|cloud|AI)")
                .expect("Invalid tech sector pattern")
        );

        sector_patterns.insert(
            "Financial".to_string(),
            Regex::new(r"(?i)(financial|banking|insurance|investment|finance)")
                .expect("Invalid financial sector pattern")
        );

        sector_patterns.insert(
            "Healthcare".to_string(),
            Regex::new(r"(?i)(healthcare|pharmaceutical|medical|biotech|health)")
                .expect("Invalid healthcare sector pattern")
        );

        Self {
            company_name_patterns: COMPANY_NAME_PATTERNS.clone(),
            sector_patterns,
        }
    }

    /// Extraction du nom d'entreprise
    pub fn extract_company_name(&self, content: &str) -> Option<String> {
        self.company_name_patterns
            .find(content)
            .map(|m| m.as_str().trim().to_string())
    }

    /// Extraction du secteur d'activité
    pub fn extract_sector(&self, content: &str) -> Option<String> {
        for (sector_name, pattern) in &self.sector_patterns {
            if pattern.is_match(content) {
                return Some(sector_name.clone());
            }
        }
        None
    }
}

// === Tests unitaires ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_metadata_enrichment() {
        let enricher = BusinessMetadataEnricher::new();
        let content = "
            Executive Summary
            
            Microsoft Corporation achieved revenue of $2.1 billion in FY 2023.
            EBITDA reached $450 million, demonstrating strong financial performance.
        ";

        let metadata = enricher.enrich_business_content(content, Some(2023), Some(1)).unwrap();

        assert_eq!(metadata.fiscal_year, Some(2023));
        assert!(matches!(metadata.section_type, BusinessSection::ExecutiveSummary));
        assert!(metadata.financial_kpis.len() >= 2);
        assert!(metadata.confidence_score > 0.5);
    }

    #[test]
    fn test_financial_kpi_extraction() {
        let extractor = FinancialKPIExtractor::new();
        let content = "Revenue increased to $2.1 billion and EBITDA of $450 million";

        let kpis = extractor.extract_kpis(content, Some(2023)).unwrap();
        assert!(kpis.len() >= 2);

        let revenue_kpi = kpis.iter().find(|k| k.name == "Revenue").unwrap();
        assert_eq!(revenue_kpi.value, 2_100_000_000.0);
        assert_eq!(revenue_kpi.currency, "USD");
    }

    #[test]
    fn test_section_classification() {
        let classifier = BusinessSectionClassifier::new();

        let exec_summary = "Executive Summary: This year we achieved strong results";
        let section = classifier.classify_section(exec_summary).unwrap();
        assert!(matches!(section, BusinessSection::ExecutiveSummary));

        let financial = "Financial Highlights show record performance";
        let section = classifier.classify_section(financial).unwrap();
        assert!(matches!(section, BusinessSection::FinancialHighlights));
    }

    #[test]
    fn test_company_extraction() {
        let extractor = CompanyExtractor::new();

        let content = "Microsoft Corporation is a leading technology company";
        let company = extractor.extract_company_name(content);
        assert_eq!(company, Some("Microsoft Corporation".to_string()));

        let sector = extractor.extract_sector(content);
        assert_eq!(sector, Some("Technology".to_string()));
    }
}