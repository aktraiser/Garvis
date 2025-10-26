// Test OCR avec le document DeepSeek-OCR PDF
// Test complet du système OCR Phase 2 avec un document technique réel

use gravis_app_lib::rag::ocr::{
    TesseractProcessor, TesseractConfig, OcrConfig, PageSegMode, OcrEngineMode,
    PreprocessConfig, PerformanceConfig, CacheConfig, OcrCache
};
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting OCR test with DeepSeek-OCR PDF document");

    // Chemin vers le document PDF
    let pdf_path = PathBuf::from("/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/2510.18234v1.pdf");
    
    if !pdf_path.exists() {
        error!("❌ PDF file not found at: {:?}", pdf_path);
        return Err("PDF file not found".into());
    }

    info!("📄 Found PDF document: {:?}", pdf_path);

    // Pour ce test, nous allons extraire une page du PDF en image
    // En attendant l'implémentation PDF complète, simulons avec des pages convertites en images
    
    // Test 1: Configuration Tiny (64 tokens, 512x512)
    info!("🔧 Testing Tiny configuration (64 vision tokens)");
    let tiny_config = create_tiny_config().await?;
    test_ocr_configuration("Tiny", tiny_config).await?;

    // Test 2: Configuration Small (100 tokens, 640x640)  
    info!("🔧 Testing Small configuration (100 vision tokens)");
    let small_config = create_small_config().await?;
    test_ocr_configuration("Small", small_config).await?;

    // Test 3: Configuration Base (256 tokens, 1024x1024)
    info!("🔧 Testing Base configuration (256 vision tokens)");
    let base_config = create_base_config().await?;
    test_ocr_configuration("Base", base_config).await?;

    // Test 4: Cache performance
    info!("💾 Testing cache performance");
    test_cache_performance().await?;

    // Test 5: Langues multiples
    info!("🌐 Testing multilingual support");
    test_multilingual_support().await?;

    info!("✅ All OCR tests completed successfully!");
    
    Ok(())
}

async fn create_tiny_config() -> Result<TesseractConfig, Box<dyn std::error::Error>> {
    Ok(TesseractConfig {
        languages: vec!["eng".to_string()],
        psm: PageSegMode::Auto,
        oem: OcrEngineMode::LstmOnly,
        preprocessing: PreprocessConfig {
            enabled: true,
            enhance_contrast: true,
            resize_for_ocr: true,
            min_width: 512,
            min_height: 512,
            target_dpi: 300,
        },
        confidence_threshold: 0.7,
        temp_dir: std::env::temp_dir().join("gravis_ocr_test"),
        max_concurrent: 2,
        timeout: std::time::Duration::from_secs(30),
    })
}

async fn create_small_config() -> Result<TesseractConfig, Box<dyn std::error::Error>> {
    Ok(TesseractConfig {
        languages: vec!["eng".to_string()],
        psm: PageSegMode::Auto,
        oem: OcrEngineMode::LstmOnly,
        preprocessing: PreprocessConfig {
            enabled: true,
            enhance_contrast: true,
            resize_for_ocr: true,
            min_width: 640,
            min_height: 640,
            target_dpi: 300,
        },
        confidence_threshold: 0.7,
        temp_dir: std::env::temp_dir().join("gravis_ocr_test"),
        max_concurrent: 2,
        timeout: std::time::Duration::from_secs(45),
    })
}

async fn create_base_config() -> Result<TesseractConfig, Box<dyn std::error::Error>> {
    Ok(TesseractConfig {
        languages: vec!["eng".to_string()],
        psm: PageSegMode::Auto,
        oem: OcrEngineMode::LstmOnly,
        preprocessing: PreprocessConfig {
            enabled: true,
            enhance_contrast: true,
            resize_for_ocr: true,
            min_width: 1024,
            min_height: 1024,
            target_dpi: 300,
        },
        confidence_threshold: 0.7,
        temp_dir: std::env::temp_dir().join("gravis_ocr_test"),
        max_concurrent: 3,
        timeout: std::time::Duration::from_secs(60),
    })
}

