// GRAVIS OCR - Pipeline PDF avec poppler-utils via Command
// Utilise les outils poppler en ligne de commande (pdftotext, pdftoppm)

use super::{OcrError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{info, warn};

/// Configuration pour poppler-utils
#[derive(Debug, Clone)]
pub struct PopplerUtilsConfig {
    /// Chemin vers pdftotext (optionnel, utilise PATH par défaut)
    pub pdftotext_path: Option<PathBuf>,
    /// Chemin vers pdftoppm (optionnel, utilise PATH par défaut)  
    pub pdftoppm_path: Option<PathBuf>,
    /// DPI pour la conversion en image
    pub image_dpi: u16,
    /// Format d'image de sortie
    pub image_format: ImageFormat,
    /// Timeout pour les commandes
    pub timeout: Duration,
    /// Répertoire temporaire
    pub temp_dir: PathBuf,
}

impl Default for PopplerUtilsConfig {
    fn default() -> Self {
        Self {
            pdftotext_path: None,
            pdftoppm_path: None,
            image_dpi: 300,
            image_format: ImageFormat::Png,
            timeout: Duration::from_secs(30),
            temp_dir: std::env::temp_dir().join("gravis_poppler"),
        }
    }
}

/// Formats d'image supportés par pdftoppm
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    Ppm,
}

impl ImageFormat {
    fn as_flag(&self) -> &'static str {
        match self {
            ImageFormat::Png => "-png",
            ImageFormat::Jpeg => "-jpeg", 
            ImageFormat::Tiff => "-tiff",
            ImageFormat::Ppm => "", // Format par défaut
        }
    }
    
    fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff", 
            ImageFormat::Ppm => "ppm",
        }
    }
}

/// Résultat d'extraction avec poppler
#[derive(Debug, Clone)]
pub struct PopplerExtractionResult {
    pub text: String,
    pub page_count: usize,
    pub processing_time: Duration,
    pub method_used: ExtractionMethod,
}

/// Résultat de conversion en images
#[derive(Debug, Clone)]
pub struct PopplerImageResult {
    pub image_paths: Vec<PathBuf>,
    pub page_count: usize,
    pub processing_time: Duration,
    pub dpi: u16,
}

#[derive(Debug, Clone)]
pub enum ExtractionMethod {
    PdfToText,
    PdfToPpmThenOcr,
}

/// Processeur utilisant poppler-utils
pub struct PopplerUtilsProcessor {
    config: PopplerUtilsConfig,
}

impl PopplerUtilsProcessor {
    /// Créer un nouveau processeur poppler-utils
    pub async fn new(config: PopplerUtilsConfig) -> Result<Self> {
        info!("🚀 Initializing Poppler Utils Processor");
        
        // Vérifier que les outils sont disponibles
        Self::check_poppler_availability(&config).await?;
        
        // Créer le répertoire temporaire
        if !config.temp_dir.exists() {
            fs::create_dir_all(&config.temp_dir).await?;
        }
        
        Ok(Self { config })
    }
    
    /// Vérifier la disponibilité des outils poppler
    async fn check_poppler_availability(config: &PopplerUtilsConfig) -> Result<()> {
        // Vérifier pdftotext
        let pdftotext_cmd = config.pdftotext_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "pdftotext".to_string());
            
