// GRAVIS Document Classifier - Phase 3A: Business Documents Detection
// Module de classification automatique pour Universal RAG Pipeline

use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::Result;
use once_cell::sync::Lazy;

/// Catégories sémantiques de documents supportées par Universal RAG
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DocumentCategory {
    Academic,
    Business,
    Legal,
    Technical,
    Mixed,
}

/// Signaux spécifiques aux documents Business
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessSignals {
    pub executive_summary: bool,
    pub financial_metrics: Vec<String>,
    pub company_identifiers: Vec<String>,
    pub fiscal_year: Option<i32>,
    pub confidence_score: f32,
}

/// Classificateur principal pour détection automatique de type
pub struct DocumentClassifier {
    pub business_patterns: BusinessPatternMatcher,
    pub academic_patterns: AcademicPatternMatcher,
    pub legal_patterns: LegalPatternMatcher,
    pub technical_patterns: TechnicalPatternMatcher,
}

/// Patterns pour détection Business (Fortune 500 + CAC40 optimisé)
pub struct BusinessPatternMatcher {
    section_patterns: Regex,
    financial_patterns: Regex,
    company_patterns: Regex,
    fiscal_patterns: Regex,
}

/// Patterns académiques (Phase 2 existant)
pub struct AcademicPatternMatcher {
    citation_patterns: Regex,
    section_patterns: Regex,
}

/// Patterns légaux (Phase 3B)
pub struct LegalPatternMatcher {
    clause_patterns: Regex,
    legal_terms: Regex,
}

/// Patterns techniques (Phase 3C)
pub struct TechnicalPatternMatcher {
    code_patterns: Regex,
    spec_patterns: Regex,
}

// === Patterns regex compilés (static pour performance) ===

static BUSINESS_SECTION_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(Executive Summary|Financial Performance|Business Overview|Risk Factors|Management Discussion|Market Analysis|Annual Report|Quarterly Report|Financial Highlights|Shareholder|Revenue|EBITDA|Balance Sheet|Income Statement|Cash Flow|Résumé Exécutif|Performance Financière|Aperçu des Activités|Facteurs de Risque|Discussion de la Direction|Analyse du Marché|Rapport Annuel|Rapport Trimestriel|Faits Saillants Financiers|Actionnaire|Chiffre d'Affaires|Bilan|Compte de Résultat|Flux de Trésorerie)")
        .expect("Invalid business section regex")
});

static BUSINESS_FINANCIAL_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(Revenue|EBITDA|Net Income|Gross Profit|Operating Income|Total Assets|Shareholders.{0,10}Equity|Return on|ROI|ROE|Dividend|Earnings per Share|EPS|Market Cap|P/E Ratio|Debt to Equity|Chiffre d'Affaires|Résultat Net|Bénéfice Net|Résultat Opérationnel|Actif Total|Capitaux Propres|Rendement|Dividende|Capitalisation Boursière)")
        .expect("Invalid financial metrics regex")
});

static BUSINESS_COMPANY_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(Inc\.|Ltd\.|Corp\.|Corporation|Company|SA|SAS|SARL|Group|Holdings|Enterprises|Solutions|Technologies|CEO|CFO|Board of Directors|Fiscal Year|FY\s*\d{4})")
        .expect("Invalid company identifier regex")
});

static BUSINESS_FISCAL_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(FY\s*(\d{4})|Fiscal Year\s*(\d{4})|Year Ended\s*\w+\s*\d{1,2},?\s*(\d{4})|For the year ended|Annual Report\s*(\d{4}))")
        .expect("Invalid fiscal year regex")
});

static ACADEMIC_CITATION_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\[(\d+|\w+)\]|\(\w+\s+et\s+al\.?,?\s+\d{4}\)|doi:|arXiv:|References|Bibliography)")
        .expect("Invalid academic citation regex")
});

impl DocumentClassifier {
    pub fn new() -> Self {
        Self {
            business_patterns: BusinessPatternMatcher::new(),
            academic_patterns: AcademicPatternMatcher::new(),
            legal_patterns: LegalPatternMatcher::new(),
            technical_patterns: TechnicalPatternMatcher::new(),
        }
    }

    /// Classification principale avec scoring pondéré
    pub fn classify(&self, content: &str) -> Result<DocumentCategory> {
        let business_score = self.detect_business_confidence(content)?;
        let academic_score = self.detect_academic_confidence(content)?;
        let legal_score = self.detect_legal_confidence(content)?;
        let technical_score = self.detect_technical_confidence(content)?;

        // Seuils de classification (calibrés sur dataset test)
        let business_threshold = 0.6;
        let academic_threshold = 0.7;
        let legal_threshold = 0.5;
        let technical_threshold = 0.5;

        // Classification par score maximum avec seuils
        if business_score >= business_threshold && business_score >= academic_score {
            Ok(DocumentCategory::Business)
        } else if academic_score >= academic_threshold && academic_score >= business_score {
            Ok(DocumentCategory::Academic)
        } else if legal_score >= legal_threshold {
            Ok(DocumentCategory::Legal)
        } else if technical_score >= technical_threshold {
            Ok(DocumentCategory::Technical)
        } else {
            Ok(DocumentCategory::Mixed)
        }
    }

