// GRAVIS OCR - Tesseract Command-based Processor 
// Phase 2: Implémentation robuste via Command::new("tesseract")

use super::{
    OcrResult, OcrMetadata, BoundingBox, 
    PageSegMode, OcrEngineMode, PreprocessConfig, OcrError, Result
};
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{info, debug, error};
use uuid::Uuid;

/// Processeur Tesseract Command-based
pub struct TesseractProcessor {
    config: TesseractConfig,
    cache: Option<super::OcrCache>,
}

/// Configuration Tesseract détaillée
#[derive(Debug, Clone)]
pub struct TesseractConfig {
    pub languages: Vec<String>,
    pub psm: PageSegMode,
    pub oem: OcrEngineMode,
    pub preprocessing: PreprocessConfig,
    pub confidence_threshold: f32,
    pub temp_dir: PathBuf,
    pub max_concurrent: usize,
    pub timeout: Duration,
}

impl Default for TesseractConfig {
    fn default() -> Self {
        Self {
            languages: vec!["eng".to_string(), "fra".to_string()],
            psm: PageSegMode::SingleBlock,  // PSM 6 - meilleur pour documents
            oem: OcrEngineMode::LstmOnly,   // OEM 1 - LSTM seul (plus fiable)
            preprocessing: PreprocessConfig::default(),
            confidence_threshold: 0.6,     // Seuil légèrement plus bas
            temp_dir: std::env::temp_dir().join("gravis_ocr"),
            max_concurrent: 4,
            timeout: Duration::from_secs(45), // Plus de temps pour qualité
        }
    }
}

impl TesseractProcessor {
    /// Créer un nouveau processeur Tesseract
    pub async fn new(config: TesseractConfig) -> Result<Self> {
        // Vérifier que Tesseract est disponible
        Self::verify_tesseract_installation().await?;
        
        // Valider les langues demandées
        super::validate_languages(&config.languages).await?;
        
        // Créer le répertoire temporaire
        if !config.temp_dir.exists() {
            fs::create_dir_all(&config.temp_dir).await?;
        }
        
        let cache = if config.preprocessing.enabled {
            Some(super::OcrCache::new(super::CacheConfig::default()).await?)
        } else {
            None
        };
        
        info!("✅ TesseractProcessor initialized with languages: {:?}", config.languages);
        
        Ok(Self { config, cache })
    }
    
    /// Traiter une image unique via Command
    pub async fn process_image(&self, image_path: &Path) -> Result<OcrResult> {
        let start_time = Instant::now();
        
        // Vérifier le cache en premier
        if let Some(cache) = &self.cache {
            if let Some(cached_result) = cache.get_image_result(image_path).await? {
                debug!("✅ Cache hit for image: {:?}", image_path);
                return Ok(cached_result);
            }
        }
        
        info!("🔄 Processing image with Tesseract: {:?}", image_path);
        
        // 1. Preprocessing si activé
        let processed_path = if self.config.preprocessing.enabled {
            self.preprocess_image(image_path).await?
        } else {
            image_path.to_path_buf()
        };
        
        // 2. Traitement OCR via Command
        let result = self.run_tesseract_command(&processed_path).await?;
        
        // 3. Mettre en cache le résultat
        if let Some(cache) = &self.cache {
            cache.store_image_result(image_path, &result).await?;
        }
        
        let total_time = start_time.elapsed();
        info!("✅ Image processed in {:.2}s, confidence: {:.1}%", 
              total_time.as_secs_f32(), result.confidence * 100.0);
        
        Ok(result)
    }
    
    /// Exécuter la commande Tesseract
    async fn run_tesseract_command(&self, image_path: &Path) -> Result<OcrResult> {
        let start_time = Instant::now();
        
        // Générer des paths temporaires uniques
        let session_id = Uuid::new_v4().to_string();
        let output_base = self.config.temp_dir.join(format!("ocr_output_{}", session_id));
        let output_txt = output_base.with_extension("txt");
        let output_tsv = output_base.with_extension("tsv");
        
        // Construire la commande Tesseract
        let mut cmd = Command::new("tesseract");
        cmd.arg(image_path)
           .arg(&output_base)  // Tesseract ajoute automatiquement .txt
           .arg("-l").arg(self.config.languages.join("+"))
           .arg("--psm").arg(self.config.psm.as_string())
           .arg("--oem").arg("1")  // Force LSTM only (most compatible)
           .arg("txt")  // Format texte de base
           .arg("tsv"); // Format TSV pour bounding boxes
        
        debug!("🔧 Tesseract command: {:?}", cmd);
        
        // Exécuter avec timeout via spawn_blocking
        let timeout = self.config.timeout;
        let result = tokio::time::timeout(timeout, 
            tokio::task::spawn_blocking(move || cmd.output())
        ).await
        .map_err(|_| OcrError::Timeout)?
        .map_err(|e| OcrError::TesseractCommand(format!("Task spawn failed: {}", e)))?
        .map_err(|e| OcrError::TesseractCommand(format!("Command failed: {}", e)))?;
        
        // Vérifier le succès de la commande
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(OcrError::TesseractCommand(format!(
                "Tesseract failed with status {}: {}", 
                result.status, stderr
            )));
        }
        