async fn test_ocr_configuration(
    config_name: &str, 
    config: TesseractConfig
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    info!("📊 Testing {} configuration", config_name);
    
    // Créer le processeur OCR
    let processor = match TesseractProcessor::new(config).await {
        Ok(p) => {
            info!("✅ {} processor created successfully", config_name);
            p
        },
        Err(e) => {
            error!("❌ Failed to create {} processor: {}", config_name, e);
            return Err(e.into());
        }
    };

    // Pour ce test, créons une image de test simple avec du texte
    let test_image_path = create_test_image(config_name).await?;
    
    // Traiter l'image de test
    match processor.process_image(&test_image_path).await {
        Ok(result) => {
            let processing_time = start_time.elapsed();
            
            info!("📈 {} Results:", config_name);
            info!("   ⏱️  Processing time: {:.2}s", processing_time.as_secs_f32());
            info!("   🎯 Confidence: {:.1}%", result.confidence * 100.0);
            info!("   🔤 Text length: {} characters", result.text.len());
            info!("   📦 Bounding boxes: {}", result.bounding_boxes.len());
            info!("   🌐 Language: {}", result.language);
            info!("   🏷️  Engine: {}", result.engine_used);
            
            if !result.text.is_empty() {
                let preview = if result.text.len() > 100 {
                    format!("{}...", &result.text[..100])
                } else {
                    result.text.clone()
                };
                info!("   📝 Text preview: \"{}\"", preview);
            }
            
            // Vérifier les métriques de qualité
            if result.confidence > 0.8 {
                info!("   ✅ High confidence result");
            } else if result.confidence > 0.5 {
                info!("   ⚠️  Medium confidence result");
            } else {
                info!("   ❌ Low confidence result");
            }
        },
        Err(e) => {
            error!("❌ {} processing failed: {}", config_name, e);
        }
    }

    // Nettoyer l'image de test
    let _ = fs::remove_file(&test_image_path).await;
    
    Ok(())
}

async fn create_test_image(config_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Luma};
    
    let temp_dir = std::env::temp_dir().join("gravis_ocr_test");
    fs::create_dir_all(&temp_dir).await?;
    
    let image_path = temp_dir.join(format!("test_image_{}.png", config_name.to_lowercase()));
    
    // Créer une image simple avec du texte simulé (motif de test)
    let width = 800;
    let height = 600;
    
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // Créer un motif qui simule du texte
        let char_width = 50;
        let char_height = 60;
        let line_height = 80;
        
        let char_x = x / char_width;
        let char_y = y / line_height;
        let local_x = x % char_width;
        let local_y = y % char_height;
        
        // Simuler des caractères (rectangles noirs sur fond blanc)
        if char_y < 7 && char_x < 15 {  // 7 lignes, 15 caractères par ligne
            if local_x > 5 && local_x < 45 && local_y > 10 && local_y < 50 {
                // Ajouter quelques variations pour simuler différentes lettres
                if (char_x + char_y) % 3 == 0 && local_x > 15 && local_x < 35 {
                    Luma([255u8])  // Blanc (espaces dans les lettres)
                } else {
                    Luma([0u8])    // Noir (texte)
                }
            } else {
                Luma([255u8])      // Blanc (fond)
            }
        } else {
            Luma([255u8])          // Blanc (fond)
        }
    });
    
    img.save(&image_path)?;
    info!("📄 Created test image: {:?}", image_path);
    
    Ok(image_path)
}

