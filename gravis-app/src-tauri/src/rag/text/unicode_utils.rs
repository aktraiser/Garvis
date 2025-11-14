// GRAVIS Unicode Utils - Phase 3A
// Module de normalisation des ligatures Unicode pour PDFs

// use unicode_normalization::UnicodeNormalization; // Commented out - not currently used
use std::collections::HashMap;
use once_cell::sync::Lazy;
use anyhow::Result;
use dashmap::DashMap;

/// Table de correspondance des ligatures communes vers leur forme décomposée
static LIGATURE_MAPPING: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    
    // Ligatures latines communes (Unicode FB00-FB06)
    mapping.insert('\u{FB00}', "ff");  // ﬀ → ff
    mapping.insert('\u{FB01}', "fi");  // ﬁ → fi
    mapping.insert('\u{FB02}', "fl");  // ﬂ → fl
    mapping.insert('\u{FB03}', "ffi"); // ﬃ → ffi
    mapping.insert('\u{FB04}', "ffl"); // ﬄ → ffl
    mapping.insert('\u{FB05}', "ft");  // ﬅ → ft
    mapping.insert('\u{FB06}', "st");  // ﬆ → st
    
    // Ligatures étendues (rares mais présentes dans certains PDFs)
    mapping.insert('\u{0133}', "ij");  // ĳ → ij (Dutch)
    mapping.insert('\u{0152}', "OE");  // Œ → OE
    mapping.insert('\u{0153}', "oe");  // œ → oe
    mapping.insert('\u{1D6B}', "ue");  // ᵫ → ue
    
    // Ligatures typographiques additionnelles
    mapping.insert('\u{FB00}', "ff");  // Double verification ﬀ
    mapping.insert('\u{FB01}', "fi");  // Double verification ﬁ
    mapping.insert('\u{FB02}', "fl");  // Double verification ﬂ
    
    mapping
});

/// Statistiques de normalisation détaillées
#[derive(Debug, Clone, Default)]
pub struct NormalizationStats {
    pub lig_fi: usize,
    pub lig_fl: usize,
    pub lig_ffi: usize,
    pub lig_ffl: usize,
    pub lig_ff: usize,
    pub lig_other: usize,
    pub ligatures_replaced: usize,
    pub unicode_normalized: bool,
    pub nbsp_replaced: usize,
    pub total_chars_before: usize,
    pub total_chars_after: usize,
}

/// Normalisation complète du texte PDF avec ligatures (optimisée, sans spam logs)
pub fn sanitize_pdf_text(input: &str) -> Result<(String, NormalizationStats)> {
    use unicode_normalization::UnicodeNormalization;
    
    let mut stats = NormalizationStats {
        total_chars_before: input.chars().count(),
        ..Default::default()
    };
    
    // Étape 1: Remplacement optimisé des ligatures avec compteurs détaillés
    let mut result = String::with_capacity(input.len());
    
    for ch in input.chars() {
        match ch {
            'ﬁ' => { 
                result.push_str("fi"); 
                stats.lig_fi += 1; 
                stats.ligatures_replaced += 1;
            }
            'ﬂ' => { 
                result.push_str("fl"); 
                stats.lig_fl += 1; 
                stats.ligatures_replaced += 1;
            }
            'ﬃ' => { 
                result.push_str("ffi"); 
                stats.lig_ffi += 1; 
                stats.ligatures_replaced += 1;
            }
            'ﬄ' => { 
                result.push_str("ffl"); 
                stats.lig_ffl += 1; 
                stats.ligatures_replaced += 1;
            }
            'ﬀ' => { 
                result.push_str("ff"); 
                stats.lig_ff += 1; 
                stats.ligatures_replaced += 1;
            }
            '\u{00A0}' => { 
                result.push(' '); 
                stats.nbsp_replaced += 1;
            }
            _ => result.push(ch),
        }
    }
    
    // Étape 2: Normalisation Unicode NFKC
    let normalized: String = result.nfkc().collect();
    stats.unicode_normalized = true;
    stats.total_chars_after = normalized.chars().count();
    
    // Log unique et concis (remplace le spam)
    if stats.ligatures_replaced > 0 {
        tracing::debug!(
            fi = stats.lig_fi,
            fl = stats.lig_fl, 
            ffi = stats.lig_ffi,
            ffl = stats.lig_ffl,
            ff = stats.lig_ff,
            nbsp = stats.nbsp_replaced,
            total = stats.ligatures_replaced,
            "Unicode ligatures normalized"
        );
    }
    
    Ok((normalized, stats))
}

