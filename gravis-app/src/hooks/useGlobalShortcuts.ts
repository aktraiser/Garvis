// GRAVIS AWCS - Hook pour les raccourcis globaux
// Phase 4: Gestion des événements de raccourcis système

import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface UseGlobalShortcutsReturn {
  isRegistered: boolean;
  isListening: boolean;
  error: string | null;
  lastTriggered: Date | null;
  registerShortcut: (shortcut: string) => Promise<void>;
  unregisterShortcut: (shortcut: string) => Promise<void>;
  setupAWCSShortcut: () => Promise<void>;
  cleanupShortcuts: () => Promise<void>;
}

export const useGlobalShortcuts = (): UseGlobalShortcutsReturn => {
  const [isRegistered, setIsRegistered] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastTriggered, setLastTriggered] = useState<Date | null>(null);

  // Gestionnaire d'événement de raccourci global
  const handleShortcutTriggered = useCallback(async (event: any) => {
    console.log('AWCS Phase 4: Global shortcut triggered!', event);
    setLastTriggered(new Date());
    
    try {
      // Déclencher automatiquement l'extraction AWCS
      console.log('AWCS Phase 4: Starting automatic context extraction...');
      
      // Ici on peut appeler directement l'extraction ou émettre un événement personnalisé
      // Pour l'instant, on émet un événement personnalisé que l'interface AWCS peut écouter
      window.dispatchEvent(new CustomEvent('awcs-global-shortcut-triggered', {
        detail: { timestamp: new Date(), source: 'global-shortcut' }
      }));
      
    } catch (err) {
      console.error('AWCS Phase 4: Error handling shortcut:', err);
      setError(`Failed to handle shortcut: ${err}`);
    }
  }, []);

  // Enregistrer un raccourci global (côté Rust seulement pour l'instant)
  const registerShortcut = useCallback(async (shortcut: string) => {
    try {
      setError(null);
      console.log('AWCS Phase 4: Registering global shortcut via Rust backend:', shortcut);
      
      // Pour l'instant, on utilise uniquement l'approche côté Rust
      // Le raccourci sera enregistré via awcs_setup_global_shortcut
      
      setIsRegistered(true);
      console.log('AWCS Phase 4: Global shortcut registration completed');
      
    } catch (err) {
      const errorMsg = `Failed to register shortcut: ${err}`;
      console.error('AWCS Phase 4:', errorMsg);
      setError(errorMsg);
      setIsRegistered(false);
    }
  }, []);

  // Désactiver un raccourci global
  const unregisterShortcut = useCallback(async (shortcut: string) => {
    try {
      setError(null);
      console.log('AWCS Phase 4: Unregistering global shortcut via Rust backend:', shortcut);
      
      // Géré côté Rust via awcs_cleanup
      
      setIsRegistered(false);
      console.log('AWCS Phase 4: Global shortcut unregistered successfully');
      
    } catch (err) {
      const errorMsg = `Failed to unregister shortcut: ${err}`;
      console.error('AWCS Phase 4:', errorMsg);
      setError(errorMsg);
    }
  }, []);

  // Configuration du raccourci AWCS par défaut (Cmd+Shift+G)
  const setupAWCSShortcut = useCallback(async () => {
    try {
      setError(null);
      console.log('AWCS Phase 4: Setting up AWCS global shortcut...');
      
      // Appeler la commande Tauri pour configurer côté backend
      await invoke('awcs_setup_global_shortcut');
      
      // Marquer comme enregistré (backend gère le raccourci réel)
      setIsRegistered(true);
      
      console.log('AWCS Phase 4: AWCS global shortcut setup completed');
      
    } catch (err) {
      const errorMsg = `Failed to setup AWCS shortcut: ${err}`;
      console.error('AWCS Phase 4:', errorMsg);
      setError(errorMsg);
    }
  }, [registerShortcut]);

  // Nettoyage de tous les raccourcis
  const cleanupShortcuts = useCallback(async () => {
    try {
      setError(null);
      console.log('AWCS Phase 4: Cleaning up global shortcuts...');
      
      // Nettoyage côté backend
      await invoke('awcs_cleanup');
      
      // Marquer comme non enregistré (backend gère le nettoyage)
      setIsRegistered(false);
      
      console.log('AWCS Phase 4: Global shortcuts cleanup completed');
      
    } catch (err) {
      const errorMsg = `Failed to cleanup shortcuts: ${err}`;
      console.error('AWCS Phase 4:', errorMsg);
      setError(errorMsg);
    }
  }, [unregisterShortcut]);

  // Setup initial des listeners
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        setIsListening(true);
        
        // Écouter les événements émis par notre handler backend
        unlisten = await listen('awcs-shortcut-triggered', (event) => {
          console.log('AWCS Phase 4: Global shortcut event received!', event);
          handleShortcutTriggered(event as any);
        });
        
        console.log('AWCS Phase 4: Global shortcut listeners setup completed');
        
      } catch (err) {
        console.error('AWCS Phase 4: Failed to setup listeners:', err);
        setError(`Failed to setup listeners: ${err}`);
        setIsListening(false);
      }
    };

    setupListeners();

    // Nettoyage
    return () => {
      if (unlisten) {
        unlisten();
      }
      setIsListening(false);
    };
  }, [handleShortcutTriggered]);

  return {
    isRegistered,
    isListening,
    error,
    lastTriggered,
    registerShortcut,
    unregisterShortcut,
    setupAWCSShortcut,
    cleanupShortcuts,
  };
};