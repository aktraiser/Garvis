// DragOverlay - Border bleu autour de la fenêtre pendant le drag & drop

export function DragOverlay() {
  return (
    <div style={{
      position: 'absolute',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      border: '3px solid #3b82f6',
      borderRadius: '8px',
      pointerEvents: 'none',
      zIndex: 1000,
      boxShadow: '0 0 0 4px rgba(59, 130, 246, 0.2)',
    }} />
  );
}
