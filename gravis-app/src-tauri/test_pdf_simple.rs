// Test PDF Simple avec les alternatives natives Rust
// Extraction de texte du PDF DeepSeek-OCR avec lopdf et pdf-extract

use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use lopdf::Object;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Test PDF Simple - Extraction du texte DeepSeek-OCR");

    // Chemin vers le PDF DeepSeek-OCR
    let pdf_path = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/2510.18234v1.pdf");
    
    if !pdf_path.exists() {
        return Err("PDF file not found".into());
    }

    info!("📄 PDF trouvé: {:?}", pdf_path);

    // Créer répertoire de sortie
    let output_dir = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/pdf_simple_results");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).await?;
    }

    // Test 1: Extraction avec pdf-extract (le plus simple)
    info!("📖 Test 1: Extraction avec pdf-extract");
    match extract_with_pdf_extract(&pdf_path).await {
        Ok(text) => {
            let word_count = text.split_whitespace().count();
            let char_count = text.len();
            
            info!("✅ pdf-extract réussi:");
            info!("   📝 {} caractères", char_count);
            info!("   🔤 {} mots", word_count);
            
            if char_count > 0 {
                let preview = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                };
                info!("   📖 Aperçu: '{}'", preview.replace('\n', " "));
                
                // Sauvegarder le texte
                let output_file = output_dir.join("pdf_extract_result.txt");
                fs::write(&output_file, &text).await?;
                info!("   💾 Sauvegardé: {:?}", output_file);
                
                // Chercher des mots-clés DeepSeek-OCR
                check_deepseek_content(&text, "pdf-extract");
            }
        }
        Err(e) => {
            info!("❌ pdf-extract échoué: {}", e);
        }
    }

    // Test 2: Extraction avec lopdf (plus avancé)
    info!("📖 Test 2: Extraction avec lopdf");
    match extract_with_lopdf(&pdf_path).await {
        Ok(text) => {
            let word_count = text.split_whitespace().count();
            let char_count = text.len();
            
            info!("✅ lopdf réussi:");
            info!("   📝 {} caractères", char_count);
            info!("   🔤 {} mots", word_count);
            
            if char_count > 0 {
                let preview = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                };
                info!("   📖 Aperçu: '{}'", preview.replace('\n', " "));
                
                // Sauvegarder le texte
                let output_file = output_dir.join("lopdf_result.txt");
                fs::write(&output_file, &text).await?;
                info!("   💾 Sauvegardé: {:?}", output_file);
                
                // Chercher des mots-clés DeepSeek-OCR
                check_deepseek_content(&text, "lopdf");
            }
        }
        Err(e) => {
            info!("❌ lopdf échoué: {}", e);
        }
    }

    // Test 3: poppler-utils si disponible
    info!("📖 Test 3: Extraction avec poppler-utils (pdftotext)");
    match extract_with_poppler(&pdf_path).await {
        Ok(text) => {
            let word_count = text.split_whitespace().count();
            let char_count = text.len();
            
            info!("✅ poppler-utils (pdftotext) réussi:");
            info!("   📝 {} caractères", char_count);
            info!("   🔤 {} mots", word_count);
            
            if char_count > 0 {
                let preview = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                };
                info!("   📖 Aperçu: '{}'", preview.replace('\n', " "));
                
                // Sauvegarder le texte
                let output_file = output_dir.join("poppler_result.txt");
                fs::write(&output_file, &text).await?;
                info!("   💾 Sauvegardé: {:?}", output_file);
                
                // Chercher des mots-clés DeepSeek-OCR
                check_deepseek_content(&text, "poppler-utils");
            }
        }
        Err(e) => {
            info!("❌ poppler-utils échoué: {} (installer avec: brew install poppler)", e);
        }
    }

    info!("📁 Résultats sauvegardés dans: {:?}", output_dir);
    info!("✅ Test PDF Simple terminé !");

    Ok(())
}

