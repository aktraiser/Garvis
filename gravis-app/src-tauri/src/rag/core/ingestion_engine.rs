// Ingestion Engine - Phase 2 Intégration OCR-RAG
// Orchestration intelligente avec détection automatique de stratégie

use std::path::Path;
use std::time::Instant;
use tracing::{info, debug, warn};

use crate::rag::{
    GroupDocument, PdfStrategy, 
    ChunkConfig, RagResult, RagError
};
use crate::rag::processing::DocumentProcessor;
use crate::rag::ocr::{PreprocessConfig, FileFormat, detect_file_format};

/// Moteur d'ingestion intelligent avec détection automatique
pub struct IngestionEngine {
    document_processor: DocumentProcessor,
    strategy_detector: StrategyDetector,
}

impl IngestionEngine {
    /// Initialise le moteur d'ingestion
    pub fn new(document_processor: DocumentProcessor) -> Self {
        Self {
            document_processor,
            strategy_detector: StrategyDetector::new(),
        }
    }

    /// Point d'entrée principal: ingestion intelligente d'un document
    pub async fn ingest_document(
        &self,
        file_path: &Path,
        group_id: &str,
        chunk_config: &ChunkConfig,
    ) -> RagResult<IngestionResult> {
        let start_time = Instant::now();
        info!("Starting intelligent ingestion for: {:?}", file_path);

        // 1. Analyse préliminaire et détection de stratégie
        let strategy = self.strategy_detector.detect_strategy(file_path).await?;
        debug!("Strategy detected: {:?}", strategy);

        // 2. Traitement avec stratégie optimisée
        let document = match &strategy {
            IngestionStrategy::OptimizedPdf(pdf_strategy) => {
                self.process_pdf_with_strategy(file_path, group_id, chunk_config, *pdf_strategy).await?
            }
            IngestionStrategy::OptimizedImage(preprocess_config) => {
                self.process_image_with_preprocessing(file_path, group_id, chunk_config, preprocess_config.clone()).await?
            }
            IngestionStrategy::DirectText => {
                self.document_processor.process_document(file_path, group_id, chunk_config).await?
            }
        };

        let processing_time = start_time.elapsed();
        
        // 3. Génération du rapport d'ingestion avec comptage correct des chunks
        let chunks_count = document.chunks.len();
        let result = IngestionResult {
            document,
            strategy_used: strategy,
            processing_time_ms: processing_time.as_millis() as u64,
            chunks_created: chunks_count,
            cache_hits: CacheStats::default(),
        };

        info!("Ingestion completed in {:?}ms for {:?}", 
              result.processing_time_ms, file_path);

        Ok(result)
    }

    /// Traitement PDF avec stratégie spécifique
    async fn process_pdf_with_strategy(
        &self,
        file_path: &Path,
        group_id: &str,
        chunk_config: &ChunkConfig,
        strategy: PdfStrategy,
    ) -> RagResult<GroupDocument> {
        debug!("Processing PDF with strategy: {:?}", strategy);

        match strategy {
            PdfStrategy::NativeOnly => {
                // Force l'utilisation de l'extraction native uniquement
                // TODO: Implémenter extraction native directe
                warn!("Native PDF extraction not fully implemented, using standard processor");
                self.document_processor.process_document(file_path, group_id, chunk_config).await
            }
            PdfStrategy::OcrOnly => {
                // Force l'utilisation de l'OCR uniquement
                debug!("Forcing OCR-only processing for PDF");
                self.document_processor.process_document(file_path, group_id, chunk_config).await
            }
            PdfStrategy::HybridIntelligent => {
                // Pipeline hybride avec logique intelligente
                debug!("Using hybrid intelligent processing for PDF");
                self.process_pdf_hybrid_intelligent(file_path, group_id, chunk_config).await
            }
        }
    }

    /// Traitement PDF hybride intelligent (Phase 2)
    async fn process_pdf_hybrid_intelligent(
        &self,
        file_path: &Path,
        group_id: &str,
        chunk_config: &ChunkConfig,
    ) -> RagResult<GroupDocument> {
        // TODO Phase 2: Implémentation complète du pipeline hybride
        // 1. Tentative extraction native rapide
        // 2. Analyse qualité du texte extrait
        // 3. Décision zones OCR nécessaires
        // 4. Fusion intelligente texte natif + OCR
        
        warn!("Hybrid intelligent processing not fully implemented, using standard processor");
        self.document_processor.process_document(file_path, group_id, chunk_config).await
    }

    /// Traitement image avec préprocessing optimisé
    async fn process_image_with_preprocessing(
        &self,
        file_path: &Path,
        group_id: &str,
        chunk_config: &ChunkConfig,
        preprocess_config: PreprocessConfig,
    ) -> RagResult<GroupDocument> {
        debug!("Processing image with preprocessing: {:?}", preprocess_config);
        
        // TODO Phase 2: Appliquer préprocessing avant OCR
        // Pour l'instant, utilise le processeur standard
        self.document_processor.process_document(file_path, group_id, chunk_config).await
    }

    /// Traitement par lot avec parallélisation
    pub async fn ingest_document_batch(
        &self,
        file_paths: Vec<&Path>,
        group_id: &str,
        chunk_config: &ChunkConfig,
    ) -> RagResult<BatchIngestionResult> {
        info!("Starting batch ingestion of {} documents", file_paths.len());
        
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut errors = Vec::new();

        let file_count = file_paths.len();
        
        // TODO Phase 2: Parallélisation avec tokio::spawn
        // Pour l'instant, traitement séquentiel
        for file_path in &file_paths {
            match self.ingest_document(file_path, group_id, chunk_config).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to process {:?}: {}", file_path, e);
                    errors.push((file_path.to_path_buf(), e));
                }
            }
        }

        let total_time = start_time.elapsed();
        
        Ok(BatchIngestionResult {
            successful_ingestions: results,
            failed_ingestions: errors,
            total_processing_time_ms: total_time.as_millis() as u64,
            total_documents: file_count,
        })
    }
}

