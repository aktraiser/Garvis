// Document Processor Unifié - Phase 1 Intégration OCR-RAG
// Orchestration intelligente: détection format → extraction → chunking → embedding

use anyhow::Result;
use std::path::Path;
use std::time::SystemTime;
use tracing::{info, warn, debug};

use crate::rag::{
    GroupDocument, DocumentType, PdfStrategy, EnrichedChunk, ChunkType, ChunkMetadata,
    SourceType, ExtractionMethod, Priority, ChunkConfig, RagResult, RagError,
    sanitize_pdf_text
};
use crate::rag::ocr::{
    TesseractProcessor, OcrMetadata, PreprocessConfig, 
    detect_file_format, FileFormat,
    pdf_extract_simple::{SimplePdfExtractor, PdfExtractConfig}
};
use crate::rag::search::custom_e5::CustomE5Embedder;
use std::sync::Arc;

/// Processeur de documents unifié avec intelligence OCR
pub struct DocumentProcessor {
    ocr_processor: TesseractProcessor,
    #[allow(dead_code)]
    embedder: Arc<CustomE5Embedder>,
}

impl DocumentProcessor {
    /// Initialise le processeur avec les composants nécessaires
    pub async fn new(
        ocr_processor: TesseractProcessor,
        embedder: Arc<CustomE5Embedder>,
    ) -> Result<Self> {
        Ok(Self {
            ocr_processor,
            embedder,
        })
    }

