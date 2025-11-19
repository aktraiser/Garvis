// GRAVIS RAG Module - Phase 4A PR #1: Span-Aware Chunker
// Chunker intelligent qui génère des source spans pour explainability

use crate::rag::{
    EnrichedChunk, ChunkType, ChunkMetadata, SourceType, ExtractionMethod, ChunkSource,
    SourceSpan, SourceSpanManager, SourceBoundingBox as BoundingBox
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Chunker avec génération automatique de source spans
#[derive(Debug)]
pub struct SpanAwareChunker {
    /// Gestionnaire des source spans
    span_manager: SourceSpanManager,
    
    /// Configuration de chunking
    config: SpanAwareChunkConfig,
}

/// Configuration pour chunking avec source spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAwareChunkConfig {
    /// Taille cible des chunks (en tokens)
    pub target_tokens: usize,
    
    /// Overlap entre chunks (pourcentage)
    pub overlap_percent: f32,
    
    /// Générer des spans même pour du texte natif
    pub generate_native_spans: bool,
    
    /// Préserver les bounding boxes lors du chunking
    pub preserve_bboxes: bool,
    
    /// Taille minimale d'un chunk pour créer un span
    pub min_span_length: usize,
}

impl Default for SpanAwareChunkConfig {
    fn default() -> Self {
        Self {
            target_tokens: 384,
            overlap_percent: 0.12, // 12%
            generate_native_spans: true,
            preserve_bboxes: true,
            min_span_length: 50,
        }
    }
}

/// Résultat de chunking avec spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAwareChunkResult {
    /// Chunks générés
    pub chunks: Vec<EnrichedChunk>,
    
    /// Spans créés
    pub created_spans: Vec<SourceSpan>,
    
    /// Statistiques du chunking
    pub stats: ChunkingStats,
}

/// Statistiques de chunking avec spans
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkingStats {
    pub total_chunks: usize,
    pub total_spans: usize,
    pub spans_with_bbox: usize,
    pub average_chunk_length: f32,
    pub overlap_efficiency: f32,
    pub processing_time_ms: u64,
}

impl SpanAwareChunker {
    /// Créer un nouveau chunker avec source spans
    pub fn new(config: SpanAwareChunkConfig) -> Self {
        Self {
            span_manager: SourceSpanManager::new(),
            config,
        }
    }
    
    /// Chunker un document avec génération de source spans
    pub fn chunk_with_spans(
        &mut self,
        document_id: String,
        document_path: PathBuf,
        content: &str,
        extraction_method: ExtractionMethod,
        group_id: String,
        bboxes: Option<Vec<BoundingBox>>, // Bounding boxes par paragraphe/section
    ) -> Result<SpanAwareChunkResult, SpanChunkError> {
        let start_time = std::time::Instant::now();
        
        // 1. Chunking basique du contenu
        let base_chunks = self.create_base_chunks(content)?;
        
        // 2. Génération des source spans
        let mut created_spans = Vec::new();
        let mut enriched_chunks = Vec::new();
        
        for (chunk_idx, (chunk_content, char_start, char_end)) in base_chunks.iter().enumerate() {
            // Créer le source span
            let span = self.create_source_span(
                &document_id,
                &document_path,
                *char_start,
                *char_end,
                chunk_content.clone(),
                extraction_method.clone(),
                bboxes.as_ref(),
                chunk_idx,
            )?;
            
            // Ajouter le span au gestionnaire
            self.span_manager.add_span(span.clone())?;
            created_spans.push(span.clone());
            
            // Créer le chunk enrichi
            let enriched_chunk = self.create_enriched_chunk(
                chunk_content,
                chunk_idx,
                &group_id,
                &extraction_method,
                vec![span.span_id.clone()],
            );
            
            enriched_chunks.push(enriched_chunk);
        }
        
        // 3. Calcul des statistiques
        let processing_time = start_time.elapsed().as_millis() as u64;
        let stats = self.calculate_stats(&enriched_chunks, &created_spans, processing_time);
        
        Ok(SpanAwareChunkResult {
            chunks: enriched_chunks,
            created_spans,
            stats,
        })
    }
    