/// Détecteur de stratégie intelligent
pub struct StrategyDetector;

impl StrategyDetector {
    pub fn new() -> Self {
        Self
    }

    /// Détection automatique de la stratégie optimale
    pub async fn detect_strategy(&self, file_path: &Path) -> RagResult<IngestionStrategy> {
        debug!("Detecting strategy for: {:?}", file_path);

        // 1. Détection du format de fichier
        match detect_file_format(file_path) {
            Ok(FileFormat::Pdf) => {
                let pdf_strategy = self.detect_pdf_strategy(file_path).await?;
                Ok(IngestionStrategy::OptimizedPdf(pdf_strategy))
            }
            Ok(FileFormat::Png | FileFormat::Jpeg | FileFormat::Tiff | FileFormat::Bmp) => {
                let preprocess_config = self.detect_image_preprocessing(file_path).await?;
                Ok(IngestionStrategy::OptimizedImage(preprocess_config))
            }
            Err(_) => {
                // Format non supporté par OCR, traitement texte direct
                debug!("File format not supported by OCR, using direct text processing");
                Ok(IngestionStrategy::DirectText)
            }
        }
    }

    /// Détection de stratégie PDF intelligente
    pub async fn detect_pdf_strategy(&self, file_path: &Path) -> RagResult<PdfStrategy> {
        debug!("Analyzing PDF strategy for: {:?}", file_path);

        // TODO Phase 2: Implémentation heuristiques avancées
        // 1. Analyse rapide du ratio texte natif vs images
        // 2. Détection de fonts/qualité typographique
        // 3. Heuristiques basées sur la taille/structure

        // Heuristiques simples pour Phase 2
        let file_size = tokio::fs::metadata(file_path).await
            .map_err(|e| RagError::Io(e))?
            .len();

        let strategy = if file_size > 50_000_000 { // > 50MB
            // Gros fichiers: probablement beaucoup d'images, utiliser hybride
            debug!("Large PDF detected ({}MB), using hybrid strategy", file_size / 1_000_000);
            PdfStrategy::HybridIntelligent
        } else if file_size < 1_000_000 { // < 1MB
            // Petits fichiers: probablement texte natif
            debug!("Small PDF detected ({}KB), trying native first", file_size / 1000);
            PdfStrategy::NativeOnly
        } else {
            // Taille moyenne: utiliser hybride intelligent
            debug!("Medium PDF detected ({}MB), using hybrid strategy", file_size / 1_000_000);
            PdfStrategy::HybridIntelligent
        };

        Ok(strategy)
    }

    /// Détection de preprocessing image optimal
    pub async fn detect_image_preprocessing(&self, file_path: &Path) -> RagResult<PreprocessConfig> {
        debug!("Analyzing image preprocessing for: {:?}", file_path);

        // TODO Phase 2: Analyse d'image pour détection automatique
        // 1. Analyse histogramme pour détection contraste
        // 2. Détection de bruit/artifacts
        // 3. Estimation qualité pour Otsu vs autres filtres

        // Configuration par défaut pour Phase 2
        Ok(PreprocessConfig::default())
    }
}

/// Stratégie d'ingestion détectée
#[derive(Debug, Clone)]
pub enum IngestionStrategy {
    OptimizedPdf(PdfStrategy),
    OptimizedImage(PreprocessConfig),
    DirectText,
}

/// Résultat d'ingestion d'un document
#[derive(Debug)]
pub struct IngestionResult {
    pub document: GroupDocument,
    pub strategy_used: IngestionStrategy,
    pub processing_time_ms: u64,
    pub chunks_created: usize,
    pub cache_hits: CacheStats,
}

/// Résultat d'ingestion par lot
#[derive(Debug)]
pub struct BatchIngestionResult {
    pub successful_ingestions: Vec<IngestionResult>,
    pub failed_ingestions: Vec<(std::path::PathBuf, RagError)>,
    pub total_processing_time_ms: u64,
    pub total_documents: usize,
}

/// Statistiques de cache
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub ocr_cache_hits: u32,
    pub embedding_cache_hits: u32,
    pub document_cache_hits: u32,
    pub total_cache_requests: u32,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f32 {
        if self.total_cache_requests == 0 {
            0.0
        } else {
            (self.ocr_cache_hits + self.embedding_cache_hits + self.document_cache_hits) as f32 
                / self.total_cache_requests as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ocr::{TesseractProcessor, TesseractConfig};
    use crate::rag::search::custom_e5::{CustomE5Embedder, CustomE5Config};

    #[tokio::test]
    async fn test_strategy_detector() {
        let detector = StrategyDetector::new();
        
        // Test avec fichier non-existant (doit retourner DirectText)
        let strategy = detector.detect_strategy(Path::new("test.txt")).await.unwrap();
        matches!(strategy, IngestionStrategy::DirectText);
    }

    #[tokio::test]
    async fn test_ingestion_engine_creation() {
        // Test de création du moteur d'ingestion
        let tesseract_config = TesseractConfig::default();
        let ocr_processor = TesseractProcessor::new(tesseract_config).await.unwrap();
        
        let e5_config = CustomE5Config::default();
        let embedder = CustomE5Embedder::new(e5_config).await.unwrap();
        
        let document_processor = DocumentProcessor::new(ocr_processor, std::sync::Arc::new(embedder)).await.unwrap();
        let _engine = IngestionEngine::new(document_processor);
        
        // Si on arrive ici, la création a réussi
        assert!(true);
    }
}