    /// Point d'entrée principal: traite n'importe quel document
    pub async fn process_document(
        &self,
        file_path: &Path,
        group_id: &str,
        chunk_config: &ChunkConfig,
    ) -> RagResult<GroupDocument> {
        info!("Processing document: {:?}", file_path);

        // 1. Détection automatique du format
        let (content, document_type, extraction_method) = match detect_file_format(file_path) {
            Ok(FileFormat::Pdf) => self.process_pdf(file_path).await?,
            Ok(FileFormat::Png | FileFormat::Jpeg | FileFormat::Tiff | FileFormat::Bmp) => {
                self.process_image(file_path).await?
            }
            Err(_) => {
                // Format non supporté par OCR, traiter comme texte
                debug!("Format not supported by OCR, treating as text: {:?}", file_path);
                self.process_text(file_path).await?
            }
        };

        // 3. Normalisation Unicode optimisée avec cache et debug conditionnel
        let (normalized_content, norm_stats) = sanitize_pdf_text(&content)
            .map_err(|e| RagError::InvalidConfig(format!("Unicode normalization failed: {}", e)))?;
        
        // Log unique par document sous flag debug
        if tracing::enabled!(tracing::Level::DEBUG) && norm_stats.ligatures_replaced > 0 {
            tracing::debug!(
                file_path = ?file_path,
                fi = norm_stats.lig_fi,
                fl = norm_stats.lig_fl,
                total_ligatures = norm_stats.ligatures_replaced,
                chars_before = norm_stats.total_chars_before,
                chars_after = norm_stats.total_chars_after,
                "Unicode ligatures normalized for document"
            );
        }
        
        // 4. Chunking adaptatif selon le type de source
        let source_type = self.determine_source_type(&extraction_method, &document_type);
        let mut chunks = self.chunk_by_content_type(
            &normalized_content,
            source_type.clone(),
            extraction_method.clone(),
            chunk_config,
            group_id,
        ).await?;
        
        // GARDE-FOU: Si aucun chunk créé, créer un chunk avec tout le contenu
        if chunks.is_empty() && !normalized_content.trim().is_empty() {
            tracing::warn!(
                content_len = normalized_content.len(),
                chunk_size = chunk_config.chunk_size,
                "Chunker returned 0 chunks, creating fallback whole-document chunk"
            );
            
            // Créer un chunk de fallback avec tout le contenu
            let fallback_chunk = EnrichedChunk {
                id: format!("chunk_{}_fallback", uuid::Uuid::new_v4().simple()),
                content: normalized_content.clone(),
                start_line: 0,
                end_line: normalized_content.lines().count(),
                chunk_type: ChunkType::TextBlock,
                embedding: None,
                hash: String::new(),
                metadata: ChunkMetadata {
                    tags: vec!["fallback".to_string()],
                    priority: Priority::Normal,
                    language: "auto".to_string(),
                    symbol: None,
                    context: None,
                    confidence: 0.8, // Confiance réduite pour chunk de fallback
                    ocr_metadata: None,
                    source_type: source_type.clone(),
                    extraction_method: extraction_method.clone(),
                },
                group_id: group_id.to_string(),
            };
            
            chunks.push(fallback_chunk);
            tracing::info!("Created fallback chunk with {} chars", normalized_content.len());
        }

        // FALLBACK SPLIT: Si seulement 1 chunk et trop long, essayer de split simple
        if chunks.len() == 1 && normalized_content.len() > chunk_config.chunk_size * 2 {
            tracing::info!(
                chunks_count = chunks.len(),
                content_len = normalized_content.len(),
                chunk_size = chunk_config.chunk_size,
                "Only 1 chunk detected but content is long, attempting simple split"
            );
            
            let original_chunk = chunks.pop().unwrap();
            let split_chunks = simple_text_split(&original_chunk.content, chunk_config);
            
            if split_chunks.len() > 1 {
                tracing::info!("Successfully split into {} chunks", split_chunks.len());
                chunks.extend(split_chunks);
            } else {
                // Si split a échoué, garder le chunk original
                chunks.push(original_chunk);
                tracing::debug!("Simple split failed, keeping original chunk");
            }
        }

        // GARDE-FOU FINAL: Assurer minimum 2-3 chunks pour documents longs
        if chunks.len() < 2 && normalized_content.len() > 3000 {
            tracing::warn!(
                chunks_count = chunks.len(),
                content_len = normalized_content.len(),
                "Document long avec trop peu de chunks, tentative fallback split agressif"
            );
            
            // Fallback split plus agressif par pages/paragraphes
            let fallback_chunks = fallback_split_by_pages_or_paragraphs(&normalized_content, 1500);
            if fallback_chunks.len() > chunks.len() {
                tracing::info!("Fallback split successful: {} → {} chunks", chunks.len(), fallback_chunks.len());
                chunks = fallback_chunks;
            }
        }
        
        // GARDE-FOU ULTIME: Si vraiment aucun chunk après tous les fallbacks, créer un chunk d'erreur
        if chunks.is_empty() {
            tracing::error!("E2E CRITICAL: expected >0 chunks after all fallbacks for {:?}", file_path);
            tracing::warn!("Creating emergency fallback chunk to prevent crash");
            
            // Créer un chunk d'urgence pour éviter la panique
            let emergency_chunk = EnrichedChunk {
                id: format!("chunk_{}_emergency", uuid::Uuid::new_v4().simple()),
                content: format!("EXTRACTION FAILED: No text could be extracted from {}", file_path.file_name().unwrap_or_default().to_string_lossy()),
                start_line: 0,
                end_line: 1,
                chunk_type: ChunkType::TextBlock,
                embedding: None,
                hash: String::new(),
                metadata: ChunkMetadata {
                    tags: vec!["extraction_failed".to_string(), "emergency".to_string()],
                    priority: Priority::Low,
                    language: "unknown".to_string(),
                    symbol: None,
                    context: None,
                    confidence: 0.0,
                    ocr_metadata: None,
                    source_type: SourceType::NativeText,
                    extraction_method: ExtractionMethod::DirectRead,
                },
                group_id: group_id.to_string(),
            };
            chunks.push(emergency_chunk);
        }

        // 5. Construction du document enrichi
        let document_id = format!("doc_{}", uuid::Uuid::new_v4().simple());
        let now = SystemTime::now();

        Ok(GroupDocument {
            id: document_id,
            file_path: file_path.to_path_buf(),
            language: "auto".to_string(), // TODO: détection langue
            content: normalized_content,
            chunks,
            metadata: crate::rag::EnrichedMetadata {
                tags: vec!["auto-imported".to_string()],
                priority: Priority::Normal,
                description: Some(format!("Processed via {:?}", extraction_method)),
                author: None,
                project: None,
                custom_fields: std::collections::HashMap::new(),
            },
            last_modified: now,
            document_type,
            group_id: group_id.to_string(),
        })
    }

