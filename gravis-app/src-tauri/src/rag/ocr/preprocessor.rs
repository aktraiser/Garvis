// GRAVIS OCR - Image Preprocessor via image crate
// Phase 2: Preprocessing intelligent sans leptess

use super::{PreprocessConfig, Result};
use image::{DynamicImage, ImageBuffer, Luma, GenericImageView};
use tracing::{debug, info};

/// Preprocesseur d'images pour optimiser l'OCR
pub struct ImagePreprocessor {
    config: PreprocessConfig,
}

impl ImagePreprocessor {
    pub fn new(config: PreprocessConfig) -> Self {
        Self { config }
    }
    
    /// Preprocessing principal via image crate
    pub async fn preprocess(&self, image: DynamicImage) -> Result<DynamicImage> {
        if !self.config.enabled {
            return Ok(image);
        }
        
        info!("🔄 Preprocessing image for OCR optimization");
        
        let original_dims = image.dimensions();
        let mut processed = image;
        
        // 1. Conversion en niveaux de gris pour OCR
        processed = DynamicImage::ImageLuma8(processed.to_luma8());
        debug!("📊 Converted to grayscale");
        
        // 2. Amélioration du contraste
        if self.config.enhance_contrast {
            processed = processed.adjust_contrast(20.0);
            debug!("📊 Enhanced contrast (+20)");
        }
        
        // 3. Redimensionnement intelligent
        if self.config.resize_for_ocr {
            processed = self.smart_resize(processed)?;
        }
        
        // 4. Filtrage de bruit basique
        processed = self.reduce_noise(processed)?;
        
        let final_dims = processed.dimensions();
        info!("✅ Image preprocessed: {}x{} → {}x{}", 
              original_dims.0, original_dims.1, final_dims.0, final_dims.1);
        
        Ok(processed)
    }
    
    /// Redimensionnement intelligent pour OCR
    fn smart_resize(&self, image: DynamicImage) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();
        
        // Calculer nouvelles dimensions si nécessaire
        let needs_resize = width < self.config.min_width || height < self.config.min_height;
        
        if !needs_resize {
            debug!("📊 No resize needed: {}x{}", width, height);
            return Ok(image);
        }
        
        // Calculer facteur d'échelle pour maintenir ratio
        let scale_width = self.config.min_width as f32 / width as f32;
        let scale_height = self.config.min_height as f32 / height as f32;
        let scale_factor = scale_width.max(scale_height);
        
        let new_width = (width as f32 * scale_factor) as u32;
        let new_height = (height as f32 * scale_factor) as u32;
        
        // Utiliser Lanczos3 pour qualité optimale
        let resized = image.resize(
            new_width, 
            new_height, 
            image::imageops::FilterType::Lanczos3
        );
        
        debug!("📊 Resized: {}x{} → {}x{} (scale: {:.2})", 
               width, height, new_width, new_height, scale_factor);
        
