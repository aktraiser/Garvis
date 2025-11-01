// GRAVIS AWCS - React Hook
// Hook principal pour l'intégration AWCS dans React

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  AWCSActivationState,
  AWCSPermissions,
  ContextEnvelope,
  TaskResult,
  // AWCSError,
  // AWCSErrorCode,
  // IntentionResult,
  AWCSConfig,
  AWCSMetrics,
  DEFAULT_AWCS_CONFIG,
} from '../types/awcs';

/// Interface du hook useAWCS
export interface UseAWCSReturn {
  // État
  state: AWCSActivationState;
  permissions: AWCSPermissions | null;
  config: AWCSConfig;
  metrics: AWCSMetrics | null;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  activateAWCS: () => Promise<void>;
  deactivateAWCS: () => Promise<void>;
  testCurrentWindow: () => Promise<ContextEnvelope | null>;
  checkPermissions: () => Promise<AWCSPermissions>;
  requestPermissions: () => Promise<void>;
  updateConfig: (config: Partial<AWCSConfig>) => Promise<void>;
  
  // Utilitaires
  isActive: boolean;
  hasRequiredPermissions: boolean;
  clearError: () => void;
}

/// Hook principal AWCS
export function useAWCS(): UseAWCSReturn {
  // État local
  const [state, setState] = useState<AWCSActivationState>(AWCSActivationState.Disabled);
  const [permissions, setPermissions] = useState<AWCSPermissions | null>(null);
  const [config, setConfig] = useState<AWCSConfig>(DEFAULT_AWCS_CONFIG);
  const [metrics, setMetrics] = useState<AWCSMetrics | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // États dérivés
  const isActive = state === AWCSActivationState.Active;
  const hasRequiredPermissions = permissions?.accessibility && permissions?.automation || false;
  
  // === Initialisation ===
  
  useEffect(() => {
    initializeAWCS();
    setupEventListeners();
    
    return () => {
      // Cleanup sera fait par les event listeners
    };
  }, []);
  
  const initializeAWCS = async () => {
    try {
      setIsLoading(true);
      
      // Charger l'état actuel
      const currentState = await invoke<AWCSActivationState>('awcs_get_state');
      setState(currentState);
      
      // Charger les permissions si nécessaire
      if (currentState !== AWCSActivationState.Disabled) {
        const currentPermissions = await invoke<AWCSPermissions>('awcs_check_permissions');
        setPermissions(currentPermissions);
      }
      
    } catch (err) {
      setError(`Échec d'initialisation AWCS: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };
  
  const setupEventListeners = async () => {
    const unlisteners: UnlistenFn[] = [];
    
    try {
      // Écouter les changements d'état
      const stateUnlisten = await listen<{ state: AWCSActivationState }>('awcs-state-changed', (event) => {
        setState(event.payload.state);
      });
      unlisteners.push(stateUnlisten);
      
      // Écouter les changements de permissions
      const permissionsUnlisten = await listen<{ permissions: AWCSPermissions }>('awcs-permissions-changed', (event) => {
        setPermissions(event.payload.permissions);
      });
      unlisteners.push(permissionsUnlisten);
      
      // Écouter les erreurs
      const errorUnlisten = await listen<{ error: string }>('awcs-error', (event) => {
        setError(event.payload.error);
        setState(AWCSActivationState.Error);
      });
      unlisteners.push(errorUnlisten);
      
    } catch (err) {
      console.error('Failed to setup AWCS event listeners:', err);
    }
    
    // Cleanup function
    return () => {
      unlisteners.forEach(unlisten => unlisten());
    };
  };
  
  // === Actions principales ===
  
  const activateAWCS = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      setState(AWCSActivationState.PermissionsPending);
      
      // 1. Vérifier les permissions
      const currentPermissions = await invoke<AWCSPermissions>('awcs_check_permissions');
      setPermissions(currentPermissions);
      
      if (!currentPermissions.accessibility || !currentPermissions.automation) {
        // Demander les permissions manquantes
        await invoke('awcs_request_permissions');
        
        // Attendre que l'utilisateur accorde les permissions
        await waitForPermissions();
      }
      
      // 2. Test d'extraction
      const testContext = await invoke<ContextEnvelope>('awcs_get_current_context');
      if (!testContext) {
        throw new Error('Test d\'extraction échoué');
      }
      
      // 3. Setup raccourci global (optionnel - ne fait pas échouer l'activation)
      try {
        await invoke('awcs_setup_global_shortcut');
      } catch (shortcutError) {
        console.warn('Setup raccourcis globaux échoué (non critique):', shortcutError);
      }
      
      // 4. Activation finale
      setState(AWCSActivationState.Active);
      
      // 5. Charger les métriques
      const currentMetrics = await invoke<AWCSMetrics>('awcs_get_metrics');
      setMetrics(currentMetrics);
      
    } catch (err) {
      setError(`Activation AWCS échouée: ${err}`);
      setState(AWCSActivationState.Error);
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const deactivateAWCS = useCallback(async () => {
    try {
      setIsLoading(true);
      await invoke('awcs_cleanup');
      setState(AWCSActivationState.Disabled);
      setPermissions(null);
      setMetrics(null);
    } catch (err) {
      setError(`Désactivation échouée: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const testCurrentWindow = useCallback(async (): Promise<ContextEnvelope | null> => {
    try {
      setError(null);
      // La commande awcs_get_current_context inclut déjà le délai de 2 secondes
      const context = await invoke<ContextEnvelope>('awcs_get_current_context');
      
      // Toast de succès sera géré par le composant
      return context;
      
    } catch (err) {
      setError(`Test échoué: ${err}`);
      return null;
    }
  }, []);
  
  const checkPermissions = useCallback(async (): Promise<AWCSPermissions> => {
    try {
      const currentPermissions = await invoke<AWCSPermissions>('awcs_check_permissions');
      setPermissions(currentPermissions);
      return currentPermissions;
    } catch (err) {
      setError(`Vérification permissions échouée: ${err}`);
      throw err;
    }
  }, []);
  
  const requestPermissions = useCallback(async () => {
    try {
      setIsLoading(true);
      await invoke('awcs_request_permissions');
      
      // Ouvrir les préférences système
      await invoke('awcs_open_system_preferences');
      
    } catch (err) {
      setError(`Demande permissions échouée: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const updateConfig = useCallback(async (newConfig: Partial<AWCSConfig>) => {
    try {
      const updatedConfig = { ...config, ...newConfig };
      await invoke('awcs_update_config', { config: updatedConfig });
      setConfig(updatedConfig);
    } catch (err) {
      setError(`Mise à jour configuration échouée: ${err}`);
    }
  }, [config]);
  
  // === Utilitaires ===
  
  const clearError = useCallback(() => {
    setError(null);
  }, []);
  
  const waitForPermissions = async (): Promise<void> => {
    return new Promise((resolve, reject) => {
      const checkInterval = setInterval(async () => {
        try {
          const currentPermissions = await checkPermissions();
          
          if (currentPermissions.accessibility && currentPermissions.automation) {
            clearInterval(checkInterval);
            resolve();
          }
        } catch (err) {
          clearInterval(checkInterval);
          reject(err);
        }
      }, 1000);
      
      // Timeout après 30 secondes
      setTimeout(() => {
        clearInterval(checkInterval);
        reject(new Error('Timeout: permissions non accordées dans les temps'));
      }, 30000);
    });
  };
  
  return {
    // État
    state,
    permissions,
    config,
    metrics,
    isLoading,
    error,
    
    // Actions
    activateAWCS,
    deactivateAWCS,
    testCurrentWindow,
    checkPermissions,
    requestPermissions,
    updateConfig,
    
    // Utilitaires
    isActive,
    hasRequiredPermissions,
    clearError,
  };
}

/// Hook pour la palette AWCS (raccourci global)
export function useAWCSPalette() {
  const [isOpen, setIsOpen] = useState(false);
  const [context, setContext] = useState<ContextEnvelope | null>(null);
  const [query, setQuery] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [result, setResult] = useState<TaskResult | null>(null);
  
  useEffect(() => {
    const setupGlobalShortcut = async () => {
      try {
        // Écouter l'événement de raccourci global
        const unlisten = await listen('awcs-shortcut-triggered', async () => {
          await handleShortcutActivation();
        });
        
        return unlisten;
      } catch (err) {
        console.error('Failed to setup global shortcut listener:', err);
      }
    };
    
    const cleanup = setupGlobalShortcut();
    
    return () => {
      cleanup.then(unlisten => unlisten?.());
    };
  }, []);
  
  const handleShortcutActivation = async () => {
    try {
      // Extraction automatique du contexte
      const currentContext = await invoke<ContextEnvelope>('awcs_get_current_context');
      setContext(currentContext);
      setIsOpen(true);
      setQuery('');
      setResult(null);
    } catch (err) {
      console.error('Failed to extract context on shortcut:', err);
    }
  };
  
  const handleQuery = async (queryText: string) => {
    if (!queryText.trim() || !context) return;
    
    setIsProcessing(true);
    try {
      const taskResult = await invoke<TaskResult>('awcs_handle_query', {
        query: queryText,
        context
      });
      
      setResult(taskResult);
      
    } catch (err) {
      console.error('Query processing failed:', err);
    } finally {
      setIsProcessing(false);
    }
  };
  
  const closePalette = () => {
    setIsOpen(false);
    setQuery('');
    setResult(null);
    setContext(null);
  };
  
  return {
    isOpen,
    context,
    query,
    setQuery,
    result,
    isProcessing,
    handleQuery,
    closePalette,
  };
}