    /// Traitement PDF avec stratégie intelligente
    async fn process_pdf(&self, path: &Path) -> RagResult<(String, DocumentType, ExtractionMethod)> {
        debug!("Processing PDF: {:?}", path);

        // Tentative d'extraction native d'abord
        match self.extract_pdf_native(path).await {
            Ok((content, native_ratio)) => {
                if native_ratio > 0.8 {
                    // Contenu natif de qualité, pas besoin d'OCR
                    let doc_type = DocumentType::PDF {
                        extraction_strategy: PdfStrategy::NativeOnly,
                        native_text_ratio: native_ratio,
                        ocr_pages: vec![],
                        total_pages: 1, // TODO: détecter pages réelles
                    };
                    Ok((content, doc_type, ExtractionMethod::PdfNative))
                } else {
                    // Qualité médiocre, basculer sur hybride
                    self.process_pdf_hybrid(path).await
                }
            }
            Err(_) => {
                // Échec extraction native, utiliser OCR
                warn!("Native PDF extraction failed for {:?}, using OCR", path);
                self.process_pdf_ocr_only(path).await
            }
        }
    }

    /// Extraction PDF native avec SimplePdfExtractor
    async fn extract_pdf_native(&self, path: &Path) -> Result<(String, f32)> {
        debug!("Attempting native PDF extraction for: {:?}", path);
        
        let config = PdfExtractConfig::default();
        let extractor = SimplePdfExtractor::new(config);
        let result = extractor.extract_pdf_text(path).await
            .map_err(|e| anyhow::anyhow!("PDF extraction failed: {}", e))?;
        
        // Si aucun texte extrait, retourner une erreur pour déclencher le fallback OCR
        if result.text.trim().is_empty() || result.token_count == 0 {
            warn!("PDF native extraction returned empty text, will trigger OCR fallback");
            return Err(anyhow::anyhow!("No native text extracted from PDF"));
        }

        // Estimer la qualité du texte extrait - amélioration pour détection native
        let quality_ratio = if result.text.len() > 200 {
            let _word_count = result.text.split_whitespace().count();
            let alpha_count = result.text.chars().filter(|c| c.is_alphabetic()).count();
            let printable_count = result.text.chars().filter(|c| c.is_ascii_graphic() || c.is_whitespace()).count();

            let alpha_ratio = alpha_count as f32 / result.text.len() as f32;
            let printable_ratio = printable_count as f32 / result.text.len() as f32;

            // Si ratio printable > 0.9 et beaucoup de texte → texte natif de qualité
            if printable_ratio > 0.9 && result.text.len() > 1000 {
                1.0  // Parfait pour texte natif
            } else {
                (alpha_ratio * 1.2).min(1.0)
            }
        } else {
            0.3 // Texte court mais présent
        };

        debug!("PDF native extraction: {} chars, quality={:.2}",
               result.text.len(), quality_ratio);

        Ok((result.text, quality_ratio))
    }

