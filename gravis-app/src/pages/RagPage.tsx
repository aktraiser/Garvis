import React from 'react';
import { RagWindow } from '../components/RagWindow';

export const RagPage: React.FC = () => {
  const handleClose = () => {
    // Pour une nouvelle fenêtre, on ferme la fenêtre Tauri
    import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
      getCurrentWindow().close();
    });
  };

  return <RagWindow onClose={handleClose} />;
};