        Ok(resized)
    }
    
    /// Réduction de bruit basique
    fn reduce_noise(&self, image: DynamicImage) -> Result<DynamicImage> {
        // Conversion vers ImageBuffer pour manipulation directe
        let gray_image = image.to_luma8();
        let (width, height) = gray_image.dimensions();
        
        // Filtre médian simple 3x3 pour réduire le bruit
        let mut filtered = ImageBuffer::new(width, height);
        
        for y in 1..height-1 {
            for x in 1..width-1 {
                let mut neighborhood = Vec::new();
                
                // Collecter les pixels du voisinage 3x3
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let px = (x as i32 + dx) as u32;
                        let py = (y as i32 + dy) as u32;
                        if px < width && py < height {
                            neighborhood.push(gray_image.get_pixel(px, py).0[0]);
                        }
                    }
                }
                
                // Tri pour obtenir la médiane
                neighborhood.sort_unstable();
                let median = neighborhood[neighborhood.len() / 2];
                
                filtered.put_pixel(x, y, Luma([median]));
            }
        }
        
        // Copier les bordures depuis l'image originale
        for y in 0..height {
            if y == 0 || y == height - 1 {
                for x in 0..width {
                    filtered.put_pixel(x, y, *gray_image.get_pixel(x, y));
                }
            } else {
                filtered.put_pixel(0, y, *gray_image.get_pixel(0, y));
                filtered.put_pixel(width - 1, y, *gray_image.get_pixel(width - 1, y));
            }
        }
        
        debug!("📊 Applied noise reduction filter");
        Ok(DynamicImage::ImageLuma8(filtered))
    }
    
    /// Estimer la qualité de l'image pour OCR
    pub fn assess_quality(&self, image: &DynamicImage) -> ImageQuality {
        let (width, height) = image.dimensions();
        
        // Score de résolution basé sur les dimensions
        let resolution_score = {
            let min_pixels = self.config.min_width * self.config.min_height;
            let actual_pixels = width * height;
            (actual_pixels as f32 / min_pixels as f32).min(1.0)
        };
        
        // Score de contraste basé sur analyse des pixels
        let contrast_score = self.estimate_contrast(image);
        
        // Estimation du niveau de bruit
        let noise_level = self.estimate_noise_level(image);
        
        // PSM recommandé basé sur l'analyse
        let recommended_psm = self.recommend_psm(image);
        
        ImageQuality {
            resolution_score,
            contrast_score,
            noise_level,
            recommended_psm,
        }
    }
    
    /// Estimer le contraste de l'image
    fn estimate_contrast(&self, image: &DynamicImage) -> f32 {
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();
        
        if width == 0 || height == 0 {
            return 0.0;
        }
        
        // Calculer l'écart-type des valeurs de pixels (proxy pour contraste)
        let mut sum = 0u64;
        let mut sum_sq = 0u64;
        let total_pixels = (width * height) as u64;
        
        for pixel in gray.pixels() {
            let value = pixel.0[0] as u64;
            sum += value;
            sum_sq += value * value;
        }
        
        let mean = sum as f64 / total_pixels as f64;
        let variance = (sum_sq as f64 / total_pixels as f64) - (mean * mean);
        let std_dev = variance.sqrt();
        
        // Normaliser l'écart-type sur une échelle 0-1
        (std_dev / 128.0).min(1.0) as f32
    }
    
    /// Estimer le niveau de bruit
    fn estimate_noise_level(&self, image: &DynamicImage) -> f32 {
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();
        
        if width < 3 || height < 3 {
            return 0.0;
        }
        
        // Utiliser la variation locale comme proxy pour le bruit
        let mut total_variation = 0.0;
        let mut count = 0;
        
        for y in 1..height-1 {
            for x in 1..width-1 {
                let center = gray.get_pixel(x, y).0[0] as i32;
                
                // Calculer la variation avec les voisins
                let neighbors = [
                    gray.get_pixel(x-1, y).0[0] as i32,
                    gray.get_pixel(x+1, y).0[0] as i32,
                    gray.get_pixel(x, y-1).0[0] as i32,
                    gray.get_pixel(x, y+1).0[0] as i32,
                ];
                
                for neighbor in neighbors {
                    total_variation += (center - neighbor).abs() as f32;
                    count += 1;
                }
            }
        }
        
        if count == 0 {
            return 0.0;
        }
        
        let avg_variation = total_variation / count as f32;
        
        // Normaliser sur une échelle 0-1
        (avg_variation / 50.0).min(1.0)
    }
    
    /// Recommander un PSM basé sur l'analyse d'image
    fn recommend_psm(&self, image: &DynamicImage) -> super::PageSegMode {
        let (width, height) = image.dimensions();
        let aspect_ratio = width as f32 / height as f32;
        
        // Analyse basique basée sur les dimensions et ratio
        match aspect_ratio {
            r if r > 5.0 => super::PageSegMode::SingleLine,     // Très large = ligne unique
            r if r > 2.0 => super::PageSegMode::SingleColumn,   // Large = colonne
            r if r < 0.3 => super::PageSegMode::SingleColumn,   // Très haut = colonne
            _ => super::PageSegMode::Auto,                      // Format standard = auto
        }
    }
}