    /// Traitement PDF hybride intelligent
    async fn process_pdf_hybrid(&self, path: &Path) -> RagResult<(String, DocumentType, ExtractionMethod)> {
        debug!("Processing PDF with hybrid intelligent strategy: {:?}", path);
        
        // 1. Tentative extraction native d'abord
        match self.extract_pdf_native(path).await {
            Ok((content, quality)) if quality > 0.7 => {
                // Qualité suffisante, utiliser extraction native
                info!("Using native PDF extraction (quality={:.2})", quality);
                
                // Sanitization Unicode pour ligatures PDF
                let (sanitized_content, normalization_stats) = sanitize_pdf_text(&content)
                    .map_err(|e| RagError::InvalidConfig(format!("Unicode sanitization failed: {}", e)))?;
                
                if normalization_stats.ligatures_replaced > 0 {
                    info!("Sanitized PDF content: {} ligatures replaced", normalization_stats.ligatures_replaced);
                }
                
                let doc_type = DocumentType::PDF {
                    extraction_strategy: PdfStrategy::NativeOnly,
                    native_text_ratio: quality,
                    ocr_pages: vec![],
                    total_pages: 1, // TODO: compter pages réelles
                };
                
                Ok((sanitized_content, doc_type, ExtractionMethod::PdfNative))
            }
            Ok((content, quality)) => {
                // Qualité insuffisante, mais on a du contenu
                info!("Native PDF quality moderate ({:.2}), using as fallback", quality);
                
                // Sanitization Unicode même pour qualité modérée
                let (sanitized_content, normalization_stats) = sanitize_pdf_text(&content)
                    .map_err(|e| RagError::InvalidConfig(format!("Unicode sanitization failed: {}", e)))?;
                
                if normalization_stats.ligatures_replaced > 0 {
                    info!("Sanitized moderate quality PDF: {} ligatures replaced", normalization_stats.ligatures_replaced);
                }
                
                let doc_type = DocumentType::PDF {
                    extraction_strategy: PdfStrategy::HybridIntelligent,
                    native_text_ratio: quality,
                    ocr_pages: vec![],
                    total_pages: 1,
                };
                
                Ok((sanitized_content, doc_type, ExtractionMethod::HybridIntelligent))
            }
            Err(_) => {
                // Échec extraction native, utiliser OCR
                warn!("Native PDF extraction failed, using OCR fallback");
                self.process_pdf_ocr_only(path).await
            }
        }
    }

    /// Traitement PDF par OCR uniquement
    async fn process_pdf_ocr_only(&self, path: &Path) -> RagResult<(String, DocumentType, ExtractionMethod)> {
        // TODO: Implémenter process_pdf pour TesseractProcessor
        // Pour l'instant, traiter comme image simple
        warn!("PDF OCR not fully implemented yet, treating as single page");
        
        let ocr_result = self.ocr_processor.process_image(path).await
            .map_err(|e| RagError::InvalidConfig(format!("PDF OCR failed: {}", e)))?;

        // Sanitization Unicode critique pour contenu OCR (plus de ligatures)
        let (sanitized_content, normalization_stats) = sanitize_pdf_text(&ocr_result.text)
            .map_err(|e| RagError::InvalidConfig(format!("Unicode sanitization failed: {}", e)))?;
        
        if normalization_stats.ligatures_replaced > 0 {
            info!("Sanitized OCR content: {} ligatures replaced", normalization_stats.ligatures_replaced);
        }

        let confidence = ocr_result.confidence;

        let doc_type = DocumentType::PDF {
            extraction_strategy: PdfStrategy::OcrOnly,
            native_text_ratio: 0.0,
            ocr_pages: vec![0],
            total_pages: 1,
        };

        let extraction_method = ExtractionMethod::TesseractOcr {
            confidence,
            language: if ocr_result.language.is_empty() { "fra".to_string() } else { ocr_result.language.clone() },
        };

        Ok((sanitized_content, doc_type, extraction_method))
    }

    /// Traitement d'image par OCR
    async fn process_image(&self, path: &Path) -> RagResult<(String, DocumentType, ExtractionMethod)> {
        debug!("Processing image: {:?}", path);

        let ocr_result = self.ocr_processor.process_image(path).await
            .map_err(|e| RagError::InvalidConfig(format!("Image OCR failed: {}", e)))?;

        let content = ocr_result.text.clone();
        let confidence = ocr_result.confidence;

        let doc_type = DocumentType::Image {
            ocr_result: ocr_result.clone(),
            preprocessing_config: PreprocessConfig::default(), // TODO: récupérer config réelle
        };

        let extraction_method = ExtractionMethod::TesseractOcr {
            confidence,
            language: if ocr_result.language.is_empty() { "fra".to_string() } else { ocr_result.language.clone() },
        };

        Ok((content, doc_type, extraction_method))
    }

    /// Traitement de fichier texte simple
    async fn process_text(&self, path: &Path) -> RagResult<(String, DocumentType, ExtractionMethod)> {
        debug!("Processing text file: {:?}", path);

        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| RagError::Io(e))?;

        let doc_type = if path.extension().and_then(|s| s.to_str()) == Some("md") {
            DocumentType::Markdown
        } else {
            DocumentType::PlainText
        };

