// GRAVIS Search Optimizer - Phase 3 Enhancements
// Normalisation embeddings + Hybrid BM25 + Query routing

use crate::rag::DocumentCategory;

/// Normalisation L2 des embeddings pour améliorer la similarité cosinus
pub fn l2_normalize(embedding: &mut [f32]) {
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in embedding.iter_mut() {
            *value /= norm;
        }
    }
}

/// Calcul BM25 simplifié pour recherche textuelle
pub fn compute_bm25_score(query_terms: &[&str], document_text: &str) -> f32 {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;
    const AVG_DOC_LEN: f32 = 1000.0; // Longueur moyenne estimée
    
    let doc_terms: Vec<&str> = document_text.split_whitespace().collect();
    let doc_len = doc_terms.len() as f32;
    
    let mut score = 0.0;
    
    for &query_term in query_terms {
        let term_freq = doc_terms.iter().filter(|&&term| 
            term.to_lowercase().contains(&query_term.to_lowercase())
        ).count() as f32;
        
        if term_freq > 0.0 {
            let tf_component = (term_freq * (K1 + 1.0)) / 
                (term_freq + K1 * (1.0 - B + B * (doc_len / AVG_DOC_LEN)));
            
            // IDF simplifié (log fixe pour cette implémentation)
            let idf = 2.0; // Valeur fixe simplifiée
            
            score += tf_component * idf;
        }
    }
    
    score
}

/// Calcul de score hybride BM25 + cosine similarity
pub fn compute_hybrid_score(
    bm25_score: f32, 
    cosine_score: f32, 
    bm25_weight: f32
) -> f32 {
    let cosine_weight = 1.0 - bm25_weight;
    bm25_weight * bm25_score + cosine_weight * cosine_score
}

/// Types de requête détectés automatiquement
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    Business,
    Academic, 
    Legal,
    Technical,
    General,
}

/// Détection de l'intention de requête pour routing intelligent
pub fn detect_query_intent(query: &str) -> QueryIntent {
    let query_lower = query.to_lowercase();
    
    // Patterns Business
    let business_terms = [
        "revenue", "profit", "ebitda", "financial", "performance", 
        "earnings", "sales", "market", "strategy", "growth",
        "chiffre d'affaires", "bénéfice", "résultat", "croissance"
    ];
    
    // Patterns Academic  
    let academic_terms = [
        "research", "study", "analysis", "methodology", "experiment",
        "dataset", "algorithm", "model", "theory", "hypothesis",
        "recherche", "étude", "analyse", "expérience", "théorie"
    ];
    
    // Patterns Legal
    let legal_terms = [
        "legal", "law", "regulation", "compliance", "procedure",
        "contract", "agreement", "policy", "governance",
        "légal", "loi", "règlement", "procédure", "contrat"
    ];
    
    // Patterns Technical
    let technical_terms = [
        "technical", "engineering", "implementation", "system",
        "architecture", "design", "specification", "protocol",
        "technique", "ingénierie", "implémentation", "système"
    ];
    
    // Compter les matches par catégorie
    let business_matches = business_terms.iter()
        .filter(|&&term| query_lower.contains(term))
        .count();
        
    let academic_matches = academic_terms.iter()
        .filter(|&&term| query_lower.contains(term))
        .count();
        
    let legal_matches = legal_terms.iter()
        .filter(|&&term| query_lower.contains(term))
        .count();
        
    let technical_matches = technical_terms.iter()
        .filter(|&&term| query_lower.contains(term))
        .count();
    
    // Retourner l'intention avec le plus de matches
    let all_matches = [business_matches, academic_matches, legal_matches, technical_matches];
    let max_matches = *all_matches.iter().max().unwrap();
    
    if max_matches == 0 {
        return QueryIntent::General;
    }
    
    if business_matches == max_matches {
        QueryIntent::Business
    } else if academic_matches == max_matches {
        QueryIntent::Academic
    } else if legal_matches == max_matches {
        QueryIntent::Legal
    } else if technical_matches == max_matches {
        QueryIntent::Technical
    } else {
        QueryIntent::General
    }
}