/// Point d'entrée unique pour déligature - remplace toutes les autres fonctions
/// Normalisation complète: ligatures + NFKC + espaces + zero-width
pub fn sanitize_pdf_text_all(input: &str) -> (String, NormalizationStats) {
    use unicode_normalization::UnicodeNormalization;
    let mut stats = NormalizationStats::default();
    let mut result = String::with_capacity(input.len());
    
    // Étape 1: Remplacement ligatures (toutes les variantes importantes)
    for ch in input.chars() {
        match ch {
            'ﬁ' => { result.push_str("fi"); stats.lig_fi += 1; }
            'ﬂ' => { result.push_str("fl"); stats.lig_fl += 1; }
            'ﬃ' => { result.push_str("ffi"); stats.lig_ffi += 1; }
            'ﬄ' => { result.push_str("ffl"); stats.lig_ffl += 1; }
            'ﬀ' => { result.push_str("ff"); stats.lig_ff += 1; }
            'ﬅ' => result.push_str("ft"),
            'ﬆ' => result.push_str("st"),
            '\u{00A0}' => { result.push(' '); stats.nbsp_replaced += 1; }
            _ => result.push(ch),
        }
    }
    
    // Log agrégé unique (plus de spam)
    let total_ligatures = stats.lig_fi + stats.lig_fl + stats.lig_ffi + stats.lig_ffl + stats.lig_ff;
    if total_ligatures > 0 {
        tracing::debug!(fi = stats.lig_fi, fl = stats.lig_fl, total = total_ligatures, "Unicode ligatures normalized");
    }
    stats.ligatures_replaced = total_ligatures;
    stats.total_chars_before = input.chars().count();
    
    // Étape 2: NFKC normalization
    result = result.nfkc().collect::<String>();
    stats.unicode_normalized = true;
    
    // Étape 3: Zero-width cleanup
    result = result.replace('\u{200B}', ""); // ZWSP
    result = result.replace('\u{200C}', ""); // ZWNJ
    result = result.replace('\u{200D}', ""); // ZWJ
    result = result.replace('\u{FEFF}', ""); // BOM
    
    // Étape 4: Espaces multiples
    result = result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
    stats.total_chars_after = result.chars().count();
    (result, stats)
}

/// Cache de normalisation pour éviter de retraiter les mêmes textes
static NORMALIZATION_CACHE: Lazy<DashMap<String, (String, NormalizationStats)>> = Lazy::new(|| {
    DashMap::new()
});

/// Point d'entrée avec cache pour éviter le retraitement des mêmes PDFs
pub fn sanitize_pdf_text_cached(input: &str) -> (String, NormalizationStats) {
    let input_hash = blake3::hash(input.as_bytes()).to_hex().to_string();
    
    if let Some(cached) = NORMALIZATION_CACHE.get(&input_hash) {
        tracing::trace!("Unicode normalization cache hit for hash {}", &input_hash[..8]);
        return cached.clone();
    }
    
    let result = sanitize_pdf_text_all(input);
    NORMALIZATION_CACHE.insert(input_hash, result.clone());
    result
}