        Ok((content, doc_type, ExtractionMethod::DirectRead))
    }

    /// Détermine le type de source selon la méthode d'extraction
    fn determine_source_type(&self, method: &ExtractionMethod, _doc_type: &DocumentType) -> SourceType {
        match method {
            ExtractionMethod::DirectRead => SourceType::NativeText,
            ExtractionMethod::TesseractOcr { .. } => SourceType::OcrExtracted,
            ExtractionMethod::PdfNative => SourceType::HybridPdfNative,
            ExtractionMethod::PdfOcrFallback => SourceType::HybridPdfOcr,
            ExtractionMethod::HybridIntelligent => SourceType::HybridPdfOcr,
        }
    }

    /// Chunking adaptatif selon le type de contenu - Phase 2 amélioré
    async fn chunk_by_content_type(
        &self,
        content: &str,
        source_type: SourceType,
        extraction_method: ExtractionMethod,
        config: &ChunkConfig,
        group_id: &str,
    ) -> RagResult<Vec<EnrichedChunk>> {
        debug!("Chunking content: {} chars, source: {:?}", content.len(), source_type);

        match source_type {
            SourceType::OcrExtracted => {
                self.chunk_ocr_content(content, extraction_method, config, group_id).await
            }
            SourceType::HybridPdfOcr => {
                self.chunk_hybrid_content(content, extraction_method, config, group_id).await
            }
            SourceType::HybridPdfNative => {
                // Pour du texte natif de qualité, utiliser chunking natif optimisé
                self.chunk_native_content(content, extraction_method, config, group_id).await
            }
            SourceType::NativeText => {
                self.chunk_native_content(content, extraction_method, config, group_id).await
            }
        }
    }

    /// Chunking spécialisé pour contenu OCR - Phase 2
    async fn chunk_ocr_content(
        &self,
        content: &str,
        extraction_method: ExtractionMethod,
        config: &ChunkConfig,
        group_id: &str,
    ) -> RagResult<Vec<EnrichedChunk>> {
        debug!("OCR-specific chunking for {} chars", content.len());

        // Chunking OCR: préservation structure + confiance par chunk
        let chunk_size = config.chunk_size;
        let overlap = config.overlap;
        
        // Découpage par paragraphes d'abord (respecte structure OCR)
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for paragraph in paragraphs {
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            
            // Si le paragraphe est trop grand, le diviser
            if words.len() > chunk_size {
                // Finaliser le chunk actuel s'il existe
                if !current_chunk.trim().is_empty() {
                    let chunk = self.create_ocr_chunk(
                        &current_chunk, 
                        chunk_index, 
                        &extraction_method, 
                        group_id
                    )?;
                    chunks.push(chunk);
                    chunk_index += 1;
                    current_chunk.clear();
                }

                // Diviser le gros paragraphe
                for word_chunk in words.chunks(chunk_size) {
                    let chunk_text = word_chunk.join(" ");
                    let chunk = self.create_ocr_chunk(
                        &chunk_text, 
                        chunk_index, 
                        &extraction_method, 
                        group_id
                    )?;
                    chunks.push(chunk);
                    chunk_index += 1;
                }
            } else {
                // Ajouter au chunk actuel
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(paragraph);

                // Vérifier si le chunk est assez grand
                let current_words = current_chunk.split_whitespace().count();
                if current_words >= chunk_size {
                    let chunk = self.create_ocr_chunk(
                        &current_chunk, 
                        chunk_index, 
                        &extraction_method, 
                        group_id
                    )?;
                    chunks.push(chunk);
                    chunk_index += 1;
                    
                    // Overlap: garder les derniers mots
                    let words: Vec<&str> = current_chunk.split_whitespace().collect();
                    let overlap_start = std::cmp::max(0, words.len().saturating_sub(overlap));
                    current_chunk = words[overlap_start..].join(" ");
                }
            }
        }

        // Finaliser le dernier chunk
        if !current_chunk.trim().is_empty() {
            let chunk = self.create_ocr_chunk(
                &current_chunk, 
                chunk_index, 
                &extraction_method, 
                group_id
            )?;
            chunks.push(chunk);
        }

        info!("Created {} OCR chunks from {} chars", chunks.len(), content.len());
        Ok(chunks)
    }

    /// Chunking pour contenu hybride PDF - Phase 2
    async fn chunk_hybrid_content(
        &self,
        content: &str,
        extraction_method: ExtractionMethod,
        config: &ChunkConfig,
        group_id: &str,
    ) -> RagResult<Vec<EnrichedChunk>> {
        debug!("Hybrid PDF chunking for {} chars", content.len());

        // TODO Phase 2: Fusion intelligente texte natif + OCR
        // Pour l'instant, utilise le chunking OCR
        self.chunk_ocr_content(content, extraction_method, config, group_id).await
    }

    /// Chunking pour contenu texte natif - Phase 2 optimisé
    async fn chunk_native_content(
        &self,
        content: &str,
        extraction_method: ExtractionMethod,
        config: &ChunkConfig,
        group_id: &str,
    ) -> RagResult<Vec<EnrichedChunk>> {
        debug!("Native text chunking for {} chars", content.len());

        let chunk_size = config.chunk_size;
        let overlap = config.overlap;
        
        // Chunking par phrases pour texte natif (meilleure qualité)
        let sentences: Vec<&str> = content.split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for sentence in sentences {
            let words_in_sentence = sentence.split_whitespace().count();
            let current_words = current_chunk.split_whitespace().count();

            if current_words + words_in_sentence > chunk_size && !current_chunk.is_empty() {
                // Créer le chunk actuel
                let chunk = self.create_native_chunk(
                    &current_chunk, 
                    chunk_index, 
                    &extraction_method, 
                    group_id
                )?;
                chunks.push(chunk);
                chunk_index += 1;

                // Overlap: garder les derniers mots
                let words: Vec<&str> = current_chunk.split_whitespace().collect();
                let overlap_start = std::cmp::max(0, words.len().saturating_sub(overlap));
                current_chunk = words[overlap_start..].join(" ");
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str(". ");
            }
            current_chunk.push_str(sentence.trim());
        }

        // Finaliser le dernier chunk
        if !current_chunk.trim().is_empty() {
            let chunk = self.create_native_chunk(
                &current_chunk, 
                chunk_index, 
                &extraction_method, 
                group_id
            )?;
            chunks.push(chunk);
        }

        info!("Created {} native chunks from {} chars", chunks.len(), content.len());
        Ok(chunks)
    }

    /// Création d'un chunk OCR avec métadonnées spécialisées
    fn create_ocr_chunk(
        &self,
        content: &str,
        index: usize,
        extraction_method: &ExtractionMethod,
        group_id: &str,
    ) -> RagResult<EnrichedChunk> {
        let confidence = match extraction_method {
            ExtractionMethod::TesseractOcr { confidence, .. } => *confidence,
            _ => 0.8, // Confidence par défaut
        };

        let mut chunk = EnrichedChunk {
            id: format!("chunk_ocr_{}_{}", uuid::Uuid::new_v4().simple(), index),
            content: content.to_string(),
            start_line: index,
            end_line: index + 1,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: String::new(),
            metadata: ChunkMetadata {
                tags: vec!["ocr-extracted".to_string()],
                priority: if confidence > 0.8 { Priority::High } else { Priority::Normal },
                language: "fra".to_string(),
                symbol: None,
                context: Some("OCR extraction".to_string()),
                confidence,
                ocr_metadata: self.extract_ocr_metadata_for_chunk(extraction_method),
                source_type: SourceType::OcrExtracted,
                extraction_method: extraction_method.clone(),
            },
            group_id: group_id.to_string(),
        };

        chunk.generate_hash();
        Ok(chunk)
    }

    /// Création d'un chunk texte natif avec métadonnées optimisées
    fn create_native_chunk(
        &self,
        content: &str,
        index: usize,
        extraction_method: &ExtractionMethod,
        group_id: &str,
    ) -> RagResult<EnrichedChunk> {
        let mut chunk = EnrichedChunk {
            id: format!("chunk_native_{}_{}", uuid::Uuid::new_v4().simple(), index),
            content: content.to_string(),
            start_line: index,
            end_line: index + 1,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: String::new(),
            metadata: ChunkMetadata {
                tags: vec!["native-text".to_string()],
                priority: Priority::High, // Texte natif = haute qualité
                language: "fra".to_string(),
                symbol: None,
                context: Some("Native text extraction".to_string()),
                confidence: 1.0, // Confiance maximale pour texte natif
                ocr_metadata: None, // Pas de métadonnées OCR pour texte natif
                source_type: SourceType::NativeText,
                extraction_method: extraction_method.clone(),
            },
            group_id: group_id.to_string(),
        };

        chunk.generate_hash();
        Ok(chunk)
    }

    /// Extrait les métadonnées OCR pour un chunk
    fn extract_ocr_metadata_for_chunk(&self, method: &ExtractionMethod) -> Option<OcrMetadata> {
        match method {
            ExtractionMethod::TesseractOcr { confidence: _, language: _ } => {
                Some(OcrMetadata {
                    source_file: "".to_string(),
                    file_size_bytes: 0,
                    image_dimensions: (0, 0),
                    preprocessing_applied: vec![],
                    psm_used: crate::rag::ocr::PageSegMode::Auto,
                    oem_used: crate::rag::ocr::OcrEngineMode::Default,
                    temp_files_created: vec![],
                })
            }
            _ => None,
        }
    }
}

