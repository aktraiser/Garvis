// OCRPanel - Panel droit avec le viewer OCR et highlighting des spans

import { OCRViewerWithSpans, type SourceSpan, type OCRContent } from '@/components/OCRViewerWithSpans';

interface OCRPanelProps {
  documentName: string;
  ocrContent: OCRContent;
  highlightedSpans: SourceSpan[];
  onSpanClick?: (span: SourceSpan) => void;
  onTextSelection?: (text: string) => void;
}

export function OCRPanel({
  documentName,
  ocrContent,
  highlightedSpans,
  onSpanClick,
  onTextSelection
}: OCRPanelProps) {
  return (
    <div style={{
      position: 'fixed',
      right: 0,
      top: 0,
      bottom: 0,
      width: '50%',
      backgroundColor: '#1f2937',
      borderLeft: '1px solid #374151',
      zIndex: 999,
      overflow: 'hidden',
    }}>
      <OCRViewerWithSpans
        documentName={documentName}
        ocrContent={ocrContent}
        highlightedSpans={highlightedSpans}
        onSpanClick={onSpanClick}
        onTextSelection={onTextSelection}
      />
    </div>
  );
}