        let result = tokio::task::spawn_blocking(move || {
            Command::new(&pdftotext_cmd)
                .arg("-v")
                .output()
        }).await.map_err(|e| OcrError::TesseractCommand(format!("Failed to spawn task: {}", e)))?;
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    info!("✅ pdftotext available");
                } else {
                    warn!("⚠️ pdftotext version check failed, but command exists");
                }
            }
            Err(_) => {
                return Err(OcrError::TesseractCommand(
                    "pdftotext not found. Install with: brew install poppler".to_string()
                ));
            }
        }
        
        // Vérifier pdftoppm
        let pdftoppm_cmd = config.pdftoppm_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "pdftoppm".to_string());
            
        let result = tokio::task::spawn_blocking(move || {
            Command::new(&pdftoppm_cmd)
                .arg("-h")
                .output()
        }).await.map_err(|e| OcrError::TesseractCommand(format!("Failed to spawn task: {}", e)))?;
        
        match result {
            Ok(_) => info!("✅ pdftoppm available"),
            Err(_) => {
                warn!("⚠️ pdftoppm not found, image conversion will be disabled");
            }
        }
        
        Ok(())
    }
    
    /// Extraire le texte avec pdftotext
    pub async fn extract_text(&self, pdf_path: &Path) -> Result<PopplerExtractionResult> {
        let start_time = Instant::now();
        info!("📄 Extracting text with pdftotext: {:?}", pdf_path);
        
        let pdftotext_cmd = self.config.pdftotext_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "pdftotext".to_string());
        
        // Créer un fichier temporaire pour le texte de sortie
        let output_file = self.config.temp_dir.join(format!(
            "extracted_text_{}.txt", 
            chrono::Utc::now().timestamp_millis()
        ));
        
        let pdf_path_str = pdf_path.to_string_lossy().to_string();
        let output_path_str = output_file.to_string_lossy().to_string();
        
        // Exécuter pdftotext
        let result = tokio::time::timeout(
            self.config.timeout,
            tokio::task::spawn_blocking(move || {
                Command::new(&pdftotext_cmd)
                    .arg("-layout")  // Préserver la mise en page
                    .arg("-nopgbrk") // Pas de sauts de page
                    .arg(&pdf_path_str)
                    .arg(&output_path_str)
                    .output()
            })
        ).await;
        
        let output = match result {
            Ok(Ok(Ok(output))) => output,
            Ok(Ok(Err(e))) => {
                return Err(OcrError::TesseractCommand(format!("pdftotext failed: {}", e)));
            }
            Ok(Err(e)) => {
                return Err(OcrError::TesseractCommand(format!("Task failed: {}", e)));
            }
            Err(_) => {
                return Err(OcrError::Timeout);
            }
        };
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::TesseractCommand(format!("pdftotext error: {}", error_msg)));
        }
        
        // Lire le texte extrait
        let text = fs::read_to_string(&output_file).await?;
        
        // Nettoyer le fichier temporaire
        let _ = fs::remove_file(&output_file).await;
        
        // Compter les pages (estimation basée sur le contenu)
        let page_count = self.estimate_page_count(&text);
        
        let processing_time = start_time.elapsed();
        info!("✅ Text extraction completed in {:.2}s: {} chars, ~{} pages", 
              processing_time.as_secs_f32(), text.len(), page_count);
        
        Ok(PopplerExtractionResult {
            text,
            page_count,
            processing_time,
            method_used: ExtractionMethod::PdfToText,
        })
    }
    
    /// Convertir PDF en images avec pdftoppm
    pub async fn convert_to_images(&self, pdf_path: &Path, page_range: Option<(u32, u32)>) -> Result<PopplerImageResult> {
        let start_time = Instant::now();
        info!("🖼️ Converting PDF to images with pdftoppm: {:?}", pdf_path);
        
        let pdftoppm_cmd = self.config.pdftoppm_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "pdftoppm".to_string());
        
        // Préparer le préfixe de sortie
        let output_prefix = self.config.temp_dir.join(format!(
            "page_{}",
            chrono::Utc::now().timestamp_millis()
        ));
        let output_prefix_str = output_prefix.to_string_lossy().to_string();
        
        let pdf_path_str = pdf_path.to_string_lossy().to_string();
        
        // Construire la commande
        let result = tokio::time::timeout(
            self.config.timeout,
            tokio::task::spawn_blocking({
                let format_flag = self.config.image_format.as_flag().to_string();
                let dpi = self.config.image_dpi.to_string();
                move || {
                    let mut cmd = Command::new(&pdftoppm_cmd);
                    cmd.arg("-r").arg(&dpi); // DPI
                    
                    if !format_flag.is_empty() {
                        cmd.arg(&format_flag); // Format d'image
                    }
                    
                    // Range de pages si spécifié
                    if let Some((first, last)) = page_range {
                        cmd.arg("-f").arg(first.to_string());
                        cmd.arg("-l").arg(last.to_string());
                    }
                    
                    cmd.arg(&pdf_path_str)
                        .arg(&output_prefix_str)
                        .output()
                }
            })
        ).await;
        
        let output = match result {
            Ok(Ok(Ok(output))) => output,
            Ok(Ok(Err(e))) => {
                return Err(OcrError::ImageProcessing(format!("pdftoppm failed: {}", e)));
            }
            Ok(Err(e)) => {
                return Err(OcrError::TesseractCommand(format!("Task failed: {}", e)));
            }
            Err(_) => {
                return Err(OcrError::Timeout);
            }
        };
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::ImageProcessing(format!("pdftoppm error: {}", error_msg)));
        }
        
        // Trouver les fichiers générés
        let image_paths = self.find_generated_images(&output_prefix).await?;
        let page_count = image_paths.len();
        
        let processing_time = start_time.elapsed();
        info!("✅ Image conversion completed in {:.2}s: {} images generated", 
              processing_time.as_secs_f32(), page_count);
        
        Ok(PopplerImageResult {
            image_paths,
            page_count,
            processing_time,
            dpi: self.config.image_dpi,
        })
    }
    
    /// Trouver les images générées par pdftoppm
    async fn find_generated_images(&self, prefix: &Path) -> Result<Vec<PathBuf>> {
        let mut images = Vec::new();
        let extension = self.config.image_format.extension();
        let prefix_str = prefix.to_string_lossy();
        
        // pdftoppm génère des fichiers avec le pattern: prefix-001.ext, prefix-002.ext, etc.
        for i in 1..=1000 { // Limite raisonnable
            let image_path = PathBuf::from(format!("{}-{:03}.{}", prefix_str, i, extension));
            if image_path.exists() {
                images.push(image_path);
            } else {
                break; // Arrêter dès qu'on ne trouve plus de fichier
            }
        }
        
        Ok(images)
    }
    
    /// Estimer le nombre de pages basé sur le contenu textuel
    fn estimate_page_count(&self, text: &str) -> usize {
        // Estimation basique - 1 page ≈ 3000 caractères en moyenne
        (text.len() / 3000).max(1)
    }
    
    /// Nettoyer les fichiers temporaires
    pub async fn cleanup_temp_files(&self) -> Result<()> {
        if self.config.temp_dir.exists() {
            let mut entries = fs::read_dir(&self.config.temp_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let _ = fs::remove_file(entry.path()).await;
            }
        }
        Ok(())
    }
}