/// Fallback simple text splitting when standard chunker fails
fn simple_text_split(content: &str, chunk_config: &ChunkConfig) -> Vec<EnrichedChunk> {
    let mut chunks = Vec::new();
    let target_size = chunk_config.chunk_size;
    let overlap = chunk_config.overlap;
    
    // Si le contenu est trop petit, retourner un seul chunk
    if content.len() <= target_size {
        return chunks; // Retourner vide pour indiquer l'échec
    }
    
    // Split par paragraphes d'abord
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut current_chunk = String::new();
    let mut chunk_index = 0;
    
    for paragraph in paragraphs {
        // Si ajouter ce paragraphe dépasse la taille cible, finaliser le chunk actuel
        if !current_chunk.is_empty() && current_chunk.len() + paragraph.len() > target_size {
            // Créer chunk avec le contenu actuel
            let chunk = EnrichedChunk {
                id: format!("chunk_{}_split_{}", uuid::Uuid::new_v4().simple(), chunk_index),
                content: current_chunk.trim().to_string(),
                start_line: 0,
                end_line: 0,
                chunk_type: ChunkType::TextBlock,
                embedding: None,
                hash: blake3::hash(current_chunk.trim().as_bytes()).to_hex().to_string(),
                metadata: ChunkMetadata {
                    tags: vec!["fallback-split".to_string()],
                    priority: Priority::Normal,
                    language: "auto".to_string(),
                    symbol: None,
                    context: None,
                    confidence: 0.8, // Split confidence
                    ocr_metadata: None,
                    source_type: SourceType::NativeText,
                    extraction_method: ExtractionMethod::DirectRead,
                },
                group_id: "split".to_string(),
            };
            chunks.push(chunk);
            
            // Préparer le chunk suivant avec overlap (respect UTF-8 boundaries)
            if overlap > 0 && current_chunk.len() > overlap {
                // Find the nearest character boundary for overlap
                let mut overlap_start = current_chunk.len().saturating_sub(overlap);
                while overlap_start > 0 && !current_chunk.is_char_boundary(overlap_start) {
                    overlap_start -= 1;
                }
                current_chunk = current_chunk[overlap_start..].to_string();
            } else {
                current_chunk.clear();
            }
            chunk_index += 1;
        }
        
        // Ajouter le paragraphe actuel
        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(paragraph);
    }
    
    // Ajouter le dernier chunk s'il contient du contenu
    if !current_chunk.trim().is_empty() {
        let chunk = EnrichedChunk {
            id: format!("chunk_{}_split_{}", uuid::Uuid::new_v4().simple(), chunk_index),
            content: current_chunk.trim().to_string(),
            start_line: 0,
            end_line: 0,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: blake3::hash(current_chunk.trim().as_bytes()).to_hex().to_string(),
            metadata: ChunkMetadata {
                tags: vec!["fallback-split".to_string()],
                priority: Priority::Normal,
                language: "auto".to_string(),
                symbol: None,
                context: None,
                confidence: 0.8, // Split confidence
                ocr_metadata: None,
                source_type: SourceType::NativeText,
                extraction_method: ExtractionMethod::DirectRead,
            },
            group_id: "split".to_string(),
        };
        chunks.push(chunk);
    }
    
    chunks
}

