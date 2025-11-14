// Integration test pour vérifier que source_spans fonctionne vraiment

#[cfg(test)]
mod integration_tests {
    use crate::rag::{
        SourceSpan, SourceSpanManager, SourceBoundingBox as BoundingBox, CoordinateSystem,
        ExtractionMethod, EnrichedChunk, ChunkType, ChunkMetadata, SourceType, Priority
    };
    use std::path::PathBuf;

    #[test]
    fn test_source_span_creation_and_management() {
        println!("🧪 Test création et gestion des source spans");
        
        let mut manager = SourceSpanManager::new();
        
        // Créer un span de test
        let span = SourceSpan::new(
            "doc_test".to_string(),
            PathBuf::from("/test/document.txt"),
            0,
            50,
            "Ceci est un texte de test pour vérifier les source spans.".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        println!("✅ Span créé avec ID: {}", span.span_id);
        assert!(!span.span_id.is_empty());
        assert_eq!(span.document_id, "doc_test");
        assert_eq!(span.char_start, 0);
        assert_eq!(span.char_end, 50);
        
        // Ajouter le span au manager
        let result = manager.add_span(span.clone());
        assert!(result.is_ok(), "❌ Échec ajout span: {:?}", result.err());
        println!("✅ Span ajouté au manager");
        
        // Vérifier récupération
        let retrieved_spans = manager.get_spans_for_document("doc_test");
        assert!(retrieved_spans.is_some(), "❌ Document non trouvé");
        assert_eq!(retrieved_spans.unwrap().len(), 1, "❌ Nombre de spans incorrect");
        println!("✅ Span récupéré correctement");
        
        // Test de recherche par position
        let span_at_pos = manager.find_span_at_position("doc_test", 25);
        assert!(span_at_pos.is_some(), "❌ Span non trouvé à la position 25");
        println!("✅ Recherche par position fonctionne");
        
        let span_outside = manager.find_span_at_position("doc_test", 100);
        assert!(span_outside.is_none(), "❌ Span trouvé en dehors de la plage");
        println!("✅ Recherche hors plage fonctionne");
        
        // Test des statistiques
        let stats = manager.get_stats();
        assert_eq!(stats.total_spans, 1);
        assert_eq!(stats.total_documents, 1);
        println!("✅ Statistiques correctes: {} spans, {} docs", stats.total_spans, stats.total_documents);
    }
    
    #[test] 
    fn test_bbox_creation_and_coordination() {
        println!("🧪 Test création bounding boxes");
        
        // Test bbox image
        let bbox_image = BoundingBox::image_pixels(100.0, 200.0, 300.0, 150.0, Some(96.0));
        assert_eq!(bbox_image.x, 100.0);
        assert_eq!(bbox_image.y, 200.0);
        assert!(matches!(bbox_image.coordinate_system, CoordinateSystem::ImagePixels { dpi: Some(96.0) }));
        println!("✅ BBox image créée: x={}, y={}, w={}, h={}", bbox_image.x, bbox_image.y, bbox_image.width, bbox_image.height);
        
        // Test bbox PDF
        let bbox_pdf = BoundingBox::pdf_points(1, 50.0, 100.0, 200.0, 75.0);
        assert_eq!(bbox_pdf.page, Some(1));
        assert!(matches!(bbox_pdf.coordinate_system, CoordinateSystem::PdfPoints));
        println!("✅ BBox PDF créée: page={:?}, x={}, y={}", bbox_pdf.page, bbox_pdf.x, bbox_pdf.y);
        
        // Test bbox normalisée
        let bbox_norm = BoundingBox::normalized(0.25, 0.5, 0.4, 0.3);
        assert_eq!(bbox_norm.x, 0.25);
        assert!(matches!(bbox_norm.coordinate_system, CoordinateSystem::Normalized));
        println!("✅ BBox normalisée créée: x={}, y={}", bbox_norm.x, bbox_norm.y);
    }
    
    #[test]
    fn test_explainability_report() {
        println!("🧪 Test rapport d'explainability");
        
        let mut manager = SourceSpanManager::new();
        
        // Créer plusieurs spans qui contribuent à un chunk
        let span1 = SourceSpan::new(
            "doc_test".to_string(),
            PathBuf::from("/test/doc.txt"),
            0,
            20,
            "Premier morceau".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        let span2 = SourceSpan::new(
            "doc_test".to_string(),
            PathBuf::from("/test/doc.txt"),
            20,
            40,
            "Deuxième morceau".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        manager.add_span(span1).unwrap();
        manager.add_span(span2).unwrap();
        
        // Générer rapport pour un chunk qui contient des parties des spans
        let chunk_content = "Premier morceau de test";
        let report = manager.generate_explainability_report(chunk_content, "doc_test");
        
        println!("✅ Rapport généré:");
        println!("  - Document ID: {}", report.document_id);
        println!("  - Spans contributeurs: {}", report.contributing_spans.len());
        println!("  - Score confidence: {:.2}", report.confidence_score);
        println!("  - Couverture: {:.1}%", report.coverage_percentage);
        
        assert_eq!(report.document_id, "doc_test");
        assert!(report.confidence_score > 0.0);
        assert!(!report.contributing_spans.is_empty());
    }
    
    #[test]
    fn test_chunk_with_source_spans() {
        println!("🧪 Test EnrichedChunk avec source spans");
        
        // Créer un chunk avec des références de spans
        let span_ids = vec!["span_123".to_string(), "span_456".to_string()];
        
        let chunk = EnrichedChunk {
            id: "chunk_test".to_string(),
            content: "Contenu de test avec source spans".to_string(),
            start_line: 1,
            end_line: 2,
            chunk_type: ChunkType::TextBlock,
            embedding: None,
            hash: "test_hash".to_string(),
            metadata: ChunkMetadata {
                tags: vec!["test".to_string()],
                priority: Priority::Normal,
                language: "fr".to_string(),
                symbol: None,
                context: None,
                confidence: 1.0,
                ocr_metadata: None,
                source_type: SourceType::NativeText,
                extraction_method: ExtractionMethod::DirectRead,
            },
            group_id: "test_group".to_string(),
            source_spans: Some(span_ids.clone()),
        };
        
        println!("✅ Chunk créé avec {} source spans", 
                chunk.source_spans.as_ref().unwrap().len());
        assert_eq!(chunk.source_spans.unwrap(), span_ids);
        println!("✅ Références aux spans préservées");
    }
    
    #[test]
    fn test_span_ranges_and_overlaps() {
        println!("🧪 Test plages et chevauchements de spans");
        
        let mut manager = SourceSpanManager::new();
        
        // Créer des spans qui se chevauchent
        let span1 = SourceSpan::new(
            "doc_overlap".to_string(),
            PathBuf::from("/test/overlap.txt"),
            0, 30, "Début du texte avec overlap".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        let span2 = SourceSpan::new(
            "doc_overlap".to_string(),
            PathBuf::from("/test/overlap.txt"), 
            20, 50, "overlap au milieu et fin".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        let span3 = SourceSpan::new(
            "doc_overlap".to_string(),
            PathBuf::from("/test/overlap.txt"),
            60, 80, "Texte séparé sans overlap".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        manager.add_span(span1).unwrap();
        manager.add_span(span2).unwrap(); 
        manager.add_span(span3).unwrap();
        
        // Test récupération par plage
        let spans_in_range = manager.get_spans_in_range("doc_overlap", 15, 35);
        println!("✅ Spans dans plage [15, 35]: {}", spans_in_range.len());
        assert_eq!(spans_in_range.len(), 2, "❌ Devrait trouver 2 spans en overlap");
        
        let spans_in_range_narrow = manager.get_spans_in_range("doc_overlap", 65, 75);
        println!("✅ Spans dans plage [65, 75]: {}", spans_in_range_narrow.len());
        assert_eq!(spans_in_range_narrow.len(), 1, "❌ Devrait trouver 1 span séparé");
        
        let spans_outside = manager.get_spans_in_range("doc_overlap", 90, 100);
        println!("✅ Spans dans plage [90, 100]: {}", spans_outside.len());
        assert_eq!(spans_outside.len(), 0, "❌ Devrait ne trouver aucun span");
    }
    
    #[test]
    fn test_span_validation() {
        println!("🧪 Test validation des spans");
        
        let mut manager = SourceSpanManager::new();
        
        // Test span invalide : start >= end
        let invalid_span = SourceSpan::new(
            "doc_invalid".to_string(),
            PathBuf::from("/test/invalid.txt"),
            50, 30, // start > end !
            "Contenu invalide".to_string(),
            ExtractionMethod::DirectRead,
        );
        
        let result = manager.add_span(invalid_span);
        assert!(result.is_err(), "❌ Devrait rejeter span invalide");
        println!("✅ Span invalide correctement rejeté: {:?}", result.err());
        
        // Test span vide
        let empty_span = SourceSpan::new(
            "doc_empty".to_string(),
            PathBuf::from("/test/empty.txt"),
            10, 20,
            "".to_string(), // contenu vide !
            ExtractionMethod::DirectRead,
        );
        
        let result_empty = manager.add_span(empty_span);
        assert!(result_empty.is_err(), "❌ Devrait rejeter span avec contenu vide");
        println!("✅ Span vide correctement rejeté: {:?}", result_empty.err());
    }
}