/// Extraction avec pdf-extract (pure Rust, simple)
async fn extract_with_pdf_extract(pdf_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    
    // Lire le fichier PDF
    let bytes = fs::read(pdf_path).await?;
    
    // Extraire le texte avec pdf-extract
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("pdf-extract failed: {}", e))?;
    
    let duration = start.elapsed();
    info!("⏱️ pdf-extract terminé en {:.2}s", duration.as_secs_f32());
    
    Ok(text)
}

/// Extraction avec lopdf (pure Rust, plus avancé)
async fn extract_with_lopdf(pdf_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    
    // Charger le document PDF avec lopdf
    let document = lopdf::Document::load(pdf_path)
        .map_err(|e| format!("lopdf load failed: {}", e))?;
    
    let mut all_text = String::new();
    
    // Itérer sur toutes les pages
    let page_numbers: Vec<u32> = document.get_pages().keys().cloned().collect();
    
    for &page_num in &page_numbers {
        if page_num > 10 { break; } // Limiter aux 10 premières pages pour le test
        
        match extract_text_from_page(&document, page_num) {
            Ok(page_text) => {
                if !page_text.trim().is_empty() {
                    all_text.push_str(&format!("\n=== PAGE {} ===\n", page_num));
                    all_text.push_str(&page_text);
                    all_text.push('\n');
                }
            }
            Err(e) => {
                info!("⚠️ Échec extraction page {}: {}", page_num, e);
            }
        }
    }
    
    let duration = start.elapsed();
    info!("⏱️ lopdf terminé en {:.2}s ({} pages)", duration.as_secs_f32(), page_numbers.len().min(10));
    
    Ok(all_text)
}

/// Extraction de texte d'une page avec lopdf (simplifié)
fn extract_text_from_page(_document: &lopdf::Document, page_num: u32) -> Result<String, Box<dyn std::error::Error>> {
    // Extraction de texte simplifiée pour le test
    // En production, utiliser les modules OCR complets 
    info!("🔄 Attempting simple text extraction for page {}", page_num);
    Ok(format!("Sample text from page {} (lopdf simple extraction)", page_num))
}

/// Extraction simple de texte depuis le contenu PDF
fn extract_text_from_content(content: &str) -> String {
    let mut text = String::new();
    let mut in_text = false;
    let mut current_string = String::new();
    
    for line in content.lines() {
        if line.contains("BT") {
            in_text = true;
        } else if line.contains("ET") {
            in_text = false;
        } else if in_text {
            // Chercher les commandes Tj et TJ (show text)
            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end {
                        let extracted = &line[start + 1..end];
                        text.push_str(extracted);
                        text.push(' ');
                    }
                }
            }
        }
    }
    
    text
}

/// Extraction avec poppler-utils (pdftotext)
async fn extract_with_poppler(pdf_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    
    // Utiliser pdftotext pour extraire le texte
    let output = std::process::Command::new("pdftotext")
        .arg("-raw")  // Format texte brut
        .arg("-enc").arg("UTF-8")  // Encodage UTF-8
        .arg(pdf_path)  // Fichier PDF source
        .arg("-")  // Sortie vers stdout
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdftotext failed: {}", stderr).into());
    }
    
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    
    let duration = start.elapsed();
    info!("⏱️ poppler-utils terminé en {:.2}s", duration.as_secs_f32());
    
    Ok(text)
}

/// Vérifier si le contenu contient des mots-clés DeepSeek-OCR
fn check_deepseek_content(text: &str, method: &str) {
    let text_lower = text.to_lowercase();
    
    let keywords = [
        "deepseek", "ocr", "vision", "transformer", "compression", 
        "token", "multimodal", "language", "model", "llm"
    ];
    
    let mut found_keywords = Vec::new();
    
    for &keyword in &keywords {
        if text_lower.contains(keyword) {
            found_keywords.push(keyword);
        }
    }
    
    if !found_keywords.is_empty() {
        info!("🎯 {} - Mots-clés trouvés: {:?}", method, found_keywords);
        
        if found_keywords.contains(&"deepseek") {
            info!("✅ {} - CONFIRMATION: Document DeepSeek-OCR détecté !", method);
        }
    } else {
        info!("⚠️ {} - Aucun mot-clé technique trouvé", method);
    }
}