/// Fallback split agressif par pages/paragraphes pour documents longs
fn fallback_split_by_pages_or_paragraphs(content: &str, target_size: usize) -> Vec<EnrichedChunk> {
    let mut chunks = Vec::new();
    let mut chunk_index = 0;
    
    // Essayer d'abord par pages (séparées par sauts de page multiples)
    let page_splits: Vec<&str> = content.split("\n\n\n").collect();
    if page_splits.len() > 1 {
        // Split par pages
        for page in page_splits {
            if page.trim().len() > target_size {
                // Page trop grande, subdiviser par paragraphes
                let para_chunks = split_by_paragraphs(page, target_size, &mut chunk_index);
                chunks.extend(para_chunks);
            } else if !page.trim().is_empty() {
                // Page de taille raisonnable
                let chunk = create_fallback_chunk(page.trim(), chunk_index);
                chunks.push(chunk);
                chunk_index += 1;
            }
        }
    } else {
        // Pas de pages distinctes, split par paragraphes
        let para_chunks = split_by_paragraphs(content, target_size, &mut chunk_index);
        chunks.extend(para_chunks);
    }
    
    chunks
}

/// Split par paragraphes avec taille cible
fn split_by_paragraphs(content: &str, target_size: usize, chunk_index: &mut usize) -> Vec<EnrichedChunk> {
    let mut chunks = Vec::new();
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut current_chunk = String::new();
    
    for paragraph in paragraphs {
        let para_trimmed = paragraph.trim();
        if para_trimmed.is_empty() {
            continue;
        }
        
        // Si ajouter ce paragraphe dépasse la taille, finaliser le chunk actuel
        if !current_chunk.is_empty() && current_chunk.len() + para_trimmed.len() > target_size {
            let chunk = create_fallback_chunk(&current_chunk, *chunk_index);
            chunks.push(chunk);
            *chunk_index += 1;
            current_chunk.clear();
        }
        
        // Ajouter le paragraphe
        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(para_trimmed);
    }
    
    // Ajouter le dernier chunk
    if !current_chunk.trim().is_empty() {
        let chunk = create_fallback_chunk(&current_chunk, *chunk_index);
        chunks.push(chunk);
    }
    
    chunks
}

