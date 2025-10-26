// Test PDF avec normalisation Unicode activée
// Teste la nouvelle fonctionnalité de normalisation des ligatures

use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use gravis_app_lib::rag::ocr::{
    SimplePdfExtractor, PdfExtractConfig, 
    normalize_for_rag, needs_normalization
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Test PDF avec Normalisation Unicode");

    // Chemin vers le PDF DeepSeek-OCR
    let pdf_path = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/2510.18234v1.pdf");
    
    if !pdf_path.exists() {
        return Err("PDF file not found".into());
    }

    info!("📄 PDF trouvé: {:?}", pdf_path);

    // Créer répertoire de sortie
    let output_dir = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/pdf_normalized_results");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).await?;
    }

    // Test 1: Extraction SANS normalisation
    info!("📖 Test 1: Extraction SANS normalisation Unicode");
    let config_raw = PdfExtractConfig {
        min_tokens: 10,
        timeout: std::time::Duration::from_secs(30),
        normalize_unicode: false,  // Désactivée
    };
    
    let extractor_raw = SimplePdfExtractor::new(config_raw);
    let result_raw = extractor_raw.extract_pdf_text(&pdf_path).await?;
    
    info!("✅ Extraction brute réussie:");
    info!("   📝 {} caractères", result_raw.text.len());
    info!("   🔤 {} mots", result_raw.text.split_whitespace().count());
    
    // Sauvegarder le texte brut
    let raw_file = output_dir.join("raw_extract_result.txt");
    fs::write(&raw_file, &result_raw.text).await?;
    info!("   💾 Sauvegardé: {:?}", raw_file);

    // Test 2: Extraction AVEC normalisation
    info!("📖 Test 2: Extraction AVEC normalisation Unicode");
    let config_normalized = PdfExtractConfig {
        min_tokens: 10,
        timeout: std::time::Duration::from_secs(30),
        normalize_unicode: true,  // Activée
    };
    
    let extractor_normalized = SimplePdfExtractor::new(config_normalized);
    let result_normalized = extractor_normalized.extract_pdf_text(&pdf_path).await?;
    
    info!("✅ Extraction normalisée réussie:");
    info!("   📝 {} caractères", result_normalized.text.len());
    info!("   🔤 {} mots", result_normalized.text.split_whitespace().count());
    
    // Sauvegarder le texte normalisé
    let normalized_file = output_dir.join("normalized_extract_result.txt");
    fs::write(&normalized_file, &result_normalized.text).await?;
    info!("   💾 Sauvegardé: {:?}", normalized_file);

    // Test 3: Comparaison et détection de problèmes
    info!("📊 Test 3: Analyse comparative");
    
    let needs_norm = needs_normalization(&result_raw.text);
    info!("🔍 Détection automatique: normalisation {} nécessaire", 
          if needs_norm { "EST" } else { "N'EST PAS" });
    
    // Compter les ligatures dans le texte brut
    let ligature_count = result_raw.text.chars()
        .filter(|&c| matches!(c, 'ﬁ' | 'ﬂ' | 'ﬀ' | 'ﬃ' | 'ﬄ'))
        .count();
    
    info!("📈 Statistiques:");
    info!("   🔤 Ligatures détectées dans le brut: {}", ligature_count);
    info!("   📏 Différence de taille: {} chars", 
          result_normalized.text.len() as i32 - result_raw.text.len() as i32);
    
    // Test 4: Normalisation manuelle pour vérification
    info!("🧪 Test 4: Normalisation manuelle");
    let manual_normalized = normalize_for_rag(&result_raw.text);
    let manual_matches = manual_normalized == result_normalized.text;
    
    info!("🔬 Vérification cohérence: {}", 
          if manual_matches { "✅ CONFORME" } else { "❌ DIFFÉRENCE DÉTECTÉE" });
    
    if !manual_matches {
        let diff_len = manual_normalized.len() as i32 - result_normalized.text.len() as i32;
        info!("   📏 Différence taille manuelle vs pipeline: {} chars", diff_len);
    }

    // Test 5: Recherche d'exemples de normalisation
    info!("🔍 Test 5: Exemples de normalisation trouvés");
    
    let examples = find_normalization_examples(&result_raw.text, &result_normalized.text);
    if examples.is_empty() {
        info!("   ℹ️ Aucun exemple de normalisation trouvé");
    } else {
        info!("   🎯 {} exemples trouvés:", examples.len().min(5));
        for (i, (before, after)) in examples.iter().take(5).enumerate() {
            info!("   {}. '{}' → '{}'", i + 1, before, after);
        }
    }

    // Sauvegarder un rapport de comparaison
    let report = format!(
        "# Rapport de Normalisation Unicode - GRAVIS OCR\n\n\
        ## Statistiques\n\
        - Texte brut: {} caractères, {} mots\n\
        - Texte normalisé: {} caractères, {} mots\n\
        - Ligatures détectées: {}\n\
        - Normalisation nécessaire: {}\n\
        - Cohérence pipeline: {}\n\n\
        ## Exemples de normalisation\n{}\n",
        result_raw.text.len(),
        result_raw.text.split_whitespace().count(),
        result_normalized.text.len(),
        result_normalized.text.split_whitespace().count(),
        ligature_count,
        if needs_norm { "Oui" } else { "Non" },
        if manual_matches { "✅ OK" } else { "❌ Problème" },
        examples.iter()
            .take(10)
            .map(|(before, after)| format!("- '{}' → '{}'", before, after))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    let report_file = output_dir.join("normalization_report.md");
    fs::write(&report_file, &report).await?;
    info!("📊 Rapport sauvegardé: {:?}", report_file);

    info!("📁 Tous les résultats sauvegardés dans: {:?}", output_dir);
    info!("✅ Test de normalisation Unicode terminé !");

    Ok(())
}

/// Trouve des exemples concrets de normalisation
fn find_normalization_examples(before: &str, after: &str) -> Vec<(String, String)> {
    let mut examples = Vec::new();
    
    // Diviser en mots pour comparaison
    let words_before: Vec<&str> = before.split_whitespace().collect();
    let words_after: Vec<&str> = after.split_whitespace().collect();
    
    // Comparer mot par mot (approximatif)
    for (word_before, word_after) in words_before.iter().zip(words_after.iter()) {
        if word_before != word_after && word_before.len() > 2 && word_after.len() > 2 {
            // Vérifier si c'est vraiment une normalisation (pas juste une différence)
            if word_before.chars().any(|c| matches!(c, 'ﬁ' | 'ﬂ' | 'ﬀ' | 'ﬃ' | 'ﬄ')) {
                examples.push((word_before.to_string(), word_after.to_string()));
            }
        }
    }
    
    examples
}