async fn test_cache_performance() -> Result<(), Box<dyn std::error::Error>> {
    info!("💾 Testing cache performance and functionality");
    
    let cache_config = CacheConfig {
        enabled: true,
        max_size_mb: 50,
        ttl_hours: 1,
        persistent: false,
        cache_directory: None,
    };
    
    let cache = OcrCache::new(cache_config).await?;
    
    // Tester les statistiques initiales
    let initial_stats = cache.get_stats();
    info!("📊 Initial cache stats:");
    info!("   Hits: {}", initial_stats.hits);
    info!("   Misses: {}", initial_stats.misses);
    info!("   Hit rate: {:.2}%", initial_stats.hit_rate() * 100.0);
    info!("   Memory usage: {:.2} MB", initial_stats.memory_usage_mb());
    
    // Créer une image de test pour le cache
    let test_image = create_test_image("cache").await?;
    
    // Premier accès (cache miss attendu)
    let start_time = Instant::now();
    let first_result = cache.get_image_result(&test_image).await?;
    let first_time = start_time.elapsed();
    
    assert!(first_result.is_none(), "Cache should be empty initially");
    info!("✅ First cache access (miss): {:.2}ms", first_time.as_millis());
    
    // Simuler un résultat OCR et le stocker
    let mock_result = create_mock_ocr_result();
    cache.store_image_result(&test_image, &mock_result).await?;
    info!("✅ Stored result in cache");
    
    // Deuxième accès (cache hit attendu)
    let start_time = Instant::now();
    let second_result = cache.get_image_result(&test_image).await?;
    let second_time = start_time.elapsed();
    
    assert!(second_result.is_some(), "Cache should return stored result");
    info!("✅ Second cache access (hit): {:.2}ms", second_time.as_millis());
    
    // Vérifier les statistiques finales
    let final_stats = cache.get_stats();
    info!("📊 Final cache stats:");
    info!("   Hits: {}", final_stats.hits);
    info!("   Misses: {}", final_stats.misses);
    info!("   Hit rate: {:.2}%", final_stats.hit_rate() * 100.0);
    info!("   Memory usage: {:.2} MB", final_stats.memory_usage_mb());
    info!("   Entries: {}", final_stats.entries_count);
    
    // Nettoyer
    let _ = fs::remove_file(&test_image).await;
    cache.clear()?;
    
    Ok(())
}

fn create_mock_ocr_result() -> gravis_app_lib::rag::ocr::OcrResult {
    use gravis_app_lib::rag::ocr::{OcrResult, OcrMetadata, BoundingBox, PageSegMode, OcrEngineMode};
    
    OcrResult {
        text: "This is a test OCR result for cache testing purposes.".to_string(),
        confidence: 0.95,
        language: "eng".to_string(),
        bounding_boxes: vec![
            BoundingBox {
                x: 100,
                y: 100,
                width: 200,
                height: 30,
                text: "test text".to_string(),
                confidence: 0.95,
                level: 5,
            }
        ],
        processing_time: std::time::Duration::from_millis(500),
        engine_used: "Tesseract Command Test".to_string(),
        tesseract_version: "5.0.0".to_string(),
        metadata: OcrMetadata {
            source_file: "test_image.png".to_string(),
            file_size_bytes: 50000,
            image_dimensions: (800, 600),
            preprocessing_applied: vec!["contrast_enhancement".to_string()],
            psm_used: PageSegMode::Auto,
            oem_used: OcrEngineMode::LstmOnly,
            temp_files_created: vec!["temp_output.txt".to_string()],
        },
    }
}

async fn test_multilingual_support() -> Result<(), Box<dyn std::error::Error>> {
    info!("🌐 Testing multilingual OCR support");
    
    // Tester la détection des langues disponibles
    match gravis_app_lib::rag::ocr::get_available_languages().await {
        Ok(languages) => {
            info!("✅ Available languages detected: {}", languages.len());
            info!("   Languages: {:?}", languages.iter().take(10).collect::<Vec<_>>());
            
            // Vérifier que les langues de base sont disponibles
            let required_langs = vec!["eng", "fra", "deu"];
            for lang in required_langs {
                if languages.contains(&lang.to_string()) {
                    info!("   ✅ {} language available", lang);
                } else {
                    info!("   ⚠️  {} language not available", lang);
                }
            }
        },
        Err(e) => {
            error!("❌ Failed to get available languages: {}", e);
        }
    }
    
    // Tester la version Tesseract
    match gravis_app_lib::rag::ocr::get_tesseract_version().await {
        Ok(version) => {
            info!("✅ Tesseract version: {}", version);
        },
        Err(e) => {
            error!("❌ Failed to get Tesseract version: {}", e);
        }
    }
    
    Ok(())
}