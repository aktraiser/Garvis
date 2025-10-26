// Test OCR avec aperçu visuel des images générées
// Permet de voir exactement ce qui est traité par Tesseract

use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig, PageSegMode, OcrEngineMode,
    PreprocessConfig
};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contrast::otsu_level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting OCR test with visual preview of generated images");

    // Créer le répertoire de sortie pour les aperçus
    let output_dir = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/ocr_preview");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).await?;
    }

    info!("📁 Output directory: {:?}", output_dir);

    // Test 1: Image simple avec du texte clair
    info!("📝 Test 1: Simple text image");
    test_simple_text_image(&output_dir).await?;

    // Test 2: Image avec différents niveaux de complexité
    info!("📝 Test 2: Complex text patterns");
    test_complex_patterns(&output_dir).await?;

    // Test 3: Comparaison preprocessing ON/OFF
    info!("📝 Test 3: Preprocessing comparison");
    test_preprocessing_comparison(&output_dir).await?;

    // Test 4: Document-like image
    info!("📝 Test 4: Document simulation");
    test_document_simulation(&output_dir).await?;

    info!("✅ All visual tests completed! Check images in: {:?}", output_dir);
    info!("📖 Open the output directory to see exactly what Tesseract processes");
    
    Ok(())
}

async fn test_simple_text_image(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Creating simple text image");
    
    // Créer une image simple avec du texte lisible
    let width = 800;
    let height = 200;
    
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // Texte "HELLO WORLD TEST" centré
        let text_start_x = 50;
        let text_y = 50;
        let char_width = 60;
        let char_height = 80;
        
        // Position du caractère
        let char_index = (x.saturating_sub(text_start_x)) / char_width;
        let local_x = (x.saturating_sub(text_start_x)) % char_width;
        let local_y = y.saturating_sub(text_y);
        
        // Définir les lettres comme des blocs simples
        if y >= text_y && y < text_y + char_height && char_index < 11 {
            match char_index {
                0 | 2 | 4 | 5 | 7 | 9 | 10 => { // H, L, O, space, W, R, D
                    if local_x > 10 && local_x < 50 && local_y > 10 && local_y < 70 {
                        if char_index == 3 { // espace
                            Luma([255u8]) // blanc
                        } else {
                            Luma([0u8]) // noir (texte)
                        }
                    } else {
                        Luma([255u8]) // blanc (fond)
                    }
                }
                _ => Luma([255u8]) // autres caractères en blanc
            }
        } else {
            Luma([255u8]) // fond blanc
        }
    });

    let image_path = output_dir.join("01_simple_text_original.png");
    img.save(&image_path)?;
    info!("💾 Saved original: {:?}", image_path);

    // Tester avec OCR
    let result = process_image_with_ocr(&image_path, "simple").await?;
    info!("📊 Simple text result: confidence={:.1}%, text_len={}", 
          result.confidence * 100.0, result.text.len());
    info!("📝 Extracted text: '{}'", result.text.trim());

    Ok(())
}

async fn test_complex_patterns(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Creating complex patterns");
    
    let width = 1000;
    let height = 600;
    
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // Simuler différents types de contenu:
        // - Titre en haut
        // - Paragraphe au milieu
        // - Tableaux en bas
        
        if y < 100 {
            // Zone titre - gros texte
            if (x / 80) % 2 == 0 && (y % 40) < 30 {
                Luma([0u8]) // texte titre
            } else {
                Luma([255u8]) // fond
            }
        } else if y < 300 {
            // Zone paragraphe - texte normal
            let line_y = (y - 100) / 40;
            let char_x = x / 20;
            if line_y < 4 && (char_x % 3) != 0 && (y % 40) < 25 {
                Luma([0u8]) // texte paragraphe
            } else {
                Luma([255u8]) // fond
            }
        } else {
            // Zone tableau - structure géométrique
            if (x % 100) < 5 || (y % 50) < 3 {
                Luma([0u8]) // lignes du tableau
            } else if ((x / 100) + (y / 50)) % 2 == 0 {
                Luma([200u8]) // cellules grises
            } else {
                Luma([255u8]) // cellules blanches
            }
        }
    });

    let image_path = output_dir.join("02_complex_patterns_original.png");
    img.save(&image_path)?;
    info!("💾 Saved complex patterns: {:?}", image_path);

    let result = process_image_with_ocr(&image_path, "complex").await?;
    info!("📊 Complex patterns result: confidence={:.1}%, text_len={}", 
          result.confidence * 100.0, result.text.len());

    Ok(())
}