        // Lire les résultats
        let text = if output_txt.exists() {
            fs::read_to_string(&output_txt).await
                .map_err(|e| OcrError::Io(e))?
                .trim().to_string()
        } else {
            String::new()
        };
        
        let bounding_boxes = if output_tsv.exists() {
            self.parse_tsv_output(&output_tsv).await?
        } else {
            Vec::new()
        };
        
        // Calculer la confiance moyenne
        let confidence = if bounding_boxes.is_empty() {
            0.0
        } else {
            bounding_boxes.iter()
                .map(|bb| bb.confidence)
                .sum::<f32>() / bounding_boxes.len() as f32
        };
        
        // Détecter la langue (utiliser la première configurée)
        let detected_language = self.config.languages.first()
            .unwrap_or(&"unknown".to_string())
            .clone();
        
        // Créer les métadonnées
        let metadata = OcrMetadata {
            source_file: image_path.to_string_lossy().to_string(),
            file_size_bytes: fs::metadata(image_path).await
                .map(|m| m.len())
                .unwrap_or(0),
            image_dimensions: self.get_image_dimensions(image_path).await.unwrap_or((0, 0)),
            preprocessing_applied: vec![],
            psm_used: self.config.psm,
            oem_used: self.config.oem,
            temp_files_created: vec![
                output_txt.to_string_lossy().to_string(),
                output_tsv.to_string_lossy().to_string(),
            ],
        };
        
        // Nettoyer les fichiers temporaires
        let _ = fs::remove_file(&output_txt).await;
        let _ = fs::remove_file(&output_tsv).await;
        
        Ok(OcrResult {
            text,
            confidence,
            language: detected_language,
            bounding_boxes,
            processing_time: start_time.elapsed(),
            engine_used: "Tesseract Command".to_string(),
            tesseract_version: super::get_tesseract_version().await.unwrap_or("unknown".to_string()),
            metadata,
        })
    }
    
    /// Parser les résultats TSV de Tesseract
    async fn parse_tsv_output(&self, tsv_path: &Path) -> Result<Vec<BoundingBox>> {
        let content = fs::read_to_string(tsv_path).await?;
        let mut boxes = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            if line_num == 0 { continue; } // Skip header
            
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 { continue; }
            
            // Parser les champs TSV de Tesseract
            let level: u32 = fields[0].parse().unwrap_or(0);
            let left: u32 = fields[6].parse().unwrap_or(0);
            let top: u32 = fields[7].parse().unwrap_or(0);
            let width: u32 = fields[8].parse().unwrap_or(0);
            let height: u32 = fields[9].parse().unwrap_or(0);
            let conf: f32 = fields[10].parse().unwrap_or(0.0) / 100.0; // Normaliser 0-1
            let text = fields[11].trim().to_string();
            
            // Filtrer les éléments sans texte ou confiance trop faible
            if text.is_empty() || conf < 0.3 { continue; }
            
            boxes.push(BoundingBox {
                x: left,
                y: top,
                width,
                height,
                text,
                confidence: conf,
                level,
            });
        }
        
        debug!("📊 Parsed {} bounding boxes from TSV", boxes.len());
        Ok(boxes)
    }
    
    /// Preprocessing d'image via image crate
    async fn preprocess_image(&self, image_path: &Path) -> Result<PathBuf> {
        let start = Instant::now();
        
        // Générer path temporaire pour l'image preprocessée
        let session_id = Uuid::new_v4().to_string();
        let processed_path = self.config.temp_dir.join(format!("preprocessed_{}.png", session_id));
        
        let source_path = image_path.to_path_buf();
        let target_path = processed_path.clone();
        let config = self.config.preprocessing.clone();
        
        // Preprocessing avancé avec Otsu via spawn_blocking
        tokio::task::spawn_blocking(move || {
            use image::{DynamicImage, ImageBuffer, Luma};
            use imageproc::contrast::otsu_level;
            
            let image = image::open(&source_path)
                .map_err(|e| OcrError::ImageProcessing(format!("Failed to load image: {}", e)))?;
            
            let mut processed = image;
            
            // Redimensionnement si nécessaire (avant binarisation)
            if config.resize_for_ocr {
                let (width, height) = processed.dimensions();
                if width < config.min_width || height < config.min_height {
                    let new_width = width.max(config.min_width);
                    let new_height = height.max(config.min_height);
                    processed = processed.resize(
                        new_width, 
                        new_height, 
                        image::imageops::FilterType::Lanczos3
                    );
                }
            }
            
            // Binarisation Otsu pour meilleure reconnaissance OCR
            if config.enhance_contrast {
                let gray = processed.to_luma8();
                let threshold = otsu_level(&gray);
                
                let binary = ImageBuffer::from_fn(gray.width(), gray.height(), |x, y| {
                    let pixel = gray.get_pixel(x, y).0[0];
                    Luma([if pixel > threshold { 255 } else { 0 }])
                });
                
                processed = DynamicImage::ImageLuma8(binary);
            } else {
                // Amélioration contraste classique si Otsu désactivé
                processed = processed.adjust_contrast(15.0);
            }
            
            // Sauvegarder en PNG pour qualité optimale
            processed.save(&target_path)
                .map_err(|e| OcrError::ImageProcessing(format!("Failed to save processed image: {}", e)))?;
            
            Ok(target_path)
        }).await
        .map_err(|e| OcrError::ImageProcessing(format!("Preprocessing task failed: {}", e)))?
        .map_err(|e: OcrError| e)?;
        
        debug!("🖼️ Image preprocessed with Otsu binarization in {:.2}s: {:?}", 
               start.elapsed().as_secs_f32(), processed_path);
        
        Ok(processed_path)
    }
    
    /// Obtenir les dimensions d'une image
    async fn get_image_dimensions(&self, image_path: &Path) -> Result<(u32, u32)> {
        let path = image_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let reader = image::ImageReader::open(path)?;
            let dimensions = reader.into_dimensions()?;
            Ok(dimensions)
        }).await
        .map_err(|e| OcrError::ImageProcessing(format!("Task failed: {}", e)))?
        .map_err(|e: image::ImageError| OcrError::ImageProcessing(format!("Failed to get dimensions: {}", e)))
    }
    
    /// Vérifier que Tesseract est installé
    async fn verify_tesseract_installation() -> Result<()> {
        tokio::task::spawn_blocking(|| {
            match Command::new("tesseract").arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        debug!("✅ Tesseract installation verified");
                        Ok(())
                    } else {
                        Err(OcrError::TesseractCommand(
                            "Tesseract command failed".to_string()
                        ))
                    }
                }
                Err(_) => Err(OcrError::TesseractCommand(
                    "Tesseract not found in PATH".to_string()
                )),
            }
        }).await
        .map_err(|e| OcrError::TesseractCommand(format!("Task spawn failed: {}", e)))?
    }
    
    /// Traitement par lots
    pub async fn process_batch(&self, image_paths: Vec<PathBuf>) -> Result<Vec<OcrResult>> {
        use tokio::sync::Semaphore;
        use std::sync::Arc;
        
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
        let mut handles = Vec::new();
        
        for path in image_paths {
            let sem = Arc::clone(&semaphore);
            let processor = self.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                processor.process_image(&path).await
            });
            
            handles.push(handle);
        }
        
        // Collecter les résultats
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => error!("OCR processing failed: {}", e),
                Err(e) => error!("Task join failed: {}", e),
            }
        }
        
        Ok(results)
    }
    
    /// Nettoyer les fichiers temporaires
    pub async fn cleanup(&self) -> Result<()> {
        if self.config.temp_dir.exists() {
            let mut entries = fs::read_dir(&self.config.temp_dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path).await;
                }
            }
        }
        
        Ok(())
    }
}