    /// Détection Business avec signaux enrichis (Phase 3A focus)
    pub fn detect_business_confidence(&self, content: &str) -> Result<f32> {
        let signals = self.extract_business_signals(content)?;
        Ok(signals.confidence_score)
    }

    /// Extraction complète des signaux Business
    pub fn extract_business_signals(&self, content: &str) -> Result<BusinessSignals> {
        // Utilisation des matchers pour éliminer warnings dead_code
        let section_matches = self.business_patterns.detect_sections(content);
        let financial_matches = self.business_patterns.detect_kpis(content);
        let company_matches = self.business_patterns.detect_companies(content);

        // Détection Executive Summary
        let executive_summary = content.to_lowercase().contains("executive summary") ||
                               content.to_lowercase().contains("management summary");

        // Extraction année fiscale via matcher
        let fiscal_year = self.business_patterns.extract_fiscal_year(content);

        // Calcul score de confiance pondéré
        let confidence_score = self.calculate_business_confidence_score(
            section_matches.len(),
            &financial_matches,
            &company_matches,
            executive_summary,
            fiscal_year.is_some(),
        );

        Ok(BusinessSignals {
            executive_summary,
            financial_metrics: financial_matches,
            company_identifiers: company_matches,
            fiscal_year,
            confidence_score,
        })
    }

    /// Algorithme de scoring Business (selon feuille de route)
    pub fn calculate_business_confidence_score(
        &self,
        section_matches: usize,
        financial_metrics: &[String],
        company_identifiers: &[String],
        has_executive_summary: bool,
        has_fiscal_year: bool,
    ) -> f32 {
        let mut score = 0.0;

        // Pondération selon feuille de route
        if has_executive_summary { score += 0.3; }
        if section_matches > 0 { score += (section_matches as f32 * 0.1).min(0.4); }
        if !financial_metrics.is_empty() { score += (financial_metrics.len() as f32 * 0.05).min(0.4); }
        if !company_identifiers.is_empty() { score += (company_identifiers.len() as f32 * 0.03).min(0.2); }
        if has_fiscal_year { score += 0.2; }

        // Normalisation 0.0-1.0
        score.min(1.0)
    }


    /// Classification académique (héritée Phase 2)
    pub fn detect_academic_confidence(&self, content: &str) -> Result<f32> {
        // Utilisation des matchers académiques
        let citation_matches = self.academic_patterns.detect_citations(content);
        let section_matches = self.academic_patterns.detect_sections(content);

        let score = (citation_matches.len() as f32 * 0.2 + section_matches.len() as f32 * 0.15).min(1.0);
        Ok(score)
    }

    /// Classification légale (Phase 3B - implémentation de base)
    pub fn detect_legal_confidence(&self, content: &str) -> Result<f32> {
        // Utilisation des matchers légaux
        let clause_matches = self.legal_patterns.detect_clauses(content);
        let term_matches = self.legal_patterns.detect_legal_terms(content);

        let score = ((clause_matches.len() + term_matches.len()) as f32 * 0.1).min(1.0);
        Ok(score)
    }

    /// Classification technique (Phase 3C - implémentation de base)
    pub fn detect_technical_confidence(&self, content: &str) -> Result<f32> {
        // Utilisation des matchers techniques
        let code_matches = self.technical_patterns.detect_code(content);
        let spec_matches = self.technical_patterns.detect_specs(content);

        let score = ((code_matches.len() + spec_matches.len()) as f32 * 0.1).min(1.0);
        Ok(score)
    }
}

// === Implémentations des pattern matchers ===

impl BusinessPatternMatcher {
    pub fn new() -> Self {
        Self {
            section_patterns: BUSINESS_SECTION_PATTERNS.clone(),
            financial_patterns: BUSINESS_FINANCIAL_PATTERNS.clone(),
            company_patterns: BUSINESS_COMPANY_PATTERNS.clone(),
            fiscal_patterns: BUSINESS_FISCAL_PATTERNS.clone(),
        }
    }