async fn test_preprocessing_comparison(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Testing preprocessing effects");
    
    // Créer une image avec du bruit pour tester le preprocessing
    let width = 600;
    let height = 400;
    
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // Texte avec du bruit de fond
        let text_x = x / 40;
        let text_y = y / 60;
        let local_x = x % 40;
        let local_y = y % 60;
        
        // Ajouter du bruit de fond
        let noise = ((x * 7 + y * 11) % 100) as u8;
        
        if text_y < 6 && text_x < 12 {
            if local_x > 5 && local_x < 35 && local_y > 10 && local_y < 50 {
                // Zone de texte avec léger bruit
                if noise < 20 {
                    Luma([50u8]) // presque noir avec bruit
                } else {
                    Luma([0u8]) // noir pur
                }
            } else {
                // Fond avec bruit
                Luma([200 + (noise / 4)])
            }
        } else {
            // Fond uniquement avec bruit
            Luma([220 + (noise / 10)])
        }
    });

    let noisy_path = output_dir.join("03_noisy_original.png");
    img.save(&noisy_path)?;
    info!("💾 Saved noisy image: {:?}", noisy_path);

    // Appliquer manuellement le preprocessing pour voir la différence
    let gray = DynamicImage::ImageLuma8(img);
    let gray_luma = gray.to_luma8();
    let threshold = otsu_level(&gray_luma);
    
    let binary = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = gray_luma.get_pixel(x, y).0[0];
        Luma([if pixel > threshold { 255u8 } else { 0u8 }])
    });

    let binary_path = output_dir.join("03_noisy_otsu_processed.png");
    binary.save(&binary_path)?;
    info!("💾 Saved Otsu processed: {:?}", binary_path);
    info!("🔍 Otsu threshold used: {}", threshold);

    // Tester les deux versions
    let result_original = process_image_with_ocr(&noisy_path, "noisy_original").await?;
    let result_processed = process_image_with_ocr(&binary_path, "noisy_processed").await?;
    
    info!("📊 Preprocessing comparison:");
    info!("   Original: confidence={:.1}%, text_len={}", 
          result_original.confidence * 100.0, result_original.text.len());
    info!("   Processed: confidence={:.1}%, text_len={}", 
          result_processed.confidence * 100.0, result_processed.text.len());

    Ok(())
}

async fn test_document_simulation(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Creating document-like image");
    
    let width = 1200;
    let height = 800;
    
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // Simuler une page de document avec:
        // - Marges
        // - Lignes de texte régulières
        // - Espacement cohérent
        
        let margin = 100;
        let line_height = 40;
        let char_width = 12;
        
        if x < margin || x > width - margin || y < margin || y > height - margin {
            Luma([255u8]) // marges blanches
        } else {
            let content_y = y - margin;
            let line_index = content_y / line_height;
            let line_y = content_y % line_height;
            
            if line_y < 25 && line_index < 15 {
                let char_x = (x - margin) / char_width;
                let local_x = (x - margin) % char_width;
                
                // Simuler du texte avec des espaces
                if char_x % 6 == 5 {
                    Luma([255u8]) // espace entre mots
                } else if local_x > 2 && local_x < 10 && line_y > 5 && line_y < 20 {
                    Luma([0u8]) // caractères
                } else {
                    Luma([255u8]) // fond
                }
            } else {
                Luma([255u8]) // espaces entre lignes
            }
        }
    });

    let doc_path = output_dir.join("04_document_simulation.png");
    img.save(&doc_path)?;
    info!("💾 Saved document simulation: {:?}", doc_path);

    let result = process_image_with_ocr(&doc_path, "document").await?;
    info!("📊 Document simulation result: confidence={:.1}%, text_len={}", 
          result.confidence * 100.0, result.text.len());
    info!("📝 Document text preview: '{}'", 
          result.text.lines().take(3).collect::<Vec<_>>().join(" | "));

    Ok(())
}

async fn process_image_with_ocr(image_path: &PathBuf, test_name: &str) -> 
    Result<gravis_app_lib::rag::ocr::OcrResult, Box<dyn std::error::Error>> {
    
    let config = TesseractConfig {
        languages: vec!["eng".to_string()],
        psm: PageSegMode::SingleBlock,
        oem: OcrEngineMode::LstmOnly,
        preprocessing: PreprocessConfig {
            enabled: true,
            enhance_contrast: true,
            resize_for_ocr: false, // garder taille originale pour le test
            min_width: 400,
            min_height: 300,
            target_dpi: 300,
        },
        confidence_threshold: 0.6,
        temp_dir: std::env::temp_dir().join("gravis_ocr_visual_test"),
        max_concurrent: 1,
        timeout: std::time::Duration::from_secs(30),
    };

    let processor = TesseractProcessor::new(config).await?;
    let result = processor.process_image(image_path).await?;
    
    info!("🔍 OCR result for '{}': {:.1}% confidence, {} chars, {} boxes", 
          test_name, result.confidence * 100.0, result.text.len(), result.bounding_boxes.len());
    
    Ok(result)
}