/// Lexique légal strict pour boost conditionnel
const LEGAL_STRONG_PATTERNS: &[&str] = &[
    r"(?i)\barrêtés?\b",
    r"(?i)\bdécrets?\b", 
    r"(?i)\bprocédures?\b",
    r"(?i)\bjuridictions?\b",
    r"(?i)\bassignations?\b",
    r"(?i)\barticles?\s+L\.?\s*\d+",
    r"(?i)\bcode\s+de\s+procédure\b",
    r"(?i)\bstatutory\b",
    r"(?i)\bsubpoenas?\b",
    r"(?i)\blitigations?\b",
    r"(?i)\bcompliance\s+policy\b",
    r"(?i)\blegal\s+proceedings?\b",
    r"(?i)\brecours\b",
    r"(?i)\btribunaux?\b"
];

/// Comptage des hits lexique légal fort
pub fn count_strong_legal_hits(text: &str) -> usize {
    LEGAL_STRONG_PATTERNS.iter()
        .map(|pattern| {
            if let Ok(re) = regex::Regex::new(pattern) {
                re.find_iter(text).count()
            } else {
                0
            }
        })
        .sum()
}

/// Type-aware capping pour éviter mauvais classements cross-category
pub fn type_aware_cap(
    intent: &QueryIntent,
    doc_category: &DocumentCategory,
    score: f32
) -> f32 {
    match (intent, doc_category) {
        (QueryIntent::Legal, DocumentCategory::Business | DocumentCategory::Academic) => score.min(0.75),
        (QueryIntent::Business, DocumentCategory::Academic | DocumentCategory::Mixed) => score.min(0.85),
        (QueryIntent::Academic, DocumentCategory::Business | DocumentCategory::Mixed) => score.min(0.80),
        _ => score,
    }
}

/// BM25 pondéré par section (anti-disclaimer)
pub fn compute_weighted_bm25(query_terms: &[&str], document_text: &str) -> f32 {
    // Détection de sections basiques par patterns
    let sections = detect_document_sections(document_text);
    let mut weighted_score = 0.0;
    
    for (section_text, section_type) in sections {
        let section_bm25 = compute_bm25_score(query_terms, &section_text);
        
        let weight = section_weight(section_type);
        
        weighted_score += weight * section_bm25;
    }
    
    weighted_score
}

/// Pondération par section pour anti-bruit BM25 fielded
fn section_weight(section_type: SectionType) -> f32 {
    match section_type {
        SectionType::Title | SectionType::HeaderH1 => 1.0,  // Titre important mais pas survalorisé
        SectionType::Body => 0.7,                           // Corps de texte standard
        SectionType::TableCell => 0.6,                      // Cellules de tableau moins pertinentes
        SectionType::Disclaimer | SectionType::ForwardLooking => 0.25, // Anti-bruit maximal
    }
}

/// Types de sections pour pondération BM25
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants not used yet but planned for future BM25 fielded scoring
enum SectionType {
    Title,
    HeaderH1,
    Body,
    TableCell,
    Disclaimer,
    ForwardLooking,
}

/// Détection basique de sections dans le document
fn detect_document_sections(text: &str) -> Vec<(String, SectionType)> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        let section_type = classify_section_type(trimmed);
        
        sections.push((trimmed.to_string(), section_type));
    }
    
    if sections.is_empty() {
        sections.push((text.to_string(), SectionType::Body));
    }
    
    sections
}

/// Classification intelligente des sections pour anti-bruit BM25
fn classify_section_type(text: &str) -> SectionType {
    let lower_text = text.to_lowercase();
    
    // Détection disclaimer/forward-looking avec patterns étendus
    let disclaimer_patterns = [
        "disclaimer", "forward-looking", "avertissement", "mise en garde",
        "risk factors", "facteurs de risque", "legal notice", "mention légale",
        "governance", "gouvernance d'entreprise", "regulatory", "réglementaire",
        "safe harbor", "protection", "limitation of liability", "limitation de responsabilité"
    ];
    
    for pattern in disclaimer_patterns {
        if lower_text.contains(pattern) {
            return SectionType::Disclaimer;
        }
    }
    
    // Forward-looking statements spécifiques
    if lower_text.contains("forward") || lower_text.contains("outlook") || 
       lower_text.contains("projection") || lower_text.contains("estimate") {
        return SectionType::ForwardLooking;
    }
    
    // Titre (court et beaucoup de majuscules)
    if text.len() < 100 && (
        text.chars().filter(|c| c.is_uppercase()).count() as f32 / text.len() as f32 > 0.5
    ) {
        return SectionType::Title;
    }
    
    // Header H1 (patterns spécifiques)
    if lower_text.starts_with("chapter") || lower_text.starts_with("section") ||
       lower_text.starts_with("chapitre") || lower_text.starts_with("partie") {
        return SectionType::HeaderH1;
    }
    
    // Tableau (caractères de séparation)
    if text.contains('\t') || text.matches('|').count() > 2 || 
       text.matches("  ").count() > 5 { // Espaces multiples = colonnes
        return SectionType::TableCell;
    }
    
    // Par défaut: corps de texte
    SectionType::Body
}

