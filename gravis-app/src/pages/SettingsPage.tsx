import { SettingsWindow } from '@/components/SettingsWindow';

export function SettingsPage() {
  const handleClose = () => {
    // Logique de fermeture (ex: navigation, fermeture de modal, etc.)
    console.log('Settings window closed');
  };

  return (
    <SettingsWindow onClose={handleClose} />
  );
}