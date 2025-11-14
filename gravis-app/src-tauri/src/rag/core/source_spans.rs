// GRAVIS RAG Module - Phase 4A PR #1: Source Spans & Traçabilité
// Ajout de source spans avec bbox + char offsets pour explainability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Span source avec traçabilité complète pour explainability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceSpan {
    /// ID unique du span
    pub span_id: String,
    
    /// Référence au document source
    pub document_id: String,
    pub document_path: PathBuf,
    
    /// Position dans le document original
    pub char_start: usize,
    pub char_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    
    /// Bounding box pour images/PDF (coordonnées)
    pub bbox: Option<BoundingBox>,
    
    /// Contenu original du span
    pub original_content: String,
    
    /// Métadonnées d'extraction
    pub extraction_metadata: ExtractionMetadata,
    
    /// Timestamp de création
    pub created_at: std::time::SystemTime,
}

/// Bounding box pour traçabilité visuelle (images, PDF)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    /// Page (pour PDF multi-pages)
    pub page: Option<usize>,
    
    /// Coordonnées rectangulaires (pixels ou points)
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    
    /// Rotation si applicable (degrés)
    pub rotation: Option<f32>,
    
    /// Système de coordonnées utilisé
    pub coordinate_system: CoordinateSystem,
}

/// Système de coordonnées pour bbox
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoordinateSystem {
    /// Coordonnées image standard (0,0 = top-left)
    ImagePixels { dpi: Option<f32> },
    
    /// Points PDF (1 point = 1/72 inch)
    PdfPoints,
    
    /// Coordonnées normalisées (0.0-1.0)
    Normalized,
    
    /// Coordonnées personnalisées
    Custom { unit: String, scale_factor: f32 },
}

/// Métadonnées d'extraction enrichies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionMetadata {
    /// Méthode d'extraction utilisée
    pub method: crate::rag::ExtractionMethod,
    
    /// Confidence de l'extraction
    pub confidence: f32,
    
    /// Langue détectée
    pub language: Option<String>,
    
    /// Propriétés spécifiques selon la méthode
    pub method_specific: HashMap<String, serde_json::Value>,
    
    /// Hash du contenu original pour vérification
    pub content_hash: String,
}

/// Gestionnaire des source spans
#[derive(Debug)]
pub struct SourceSpanManager {
    /// Cache des spans par document
    spans_by_document: HashMap<String, Vec<SourceSpan>>,
    
    /// Index par char offsets pour recherche rapide
    char_offset_index: HashMap<String, Vec<(usize, usize, String)>>, // (start, end, span_id)
    
    /// Statistiques d'utilisation
    stats: SpanStats,
}

/// Statistiques des source spans
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpanStats {
    pub total_spans: usize,
    pub spans_with_bbox: usize,
    pub spans_by_extraction_method: HashMap<String, usize>,
    pub average_span_length: f32,
    pub total_documents: usize,
}

impl SourceSpanManager {
    /// Créer un nouveau gestionnaire de source spans
    pub fn new() -> Self {
        Self {
            spans_by_document: HashMap::new(),
            char_offset_index: HashMap::new(),
            stats: SpanStats::default(),
        }
    }
    
    /// Ajouter un source span
    pub fn add_span(&mut self, span: SourceSpan) -> Result<(), SourceSpanError> {
        // Validation du span
        self.validate_span(&span)?;
        
        // Mise à jour de l'index
        self.update_char_offset_index(&span);
        
        // Ajout au cache
        let document_spans = self.spans_by_document
            .entry(span.document_id.clone())
            .or_insert_with(Vec::new);
        document_spans.push(span);
        
        // Mise à jour des statistiques
        self.update_stats();
        
        Ok(())
    }
    
    /// Récupérer tous les spans d'un document
    pub fn get_spans_for_document(&self, document_id: &str) -> Option<&Vec<SourceSpan>> {
        self.spans_by_document.get(document_id)
    }
    
    /// Trouver le span contenant une position donnée
    pub fn find_span_at_position(&self, document_id: &str, char_offset: usize) -> Option<&SourceSpan> {
        let spans = self.spans_by_document.get(document_id)?;
        
        spans.iter().find(|span| 
            span.char_start <= char_offset && char_offset < span.char_end
        )
    }
    
    /// Récupérer les spans dans une plage
    pub fn get_spans_in_range(&self, document_id: &str, start: usize, end: usize) -> Vec<&SourceSpan> {
        let spans = match self.spans_by_document.get(document_id) {
            Some(spans) => spans,
            None => return Vec::new(),
        };
        
        spans.iter()
            .filter(|span| {
                // Intersection: span overlap avec la plage
                !(span.char_end <= start || span.char_start >= end)
            })
            .collect()
    }
    