/// Créer un chunk de fallback avec métadonnées appropriées
fn create_fallback_chunk(content: &str, index: usize) -> EnrichedChunk {
    EnrichedChunk {
        id: format!("chunk_{}_fallback_{}", uuid::Uuid::new_v4().simple(), index),
        content: content.to_string(),
        start_line: 0,
        end_line: 0,
        chunk_type: ChunkType::TextBlock,
        embedding: None,
        hash: blake3::hash(content.as_bytes()).to_hex().to_string(),
        metadata: ChunkMetadata {
            tags: vec!["fallback-aggressive".to_string()],
            priority: Priority::Normal,
            language: "auto".to_string(),
            symbol: None,
            context: None,
            confidence: 0.7, // Confidence plus faible pour fallback
            ocr_metadata: None,
            source_type: SourceType::NativeText,
            extraction_method: ExtractionMethod::DirectRead,
        },
        group_id: "fallback".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ocr::{TesseractConfig, OcrConfig};

    #[tokio::test]
    async fn test_document_processor_text_file() {
        // Test basique de traitement fichier texte
        let temp_file = std::env::temp_dir().join("test_doc_processor.txt");
        tokio::fs::write(&temp_file, "Test content for chunking").await.unwrap();

        let ocr_config = OcrConfig::default();
        let tesseract_config = TesseractConfig::default();
        let ocr_processor = TesseractProcessor::new(ocr_config, tesseract_config).unwrap();
        
        // TODO: Mock embedder pour tests
        // let embedder = CustomE5Embedder::new(CustomE5Config::default()).await.unwrap();
        // let processor = DocumentProcessor::new(ocr_processor, embedder).await.unwrap();
        
        // Cleanup
        let _ = tokio::fs::remove_file(&temp_file).await;
    }
}