/// Boost de score intelligent avec lexique légal et type-aware capping
pub fn apply_intelligent_boost(
    base_score: f32,
    query_intent: &QueryIntent,
    document_category: &DocumentCategory,
    document_text: &str
) -> f32 {
    let mut boosted_score = base_score;
    
    // Boost basique par alignement (plus agressif pour corriger la normalisation)
    let alignment_boost = match (query_intent, document_category) {
        (QueryIntent::Business, DocumentCategory::Business) => 0.25, // Boost plus fort
        (QueryIntent::Academic, DocumentCategory::Academic) => 0.20,
        (QueryIntent::Legal, DocumentCategory::Mixed) => 0.15,
        (QueryIntent::Technical, DocumentCategory::Academic) => 0.12,
        _ => 0.0,
    };
    
    boosted_score += alignment_boost;
    
    // Boost lexique légal conditionnel
    if *query_intent == QueryIntent::Legal {
        let strong_hits = count_strong_legal_hits(document_text);
        if strong_hits >= 2 {
            boosted_score += 0.15; // Boost fort si vraiment légal
        } else if !matches!(document_category, DocumentCategory::Mixed) {
            boosted_score -= 0.10; // Pénalité si pas de vraies mentions légales
        }
    }
    
    // Type-aware capping final + clamp to prevent negative scores
    let capped_score = type_aware_cap(query_intent, document_category, boosted_score);
    capped_score.max(0.0) // Jamais de score négatif
}

/// Structure pour résultat de recherche enrichi
#[derive(Debug, Clone)]
pub struct EnhancedSearchResult {
    pub document_id: String,
    pub content: String,
    pub category: DocumentCategory,
    pub cosine_score: f32,
    pub bm25_score: f32,
    pub hybrid_score: f32,
    pub final_score: f32, // Après boost intent
}

/// Moteur de recherche optimisé
pub struct SearchEngine {
    pub bm25_weight: f32,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            bm25_weight: 0.5, // Équilibrage 50/50 BM25/cosine par défaut
        }
    }
    
    pub fn search_with_optimization(
        &self,
        query: &str,
        query_embedding: &[f32],
        documents: &[(String, String, Vec<f32>, DocumentCategory)]
    ) -> Vec<EnhancedSearchResult> {
        let query_intent = detect_query_intent(query);
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        
        // Étape 1: Calcul des scores bruts
        let mut raw_results = Vec::new();
        let mut bm25_scores = Vec::new();
        let mut cosine_scores = Vec::new();
        
        for (doc_id, content, embedding, category) in documents {
            // Calcul BM25 pondéré par section (anti-disclaimer)
            let bm25_score = compute_weighted_bm25(&query_terms, content);
            
            // Calcul cosine (embeddings déjà normalisés)
            let cosine_score = cosine_similarity(query_embedding, embedding);
            
            bm25_scores.push(bm25_score);
            cosine_scores.push(cosine_score);
            
            raw_results.push((doc_id.clone(), content.clone(), embedding.clone(), category.clone()));
        }
        
        // Étape 2: Normalisation MinMax pour stabiliser les scores
        normalize_minmax(&mut bm25_scores);
        normalize_minmax(&mut cosine_scores);
        
        // Étape 3: Fusion hybride avec scores normalisés
        let mut results = Vec::new();
        
        for (i, (doc_id, content, _embedding, category)) in raw_results.iter().enumerate() {
            let bm25_norm = bm25_scores[i];
            let cosine_norm = cosine_scores[i];
            
            // Score hybride avec recette par intent (FIGÉE PROD - NE PAS MODIFIER)
            let hybrid_score = compute_intent_hybrid_score(&query_intent, cosine_norm, bm25_norm);
            
            // Boost intelligent après normalisation
            let final_score = apply_intelligent_boost(
                hybrid_score,
                &query_intent, 
                category,
                content
            );
            
            results.push(EnhancedSearchResult {
                document_id: doc_id.clone(),
                content: content.clone(),
                category: category.clone(),
                cosine_score: cosine_norm,  // Score normalisé
                bm25_score: bm25_norm,     // Score normalisé  
                hybrid_score,
                final_score,
            });
        }
        
        // Tri par score final décroissant
        results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        
        results
    }
}