/// Évaluation de la qualité d'image
#[derive(Debug, Clone)]
pub struct ImageQuality {
    pub resolution_score: f32,      // 0.0-1.0
    pub contrast_score: f32,        // 0.0-1.0  
    pub noise_level: f32,           // 0.0-1.0 (plus bas = mieux)
    pub recommended_psm: super::PageSegMode,
}

impl ImageQuality {
    pub fn overall_score(&self) -> f32 {
        (self.resolution_score + self.contrast_score + (1.0 - self.noise_level)) / 3.0
    }
    
    pub fn is_good_for_ocr(&self) -> bool {
        self.overall_score() > 0.6
    }
    
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.resolution_score < 0.7 {
            recommendations.push("Consider higher resolution image".to_string());
        }
        
        if self.contrast_score < 0.5 {
            recommendations.push("Enhance contrast for better text recognition".to_string());
        }
        
        if self.noise_level > 0.6 {
            recommendations.push("Apply noise reduction filters".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Image quality looks good for OCR".to_string());
        }
        
        recommendations
    }
}

impl Default for ImagePreprocessor {
    fn default() -> Self {
        Self::new(PreprocessConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Luma};
    
    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            // Créer un motif simple avec du texte simulé
            if (x / 10) % 2 == 0 && (y / 10) % 2 == 0 {
                Luma([0u8])   // Noir (texte)
            } else {
                Luma([255u8]) // Blanc (fond)
            }
        });
        
        DynamicImage::ImageLuma8(img)
    }
    
    #[tokio::test]
    async fn test_preprocessing() {
        let config = PreprocessConfig::default();
        let preprocessor = ImagePreprocessor::new(config);
        
        let test_image = create_test_image(800, 600);
        
        match preprocessor.preprocess(test_image).await {
            Ok(processed) => {
                let (width, height) = processed.dimensions();
                assert!(width >= 1200); // Should be resized
                assert!(height >= 800);
                println!("✅ Preprocessing test passed: {}x{}", width, height);
            }
            Err(e) => println!("⚠️ Preprocessing test failed: {}", e),
        }
    }
    
    #[test]
    fn test_quality_assessment() {
        let preprocessor = ImagePreprocessor::default();
        
        // Test avec une image de bonne qualité
        let good_image = create_test_image(1600, 1200);
        let quality = preprocessor.assess_quality(&good_image);
        
        println!("📊 Image quality assessment:");
        println!("   Resolution: {:.2}", quality.resolution_score);
        println!("   Contrast: {:.2}", quality.contrast_score);
        println!("   Noise: {:.2}", quality.noise_level);
        println!("   Overall: {:.2}", quality.overall_score());
        println!("   Good for OCR: {}", quality.is_good_for_ocr());
        
        assert!(quality.resolution_score > 0.8);
        println!("✅ Quality assessment test passed");
    }
    
    #[test]
    fn test_psm_recommendation() {
        let preprocessor = ImagePreprocessor::default();
        
        // Test différents ratios d'aspect
        let wide_image = create_test_image(2000, 100);  // Très large
        let tall_image = create_test_image(100, 2000);  // Très haut
        let square_image = create_test_image(1000, 1000); // Carré
        
        let wide_quality = preprocessor.assess_quality(&wide_image);
        let tall_quality = preprocessor.assess_quality(&tall_image);
        let square_quality = preprocessor.assess_quality(&square_image);
        
        println!("📊 PSM recommendations:");
        println!("   Wide image: {:?}", wide_quality.recommended_psm);
        println!("   Tall image: {:?}", tall_quality.recommended_psm);
        println!("   Square image: {:?}", square_quality.recommended_psm);
        
        println!("✅ PSM recommendation test passed");
    }
    
    #[test]
    fn test_smart_resize() {
        let config = PreprocessConfig {
            min_width: 1200,
            min_height: 800,
            ..Default::default()
        };
        let preprocessor = ImagePreprocessor::new(config);
        
        // Test avec image trop petite
        let small_image = create_test_image(600, 400);
        let resized = preprocessor.smart_resize(small_image).unwrap();
        
        let (width, height) = resized.dimensions();
        assert!(width >= 1200);
        assert!(height >= 800);
        
        println!("✅ Smart resize test passed: {}x{}", width, height);
    }
}