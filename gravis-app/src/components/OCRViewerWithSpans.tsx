// OCRViewerWithSpans - Composant pour afficher un document avec highlighting des source spans
// PR #4 Phase 2 - Interface Avancée

import React, { useState, useEffect, useRef } from 'react';
import './OCRViewerWithSpans.css';

// Types pour les source spans
export interface BoundingBox {
  page: number | null;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number | null;
  coordinate_system: 'PdfPoints' | 'PixelTopLeft' | 'PixelBottomLeft';
}

export interface SourceSpan {
  span_id: string;
  document_id: string;
  document_path: string;
  char_start: number;
  char_end: number;
  line_start: number;
  line_end: number;
  bbox: BoundingBox | null;
  original_content: string;
  extraction_metadata: {
    method: string;
    confidence: number;
    language: string | null;
    method_specific: Record<string, any>;
    content_hash: string;
  };
  created_at: string;
}

// Backend OCR structures
export interface OCRContent {
  pages: OCRPage[];
  total_confidence: number;
  layout_analysis: LayoutAnalysis;
}

export interface OCRPage {
  page_number: number;
  blocks: OCRBlock[];
  width: number;
  height: number;
}

export interface OCRBlock {
  block_type: 'Text' | 'Header' | 'Table' | 'List' | 'KeyValue' | 'Amount' | 'Date';
  content: string;
  bounding_box: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  confidence: number;
  spans: string[];
}

export interface LayoutAnalysis {
  detected_columns: number;
  has_tables: boolean;
  has_headers: boolean;
  text_density: number;
  dominant_font_size: number | null;
}

interface OCRViewerWithSpansProps {
  documentName: string;
  ocrContent: OCRContent;
  highlightedSpans: SourceSpan[];
  onSpanClick?: (span: SourceSpan) => void;
  onTextSelection?: (selectedText: string, bbox: BoundingBox | null) => void;
}

