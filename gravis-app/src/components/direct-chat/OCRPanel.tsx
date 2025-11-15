// OCRPanel - Panel droit avec SimplePdfViewer

import SimplePdfViewer from '@/components/SimplePdfViewer';

interface OCRPanelProps {
  sessionId: string;
  onTextAction?: (action: 'explain' | 'summarize', text: string) => void;
}

export function OCRPanel({
  sessionId,
  onTextAction
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
      <SimplePdfViewer
        sessionId={sessionId}
        onTextAction={onTextAction}
      />
    </div>
  );
}