    /// Détection des sections Business
    pub fn detect_sections(&self, content: &str) -> Vec<String> {
        self.section_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Détection des KPIs financiers
    pub fn detect_kpis(&self, content: &str) -> Vec<String> {
        self.financial_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Détection des identifiants d'entreprise
    pub fn detect_companies(&self, content: &str) -> Vec<String> {
        self.company_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Extraction de l'année fiscale
    pub fn extract_fiscal_year(&self, content: &str) -> Option<i32> {
        if let Some(captures) = self.fiscal_patterns.captures(content) {
            for i in 1..=5 {
                if let Some(year_match) = captures.get(i) {
                    if let Ok(year) = year_match.as_str().parse::<i32>() {
                        if year >= 2000 && year <= 2030 {
                            return Some(year);
                        }
                    }
                }
            }
        }
        None
    }
}

impl AcademicPatternMatcher {
    pub fn new() -> Self {
        Self {
            citation_patterns: ACADEMIC_CITATION_PATTERNS.clone(),
            section_patterns: Regex::new(r"(?i)(Abstract|Introduction|Methodology|Results|Discussion|Conclusion|References)")
                .expect("Invalid academic section regex"),
        }
    }

    /// Détection des citations académiques
    pub fn detect_citations(&self, content: &str) -> Vec<String> {
        self.citation_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Détection des sections académiques
    pub fn detect_sections(&self, content: &str) -> Vec<String> {
        self.section_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

impl LegalPatternMatcher {
    pub fn new() -> Self {
        Self {
            clause_patterns: Regex::new(r"(?i)(Article|Clause|Section|Whereas|Therefore)")
                .expect("Invalid legal clause regex"),
            legal_terms: Regex::new(r"(?i)(Party|Obligation|Liability|Termination|Contract)")
                .expect("Invalid legal terms regex"),
        }
    }

    /// Détection des clauses légales
    pub fn detect_clauses(&self, content: &str) -> Vec<String> {
        self.clause_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Détection des termes légaux
    pub fn detect_legal_terms(&self, content: &str) -> Vec<String> {
        self.legal_terms
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

impl TechnicalPatternMatcher {
    pub fn new() -> Self {
        Self {
            code_patterns: Regex::new(r"(```|`\w+`|def\s+\w+|class\s+\w+|function\s+\w+)")
                .expect("Invalid code pattern regex"),
            spec_patterns: Regex::new(r"(?i)(Specification|Implementation|Algorithm|API|Interface)")
                .expect("Invalid spec pattern regex"),
        }
    }

    /// Détection des fragments de code
    pub fn detect_code(&self, content: &str) -> Vec<String> {
        self.code_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Détection des termes de spécification technique
    pub fn detect_specs(&self, content: &str) -> Vec<String> {
        self.spec_patterns
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

// === Tests unitaires ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_classification() {
        let classifier = DocumentClassifier::new();
        let business_content = "
            Executive Summary
            
            Our company achieved strong financial performance in FY 2023.
            Revenue increased to $2.1 billion, with EBITDA of $450 million.
            Total Assets reached $3.2 billion.
            
            Management Discussion
            The Board of Directors approved the annual dividend.
        ";

        let doc_type = classifier.classify(business_content).unwrap();
        assert_eq!(doc_type, DocumentCategory::Business);

        let signals = classifier.extract_business_signals(business_content).unwrap();
        assert!(signals.executive_summary);
        assert!(signals.confidence_score > 0.6);
        assert_eq!(signals.fiscal_year, Some(2023));
        assert!(!signals.financial_metrics.is_empty());
    }

    #[test]
    fn test_academic_classification() {
        let classifier = DocumentClassifier::new();
        let academic_content = "
            Abstract
            
            This study presents a novel approach to machine learning.
            Previous work by Smith et al. (2020) showed limitations.
            Our methodology improves upon [15] by 15%.
            
            References
            [1] Smith, J. et al. (2020). Machine Learning Advances.
        ";

        let doc_type = classifier.classify(academic_content).unwrap();
        assert_eq!(doc_type, DocumentCategory::Academic);
    }

    #[test]
    fn test_mixed_classification() {
        let classifier = DocumentClassifier::new();
        let mixed_content = "
            This document contains various information.
            Some technical details but no clear pattern.
        ";

        let doc_type = classifier.classify(mixed_content).unwrap();
        assert_eq!(doc_type, DocumentCategory::Mixed);
    }

    #[test]
    fn test_fiscal_year_extraction() {
        let classifier = DocumentClassifier::new();
        
        let test_cases = vec![
            ("FY 2023", Some(2023)),
            ("Fiscal Year 2022", Some(2022)),
            ("Year Ended December 31, 2021", Some(2021)),
            ("Annual Report 2024", Some(2024)),
            ("No year here", None),
        ];

        for (content, expected) in test_cases {
            let result = classifier.business_patterns.extract_fiscal_year(content);
            assert_eq!(result, expected, "Failed for content: {}", content);
        }
    }

    #[test]
    fn test_business_confidence_scoring() {
        let classifier = DocumentClassifier::new();
        
        // Test score élevé
        let high_score = classifier.calculate_business_confidence_score(
            5,  // section_matches
            &vec!["Revenue".to_string(), "EBITDA".to_string()], // financial_metrics
            &vec!["Corp.".to_string()], // company_identifiers
            true, // has_executive_summary
            true, // has_fiscal_year
        );
        assert!(high_score > 0.8);

        // Test score faible
        let low_score = classifier.calculate_business_confidence_score(
            0, &vec![], &vec![], false, false
        );
        assert!(low_score < 0.1);
    }
}