export const OCRViewerWithSpans: React.FC<OCRViewerWithSpansProps> = ({
  documentName,
  ocrContent,
  highlightedSpans,
  onSpanClick,
  onTextSelection,
}) => {
  const [hoveredSpanId, setHoveredSpanId] = useState<string | null>(null);
  const [selectedSpanId, setSelectedSpanId] = useState<string | null>(null);
  const viewerRef = useRef<HTMLDivElement>(null);

  // Afficher le texte complet avec overlays pour les spans
  const renderDocumentWithHighlights = () => {
    if (!ocrContent.pages || ocrContent.pages.length === 0) {
      return <div className="ocr-empty">Aucun contenu OCR disponible</div>;
    }

    // Extraire tout le texte des blocks
    const fullText = ocrContent.pages
      .flatMap(page => page.blocks)
      .map(block => block.content)
      .join('\n');

    return (
      <div className="ocr-document-container">
        {/* Texte de base */}
        <div className="ocr-text-layer">
          {fullText.split('\n').map((line: string, idx: number) => (
            <div key={`line-${idx}`} className="ocr-line">
              {line || '\u00A0'}
            </div>
          ))}
        </div>

        {/* Overlays pour les spans (highlighting) */}
        <div className="ocr-highlight-layer">
          {highlightedSpans.map((span) => {
            if (!span.bbox) return null;

            const isHovered = hoveredSpanId === span.span_id;
            const isSelected = selectedSpanId === span.span_id;

            // Calculer la position du highlight basée sur le bbox
            const style: React.CSSProperties = {
              position: 'absolute',
              left: `${span.bbox.x}px`,
              top: `${span.bbox.y}px`,
              width: `${span.bbox.width}px`,
              height: `${span.bbox.height}px`,
              backgroundColor: isSelected
                ? 'rgba(59, 130, 246, 0.3)'
                : isHovered
                ? 'rgba(251, 191, 36, 0.3)'
                : 'rgba(34, 197, 94, 0.2)',
              border: isSelected
                ? '2px solid #3b82f6'
                : isHovered
                ? '2px solid #fbbf24'
                : '1px solid rgba(34, 197, 94, 0.4)',
              borderRadius: '4px',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
              pointerEvents: 'all',
              zIndex: isSelected ? 30 : isHovered ? 20 : 10,
            };

            return (
              <div
                key={span.span_id}
                className="ocr-span-highlight"
                style={style}
                onMouseEnter={() => setHoveredSpanId(span.span_id)}
                onMouseLeave={() => setHoveredSpanId(null)}
                onClick={() => {
                  setSelectedSpanId(span.span_id);
                  if (onSpanClick) {
                    onSpanClick(span);
                  }
                }}
                title={`Span: ${span.span_id}\nConfiance: ${Math.round(span.extraction_metadata.confidence * 100)}%\nLignes: ${span.line_start}-${span.line_end}`}
              />
            );
          })}
        </div>
      </div>
    );
  };

  // Gérer la sélection de texte par l'utilisateur
  useEffect(() => {
    const handleTextSelection = () => {
      const selection = window.getSelection();
      if (!selection || selection.toString().trim() === '') return;

      const selectedText = selection.toString().trim();

      // TODO: Calculer le bbox de la sélection utilisateur
      // Pour le MVP, on envoie null pour le bbox
      if (onTextSelection) {
        onTextSelection(selectedText, null);
      }
    };

    const viewer = viewerRef.current;
    if (viewer) {
      viewer.addEventListener('mouseup', handleTextSelection);
    }

    return () => {
      if (viewer) {
        viewer.removeEventListener('mouseup', handleTextSelection);
      }
    };
  }, [onTextSelection]);

  return (
    <div className="ocr-viewer-with-spans" ref={viewerRef}>
      {/* Header */}
      <div className="ocr-viewer-header">
        <div className="ocr-document-title">
          <span className="ocr-document-icon">📄</span>
          <span className="ocr-document-name">{documentName}</span>
        </div>
        <div className="ocr-document-stats">
          <span className="ocr-stat">{ocrContent.pages.length} pages</span>
          <span className="ocr-stat-separator">•</span>
          <span className="ocr-stat">{ocrContent.pages.flatMap(p => p.blocks).length} blocs</span>
          <span className="ocr-stat-separator">•</span>
          <span className="ocr-stat">{highlightedSpans.length} spans</span>
        </div>
      </div>

      {/* Document content with highlights */}
      <div className="ocr-viewer-content">
        {renderDocumentWithHighlights()}
      </div>

      {/* Selected span details */}
      {selectedSpanId && (() => {
        const selectedSpan = highlightedSpans.find(s => s.span_id === selectedSpanId);
        if (!selectedSpan) return null;

        return (
          <div className="ocr-span-details">
            <div className="ocr-span-details-header">
              <span className="ocr-span-details-title">Span sélectionné</span>
              <button
                className="ocr-span-details-close"
                onClick={() => setSelectedSpanId(null)}
              >
                ×
              </button>
            </div>
            <div className="ocr-span-details-content">
              <div className="ocr-span-detail-row">
                <span className="ocr-span-detail-label">ID:</span>
                <span className="ocr-span-detail-value">{selectedSpan.span_id}</span>
              </div>
              <div className="ocr-span-detail-row">
                <span className="ocr-span-detail-label">Lignes:</span>
                <span className="ocr-span-detail-value">{selectedSpan.line_start} - {selectedSpan.line_end}</span>
              </div>
              <div className="ocr-span-detail-row">
                <span className="ocr-span-detail-label">Confiance:</span>
                <span className="ocr-span-detail-value">{Math.round(selectedSpan.extraction_metadata.confidence * 100)}%</span>
              </div>
              <div className="ocr-span-detail-row">
                <span className="ocr-span-detail-label">Méthode:</span>
                <span className="ocr-span-detail-value">{selectedSpan.extraction_metadata.method}</span>
              </div>
              {selectedSpan.extraction_metadata.method_specific.relevance_score && (
                <div className="ocr-span-detail-row">
                  <span className="ocr-span-detail-label">Pertinence:</span>
                  <span className="ocr-span-detail-value">
                    {Math.round(parseFloat(selectedSpan.extraction_metadata.method_specific.relevance_score) * 100)}%
                  </span>
                </div>
              )}
              <div className="ocr-span-content">
                <div className="ocr-span-content-label">Contenu:</div>
                <div className="ocr-span-content-text">
                  {selectedSpan.original_content}
                </div>
              </div>
            </div>
          </div>
        );
      })()}
    </div>
  );
};
