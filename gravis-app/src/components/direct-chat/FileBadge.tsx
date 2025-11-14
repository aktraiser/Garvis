// FileBadge - Badge affichant le fichier déposé avec bouton de suppression

import { FileText } from 'lucide-react';

interface FileBadgeProps {
  fileName: string;
  onRemove: () => void;
}

export function FileBadge({ fileName, onRemove }: FileBadgeProps) {
  return (
    <div style={{
      fontSize: '11px',
      color: '#3b82f6',
      textAlign: 'left',
      marginTop: '4px',
      display: 'flex',
      alignItems: 'center',
      gap: '6px',
      backgroundColor: 'rgba(59, 130, 246, 0.1)',
      padding: '6px 8px',
      borderRadius: '4px',
    }}>
      <FileText size={12} />
      <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {fileName}
      </span>
      <button
        type="button"
        onClick={onRemove}
        style={{
          background: 'none',
          border: 'none',
          color: '#3b82f6',
          cursor: 'pointer',
          padding: '2px',
          fontSize: '14px',
        }}
        title="Supprimer le document"
      >
        ×
      </button>
    </div>
  );
}