    /// Générer un rapport d'explainability pour un chunk
    pub fn generate_explainability_report(&self, chunk_content: &str, document_id: &str) -> ExplainabilityReport {
        let related_spans: Vec<SourceSpan> = self.spans_by_document
            .get(document_id)
            .map(|spans| spans.iter().filter(|span| {
                // Heuristique: spans qui contribuent au chunk
                chunk_content.contains(&span.original_content) || 
                span.original_content.contains(chunk_content)
            }).cloned().collect())
            .unwrap_or_default();
            
        ExplainabilityReport {
            chunk_content: chunk_content.to_string(),
            document_id: document_id.to_string(),
            confidence_score: self.calculate_confidence(&related_spans),
            coverage_percentage: self.calculate_coverage(&related_spans, chunk_content),
            contributing_spans: related_spans,
            generated_at: std::time::SystemTime::now(),
        }
    }
    
    /// Validation d'un span
    fn validate_span(&self, span: &SourceSpan) -> Result<(), SourceSpanError> {
        if span.char_start >= span.char_end {
            return Err(SourceSpanError::InvalidRange(
                format!("char_start ({}) >= char_end ({})", span.char_start, span.char_end)
            ));
        }
        
        if span.line_start > span.line_end {
            return Err(SourceSpanError::InvalidRange(
                format!("line_start ({}) > line_end ({})", span.line_start, span.line_end)
            ));
        }
        
        if span.original_content.is_empty() {
            return Err(SourceSpanError::EmptyContent);
        }
        
        Ok(())
    }
    
    /// Mise à jour de l'index char offset
    fn update_char_offset_index(&mut self, span: &SourceSpan) {
        let index_entry = self.char_offset_index
            .entry(span.document_id.clone())
            .or_insert_with(Vec::new);
        
        index_entry.push((span.char_start, span.char_end, span.span_id.clone()));
        
        // Tri pour recherche binaire
        index_entry.sort_by_key(|(start, _, _)| *start);
    }
    
    /// Mise à jour des statistiques
    fn update_stats(&mut self) {
        self.stats.total_spans = self.spans_by_document.values()
            .map(|spans| spans.len())
            .sum();
            
        self.stats.total_documents = self.spans_by_document.len();
        
        self.stats.spans_with_bbox = self.spans_by_document.values()
            .flatten()
            .filter(|span| span.bbox.is_some())
            .count();
            
        // Calcul longueur moyenne
        let total_length: usize = self.spans_by_document.values()
            .flatten()
            .map(|span| span.char_end - span.char_start)
            .sum();
            
        self.stats.average_span_length = if self.stats.total_spans > 0 {
            total_length as f32 / self.stats.total_spans as f32
        } else {
            0.0
        };
    }
    
    /// Calculer la confidence d'un rapport
    fn calculate_confidence(&self, spans: &[SourceSpan]) -> f32 {
        if spans.is_empty() {
            return 0.0;
        }
        
        spans.iter()
            .map(|span| span.extraction_metadata.confidence)
            .sum::<f32>() / spans.len() as f32
    }
    
    /// Calculer le pourcentage de couverture
    fn calculate_coverage(&self, spans: &[SourceSpan], chunk_content: &str) -> f32 {
        if spans.is_empty() || chunk_content.is_empty() {
            return 0.0;
        }
        
        let covered_chars: usize = spans.iter()
            .map(|span| {
                // Approximation: intersection contenu
                let common = span.original_content.chars()
                    .filter(|c| chunk_content.contains(*c))
                    .count();
                common
            })
            .sum();
            
        (covered_chars as f32 / chunk_content.len() as f32) * 100.0
    }
    
    /// Obtenir les statistiques
    pub fn get_stats(&self) -> &SpanStats {
        &self.stats
    }
    
    /// Nettoyer les spans d'un document supprimé
    pub fn remove_document_spans(&mut self, document_id: &str) {
        self.spans_by_document.remove(document_id);
        self.char_offset_index.remove(document_id);
        self.update_stats();
    }
}

/// Rapport d'explainability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainabilityReport {
    pub chunk_content: String,
    pub document_id: String,
    pub contributing_spans: Vec<SourceSpan>,
    pub confidence_score: f32,
    pub coverage_percentage: f32,
    pub generated_at: std::time::SystemTime,
}

/// Erreurs de source spans
#[derive(Debug, thiserror::Error)]
pub enum SourceSpanError {
    #[error("Invalid range: {0}")]
    InvalidRange(String),
    
    #[error("Empty content not allowed")]
    EmptyContent,
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Span not found: {0}")]
    SpanNotFound(String),
}