/// Fonction utilitaire pour extraction rapide avec poppler
pub async fn quick_poppler_extract(pdf_path: &Path) -> Result<String> {
    let config = PopplerUtilsConfig::default();
    let processor = PopplerUtilsProcessor::new(config).await?;
    let result = processor.extract_text(pdf_path).await?;
    Ok(result.text)
}

/// Installation automatique de poppler sur macOS
pub async fn install_poppler_macos() -> Result<()> {
    info!("🍺 Installing poppler via Homebrew...");
    
    let output = tokio::task::spawn_blocking(|| {
        Command::new("brew")
            .arg("install")
            .arg("poppler")
            .output()
    }).await.map_err(|e| OcrError::TesseractCommand(format!("Failed to spawn brew: {}", e)))?;
    
    match output {
        Ok(result) => {
            if result.status.success() {
                info!("✅ Poppler installed successfully");
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&result.stderr);
                Err(OcrError::TesseractCommand(format!("Brew install failed: {}", error)))
            }
        }
        Err(e) => Err(OcrError::TesseractCommand(format!("Brew command failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_poppler_availability() {
        let config = PopplerUtilsConfig::default();
        match PopplerUtilsProcessor::check_poppler_availability(&config).await {
            Ok(_) => println!("✅ Poppler tools available"),
            Err(e) => println!("⚠️ Poppler check failed: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_poppler_text_extraction() {
        let test_pdf = PathBuf::from("test.pdf");
        if test_pdf.exists() {
            match quick_poppler_extract(&test_pdf).await {
                Ok(text) => {
                    println!("✅ Poppler extraction successful: {} chars", text.len());
                    if !text.is_empty() {
                        println!("Preview: {}...", &text[..text.len().min(100)]);
                    }
                }
                Err(e) => println!("⚠️ Poppler extraction failed: {}", e),
            }
        } else {
            println!("📝 Test PDF not found, skipping test");
        }
    }
}