/// Remplacement ciblé des ligatures problématiques
#[allow(dead_code)] // Keep for potential future use in optimization
fn replace_ligatures(input: &str, stats: &mut NormalizationStats) -> String {
    let mut result = String::with_capacity(input.len() * 2); // Buffer plus large
    
    for ch in input.chars() {
        if let Some(&replacement) = LIGATURE_MAPPING.get(&ch) {
            result.push_str(replacement);
            stats.ligatures_replaced += 1;
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Détection et rapport des ligatures présentes dans le texte
pub fn detect_ligatures(input: &str) -> Vec<(char, usize, String)> {
    let mut ligature_detections = Vec::new();
    
    for (position, ch) in input.char_indices() {
        if let Some(&replacement) = LIGATURE_MAPPING.get(&ch) {
            ligature_detections.push((ch, position, replacement.to_string()));
        }
    }
    
    ligature_detections
}

/// Validation de la qualité de normalisation
pub fn validate_normalization_quality(original: &str, normalized: &str) -> f32 {
    let original_len = original.chars().count() as f32;
    let normalized_len = normalized.chars().count() as f32;
    
    // Score basé sur l'expansion raisonnable du texte
    // Les ligatures causent généralement 10-20% d'expansion
    let expansion_ratio = normalized_len / original_len;
    
    match expansion_ratio {
        ratio if ratio >= 1.0 && ratio <= 1.3 => 1.0,      // Excellent
        ratio if ratio > 1.3 && ratio <= 1.5 => 0.8,       // Bon
        ratio if ratio > 1.5 && ratio <= 2.0 => 0.6,       // Acceptable
        _ => 0.3,                                            // Problématique
    }
}

/// Nettoyage avancé pour texte extrait d'OCR/PDF
pub fn clean_extracted_text(input: &str) -> Result<String> {
    let (normalized, _stats) = sanitize_pdf_text(input)?;
    
    // Post-processing additionnel pour OCR
    let cleaned = normalized
        // Suppression espaces multiples
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        // Normalisation des guillemets
        .replace('"', "\"")
        .replace('"', "\"")
        .replace('\u{2018}', "'")  // '
        .replace('\u{2019}', "'")  // '
        // Normalisation des tirets
        .replace('–', "-")
        .replace('—', "-")
        // Suppression caractères de contrôle
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect();
    
    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ligature_replacement() {
        let input = "The ﬁrst ﬂoor oﬃce has ﬀ and ﬃ ligatures";
        let (result, stats) = sanitize_pdf_text(input).unwrap();
        
        assert_eq!(result, "The first floor office has ff and ffi ligatures");
        assert_eq!(stats.ligatures_replaced, 5);
        assert!(stats.total_chars_before > 0);
    }

    #[test]
    fn test_unicode_normalization() {
        let input = "café naïve résumé"; // Avec accents combinés
        let (result, stats) = sanitize_pdf_text(input).unwrap();
        
        // Le texte devrait rester identique car déjà normalisé
        assert_eq!(result, input);
        assert_eq!(stats.ligatures_replaced, 0);
    }

    #[test]
    fn test_ligature_detection() {
        let input = "ﬁle.pdf and ﬂux.txt";
        let detections = detect_ligatures(input);
        
        assert_eq!(detections.len(), 2);
        assert_eq!(detections[0], ('\u{FB01}', 0, "fi".to_string()));
        assert_eq!(detections[1], ('\u{FB02}', 13, "fl".to_string()));
    }

    #[test]
    fn test_normalization_quality() {
        let original = "ﬁﬂe";
        let normalized = "file";
        let quality = validate_normalization_quality(original, normalized);
        
        assert!(quality >= 0.8); // Should be high quality
    }

    #[test]
    fn test_clean_extracted_text() {
        let messy_input = "  The   ﬁrst  ﬂoor   has  \"strange\"  characters–—  ";
        let cleaned = clean_extracted_text(messy_input).unwrap();
        
        assert_eq!(cleaned, "The first floor has \"strange\" characters--");
    }

    #[test]
    fn test_empty_and_edge_cases() {
        // Test chaîne vide
        let (result, stats) = sanitize_pdf_text("").unwrap();
        assert_eq!(result, "");
        assert_eq!(stats.ligatures_replaced, 0);

        // Test sans ligatures
        let (result, stats) = sanitize_pdf_text("Normal text").unwrap();
        assert_eq!(result, "Normal text");
        assert_eq!(stats.ligatures_replaced, 0);

        // Test avec ligatures seulement
        let (result, stats) = sanitize_pdf_text("ﬁﬂﬃﬄ").unwrap();
        assert_eq!(result, "fiflffiffl");
        assert_eq!(stats.ligatures_replaced, 4);
    }

    #[test]
    fn test_french_ligatures() {
        let input = "Œuvre complète avec ﬁnitions";
        let (result, _stats) = sanitize_pdf_text(input).unwrap();
        
        assert_eq!(result, "OEuvre complète avec finitions");
    }
}