impl Default for SourceSpanManager {
    fn default() -> Self {
        Self::new()
    }
}

// === Implémentations utilitaires ===

impl SourceSpan {
    /// Créer un nouveau source span
    pub fn new(
        document_id: String,
        document_path: PathBuf,
        char_start: usize,
        char_end: usize,
        original_content: String,
        extraction_method: crate::rag::ExtractionMethod,
    ) -> Self {
        let span_id = format!("span_{}_{}", 
            blake3::hash(document_id.as_bytes()).to_hex()[..8].to_string(),
            blake3::hash(format!("{}:{}", char_start, char_end).as_bytes()).to_hex()[..8].to_string()
        );
        
        let content_hash = blake3::hash(original_content.as_bytes()).to_hex().to_string();
        
        Self {
            span_id,
            document_id,
            document_path,
            char_start,
            char_end,
            line_start: 0, // À calculer selon le contenu
            line_end: 0,   // À calculer selon le contenu
            bbox: None,
            original_content,
            extraction_metadata: ExtractionMetadata {
                method: extraction_method,
                confidence: 1.0,
                language: None,
                method_specific: HashMap::new(),
                content_hash,
            },
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Ajouter une bounding box
    pub fn with_bbox(mut self, bbox: BoundingBox) -> Self {
        self.bbox = Some(bbox);
        self
    }
    
    /// Ajouter des métadonnées spécifiques
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.extraction_metadata.method_specific.insert(key, value);
        self
    }
    
    /// Calculer les lignes à partir du contenu
    pub fn calculate_lines(&mut self, full_content: &str) {
        let lines_before = full_content[..self.char_start].matches('\n').count();
        let lines_in_span = self.original_content.matches('\n').count();
        
        self.line_start = lines_before + 1; // 1-indexed
        self.line_end = self.line_start + lines_in_span;
    }
}

impl BoundingBox {
    /// Créer une bbox pour image en pixels
    pub fn image_pixels(x: f32, y: f32, width: f32, height: f32, dpi: Option<f32>) -> Self {
        Self {
            page: None,
            x, y, width, height,
            rotation: None,
            coordinate_system: CoordinateSystem::ImagePixels { dpi },
        }
    }
    
    /// Créer une bbox pour PDF en points
    pub fn pdf_points(page: usize, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            page: Some(page),
            x, y, width, height,
            rotation: None,
            coordinate_system: CoordinateSystem::PdfPoints,
        }
    }
    
    /// Créer une bbox normalisée (0.0-1.0)
    pub fn normalized(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            page: None,
            x, y, width, height,
            rotation: None,
            coordinate_system: CoordinateSystem::Normalized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::ExtractionMethod;

    #[test]
    fn test_source_span_creation() {
        let span = SourceSpan::new(
            "doc1".to_string(),
            PathBuf::from("/test/doc1.txt"),
            10,
            50,
            "test content".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        assert_eq!(span.document_id, "doc1");
        assert_eq!(span.char_start, 10);
        assert_eq!(span.char_end, 50);
        assert_eq!(span.original_content, "test content");
        assert!(!span.span_id.is_empty());
    }
    
    #[test]
    fn test_source_span_manager() {
        let mut manager = SourceSpanManager::new();
        
        let span = SourceSpan::new(
            "doc1".to_string(),
            PathBuf::from("/test/doc1.txt"),
            10,
            50,
            "test content".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        manager.add_span(span).unwrap();
        
        assert_eq!(manager.get_stats().total_spans, 1);
        assert!(manager.get_spans_for_document("doc1").is_some());
        assert!(manager.find_span_at_position("doc1", 25).is_some());
        assert!(manager.find_span_at_position("doc1", 5).is_none());
    }
    
    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::pdf_points(1, 100.0, 200.0, 300.0, 50.0);
        
        assert_eq!(bbox.page, Some(1));
        assert_eq!(bbox.x, 100.0);
        assert_eq!(bbox.y, 200.0);
        assert!(matches!(bbox.coordinate_system, CoordinateSystem::PdfPoints));
    }
    
    #[test]
    fn test_explainability_report() {
        let mut manager = SourceSpanManager::new();
        
        let span = SourceSpan::new(
            "doc1".to_string(),
            PathBuf::from("/test/doc1.txt"),
            0,
            12,
            "test content".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        manager.add_span(span).unwrap();
        
        let report = manager.generate_explainability_report("test content", "doc1");
        
        assert_eq!(report.document_id, "doc1");
        assert_eq!(report.contributing_spans.len(), 1);
        assert!(report.confidence_score > 0.0);
    }
}