/// Similarité cosinus pour embeddings normalisés
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    // Si embeddings normalisés, le produit scalaire = cosine similarity
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Normalisation MinMax pour stabiliser les scores hybrides
pub fn normalize_minmax(scores: &mut [f32]) {
    if scores.is_empty() {
        return;
    }
    
    let (min_val, max_val) = scores.iter().fold((f32::MAX, f32::MIN), |(min, max), &val| {
        (min.min(val), max.max(val))
    });
    
    let range = (max_val - min_val).max(1e-6); // Éviter division par zéro
    
    for score in scores.iter_mut() {
        *score = (*score - min_val) / range; // Normalize to [0,1]
    }
}

/// Normalisation Z-score + logistic pour robustesse aux outliers  
pub fn normalize_zscore_logistic(scores: &[f32]) -> Vec<f32> {
    if scores.is_empty() {
        return vec![];
    }
    
    // Calcul mean et std dev
    let mean = scores.iter().sum::<f32>() / scores.len() as f32;
    let variance = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / scores.len() as f32;
    let std_dev = variance.sqrt().max(1e-6); // Éviter division par zéro
    
    // Z-score puis logistic pour borner à (0,1)
    scores.iter().map(|&x| {
        let z_score = (x - mean) / std_dev;
        1.0 / (1.0 + (-z_score).exp()) // Logistic function
    }).collect()
}

/// Recette hybride par intent - FIGÉE PRODUCTION (validée E2E)
/// Ne pas modifier ces coefficients sans validation complète
pub fn compute_intent_hybrid_score(
    query_intent: &QueryIntent, 
    cosine_score: f32, 
    bm25_score: f32
) -> f32 {
    match query_intent {
        // Business: privilégie BM25 (terminologie métier)
        QueryIntent::Business => 0.4 * cosine_score + 0.6 * bm25_score,
        
        // Academic: équilibre cosine/BM25 (concepts + termes)
        QueryIntent::Academic => 0.55 * cosine_score + 0.45 * bm25_score,
        
        // Legal: privilégie BM25 fielded (termes juridiques précis)
        QueryIntent::Legal => 0.35 * cosine_score + 0.65 * bm25_score,
        
        // Technical: équilibre parfait
        QueryIntent::Technical => 0.5 * cosine_score + 0.5 * bm25_score,
        
        // General: équilibre par défaut
        QueryIntent::General => 0.5 * cosine_score + 0.5 * bm25_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_intent_detection() {
        assert_eq!(detect_query_intent("revenue financial performance"), QueryIntent::Business);
        assert_eq!(detect_query_intent("research methodology analysis"), QueryIntent::Academic);
        assert_eq!(detect_query_intent("legal administrative procedure"), QueryIntent::Legal);
        assert_eq!(detect_query_intent("hello world"), QueryIntent::General);
    }
    
    #[test]
    fn test_l2_normalization() {
        let mut vec = vec![3.0, 4.0, 0.0];
        l2_normalize(&mut vec);
        
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_intent_boost() {
        let base_score = 0.5;
        let boosted = apply_intent_boost(
            base_score,
            &QueryIntent::Business,
            &DocumentCategory::Business
        );
        
        assert!(boosted > base_score);
        assert_eq!(boosted, 0.65); // 0.5 + 0.15
    }
}