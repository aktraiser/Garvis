import { ModelSelectorWindow } from '@/components/ModelSelectorWindow';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function ModelSelectorPage() {
  const handleClose = async () => {
    try {
      const window = getCurrentWindow();
      await window.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };

  return (
    <ModelSelectorWindow onClose={handleClose} />
  );
}