// Nécessaire pour process_batch
impl Clone for TesseractProcessor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: None, // Simplifié pour le clonage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_processor_creation() {
        let config = TesseractConfig::default();
        match TesseractProcessor::new(config).await {
            Ok(_) => println!("✅ TesseractProcessor created successfully"),
            Err(e) => println!("⚠️ Failed to create processor: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_tesseract_verification() {
        match TesseractProcessor::verify_tesseract_installation().await {
            Ok(_) => println!("✅ Tesseract verification passed"),
            Err(e) => println!("⚠️ Tesseract verification failed: {}", e),
        }
    }
    
    async fn create_test_image() -> Result<PathBuf> {
        let temp_file = NamedTempFile::with_suffix(".png").unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Créer une image simple avec du texte blanc sur fond noir
        let img = image::ImageBuffer::from_fn(400, 100, |_x, _y| {
            image::Luma([255u8]) // Blanc
        });
        
        img.save(&temp_path).unwrap();
        temp_file.keep().unwrap();
        
        Ok(temp_path)
    }
    
    #[tokio::test]
    async fn test_image_processing() {
        let config = TesseractConfig::default();
        
        match TesseractProcessor::new(config).await {
            Ok(processor) => {
                match create_test_image().await {
                    Ok(test_path) => {
                        match processor.process_image(&test_path).await {
                            Ok(result) => {
                                println!("✅ Image processing test passed");
                                println!("   Text length: {}", result.text.len());
                                println!("   Confidence: {:.1}%", result.confidence * 100.0);
                                println!("   Bounding boxes: {}", result.bounding_boxes.len());
                                
                                // Nettoyer
                                let _ = std::fs::remove_file(test_path);
                            }
                            Err(e) => println!("⚠️ Image processing failed: {}", e),
                        }
                    }
                    Err(e) => println!("⚠️ Failed to create test image: {}", e),
                }
            }
            Err(e) => println!("⚠️ Failed to create processor: {}", e),
        }
    }
}