    /// Récupérer les spans d'un chunk donné
    pub fn get_spans_for_chunk(&self, chunk: &EnrichedChunk) -> Vec<&SourceSpan> {
        if let Some(span_ids) = &chunk.source_spans {
            // TODO: Implémenter récupération par ID
            self.span_manager.get_spans_for_document(&chunk.metadata.source_type.to_string())
                .map(|spans| spans.iter().filter(|span| 
                    span_ids.contains(&span.span_id)
                ).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }
    
    /// Générer un rapport d'explainability pour un chunk
    pub fn explain_chunk(&self, chunk: &EnrichedChunk) -> Option<crate::rag::ExplainabilityReport> {
        if let Some(span_ids) = &chunk.source_spans {
            if !span_ids.is_empty() {
                // Utiliser le premier span pour récupérer le document_id
                // Dans une implémentation réelle, on stockerait le document_id avec le chunk
                let document_id = format!("doc_{}", chunk.group_id); // Approximation
                return Some(self.span_manager.generate_explainability_report(&chunk.content, &document_id));
            }
        }
        None
    }
    
    /// Chunking basique en fenêtres avec overlap
    fn create_base_chunks(&self, content: &str) -> Result<Vec<(String, usize, usize)>, SpanChunkError> {
        if content.is_empty() {
            return Ok(Vec::new());
        }
        
        let chars: Vec<char> = content.chars().collect();
        let total_chars = chars.len();
        
        if total_chars < self.config.min_span_length {
            return Ok(vec![(content.to_string(), 0, total_chars)]);
        }
        
        let mut chunks = Vec::new();
        let chunk_size = self.estimate_char_count_for_tokens(self.config.target_tokens);
        let overlap_size = (chunk_size as f32 * self.config.overlap_percent) as usize;
        
        let mut start = 0;
        
        while start < total_chars {
            let end = std::cmp::min(start + chunk_size, total_chars);
            
            // Ajustement sur limites de mots si possible
            let adjusted_end = if end < total_chars {
                self.find_word_boundary(&chars, end)
            } else {
                end
            };
            
            let chunk_chars: String = chars[start..adjusted_end].iter().collect();
            
            if chunk_chars.trim().len() >= self.config.min_span_length {
                chunks.push((chunk_chars, start, adjusted_end));
            }
            
            // Avancement avec overlap
            if adjusted_end >= total_chars {
                break;
            }
            
            start = adjusted_end.saturating_sub(overlap_size);
        }
        
        Ok(chunks)
    }
    
    /// Créer un source span pour un chunk
    fn create_source_span(
        &self,
        document_id: &str,
        document_path: &PathBuf,
        char_start: usize,
        char_end: usize,
        content: String,
        extraction_method: ExtractionMethod,
        bboxes: Option<&Vec<BoundingBox>>,
        chunk_index: usize,
    ) -> Result<SourceSpan, SpanChunkError> {
        let mut span = SourceSpan::new(
            document_id.to_string(),
            document_path.clone(),
            char_start,
            char_end,
            content,
            extraction_method,
        );
        
        // Ajouter bbox si disponible
        if self.config.preserve_bboxes {
            if let Some(bboxes) = bboxes {
                if chunk_index < bboxes.len() {
                    span = span.with_bbox(bboxes[chunk_index].clone());
                }
            }
        }
        
        Ok(span)
    }
    
    /// Créer un chunk enrichi
    fn create_enriched_chunk(
        &self,
        content: &str,
        chunk_index: usize,
        group_id: &str,
        extraction_method: &ExtractionMethod,
        span_ids: Vec<String>,
    ) -> EnrichedChunk {
        let chunk_id = format!("chunk_{}_{}", group_id, chunk_index);
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        
        // Détection basique du type de chunk
        let chunk_type = if content.trim().starts_with("fn ") || content.trim().starts_with("function ") {
            ChunkType::Function
        } else if content.trim().starts_with("class ") || content.trim().starts_with("struct ") {
            ChunkType::Class
        } else if content.trim().starts_with("//") || content.trim().starts_with("/*") {
            ChunkType::Comment
        } else {
            ChunkType::TextBlock
        };
        
        // Calcul approximatif des lignes
        let lines_count = content.matches('\n').count() + 1;
        
        EnrichedChunk {
            id: chunk_id,
            content: content.to_string(),
            start_line: 1, // À améliorer avec position réelle
            end_line: lines_count,
            chunk_type,
            embedding: None,
            hash: content_hash,
            metadata: ChunkMetadata {
                tags: vec!["chunked".to_string()],
                priority: crate::rag::Priority::Normal,
                language: "auto".to_string(),
                symbol: None,
                context: None,
                confidence: 0.95,
                ocr_metadata: None,
                source_type: match extraction_method {
                    ExtractionMethod::DirectRead => SourceType::NativeText,
                    ExtractionMethod::TesseractOcr { .. } => SourceType::OcrExtracted,
                    ExtractionMethod::PdfNative => SourceType::HybridPdfNative,
                    ExtractionMethod::PdfOcrFallback => SourceType::HybridPdfOcr,
                    ExtractionMethod::HybridIntelligent => SourceType::HybridPdfNative,
                },
                extraction_method: extraction_method.clone(),
            },
            group_id: group_id.to_string(),
            source_spans: Some(span_ids),
            chunk_source: ChunkSource::BodyText,
            figure_id: None,
        }
    }
    
    /// Trouver une limite de mot appropriée
    fn find_word_boundary(&self, chars: &[char], target_pos: usize) -> usize {
        if target_pos >= chars.len() {
            return chars.len();
        }
        
        // Chercher en arrière jusqu'à un espace ou ponctuation
        for i in (target_pos.saturating_sub(50)..target_pos).rev() {
            if chars[i].is_whitespace() || chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
                return i + 1;
            }
        }
        
        target_pos
    }
    
    /// Estimation tokens -> caractères (approximation)
    fn estimate_char_count_for_tokens(&self, tokens: usize) -> usize {
        // Approximation: 1 token ≈ 4 caractères pour du texte français/anglais
        tokens * 4
    }
    
    /// Calculer les statistiques de chunking
    fn calculate_stats(
        &self,
        chunks: &[EnrichedChunk],
        spans: &[SourceSpan],
        processing_time_ms: u64,
    ) -> ChunkingStats {
        let total_chunks = chunks.len();
        let total_spans = spans.len();
        
        let spans_with_bbox = spans.iter()
            .filter(|span| span.bbox.is_some())
            .count();
            
        let average_chunk_length = if total_chunks > 0 {
            chunks.iter()
                .map(|chunk| chunk.content.len())
                .sum::<usize>() as f32 / total_chunks as f32
        } else {
            0.0
        };
        
        let overlap_efficiency = if total_chunks > 1 {
            // Calcul approximatif de l'efficacité de l'overlap
            self.config.overlap_percent * 100.0
        } else {
            0.0
        };
        
        ChunkingStats {
            total_chunks,
            total_spans,
            spans_with_bbox,
            average_chunk_length,
            overlap_efficiency,
            processing_time_ms,
        }
    }
    
    /// Accéder au gestionnaire de spans (pour récupération)
    pub fn span_manager(&self) -> &SourceSpanManager {
        &self.span_manager
    }
}

/// Erreurs de chunking avec spans
#[derive(Debug, thiserror::Error)]
pub enum SpanChunkError {
    #[error("Empty content")]
    EmptyContent,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Source span error: {0}")]
    SourceSpanError(#[from] crate::rag::SourceSpanError),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

// === Helper pour conversion des types existants ===

impl ToString for SourceType {
    fn to_string(&self) -> String {
        match self {
            SourceType::NativeText => "native_text".to_string(),
            SourceType::OcrExtracted => "ocr_extracted".to_string(),
            SourceType::HybridPdfNative => "hybrid_pdf_native".to_string(),
            SourceType::HybridPdfOcr => "hybrid_pdf_ocr".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ExtractionMethod;

    #[test]
    fn test_span_aware_chunker_creation() {
        let config = SpanAwareChunkConfig::default();
        let chunker = SpanAwareChunker::new(config);
        
        assert_eq!(chunker.span_manager.get_stats().total_spans, 0);
    }
    
    #[test]
    fn test_basic_chunking_with_spans() {
        let mut chunker = SpanAwareChunker::new(SpanAwareChunkConfig::default());
        
        let content = "Ceci est un test de chunking. Il devrait créer des source spans appropriés pour chaque chunk généré.";
        
        let result = chunker.chunk_with_spans(
            "test_doc".to_string(),
            PathBuf::from("/test/doc.txt"),
            content,
            ExtractionMethod::DirectRead,
            "test_group".to_string(),
            None,
        ).unwrap();
        
        assert!(!result.chunks.is_empty());
        assert_eq!(result.chunks.len(), result.created_spans.len());
        assert!(result.stats.total_chunks > 0);
        assert!(result.stats.total_spans > 0);
        
        // Vérifier que les chunks ont des références aux spans
        for chunk in &result.chunks {
            assert!(chunk.source_spans.is_some());
            assert!(!chunk.source_spans.as_ref().unwrap().is_empty());
        }
    }
    
    #[test]
    fn test_bbox_preservation() {
        let mut chunker = SpanAwareChunker::new(SpanAwareChunkConfig {
            preserve_bboxes: true,
            ..Default::default()
        });
        
        let bboxes = vec![
            BoundingBox::pdf_points(1, 100.0, 200.0, 300.0, 50.0),
            BoundingBox::pdf_points(1, 100.0, 250.0, 300.0, 50.0),
        ];
        
        let content = "Premier chunk avec bbox. Deuxième chunk avec autre bbox.";
        
        let result = chunker.chunk_with_spans(
            "test_doc".to_string(),
            PathBuf::from("/test/doc.pdf"),
            content,
            ExtractionMethod::PdfNative,
            "test_group".to_string(),
            Some(bboxes),
        ).unwrap();
        
        // Vérifier que les spans ont des bboxes
        let spans_with_bbox = result.created_spans.iter()
            .filter(|span| span.bbox.is_some())
            .count();
        
        assert!(spans_with_bbox > 0);
        assert_eq!(result.stats.spans_with_bbox, spans_with_bbox);
    }
}