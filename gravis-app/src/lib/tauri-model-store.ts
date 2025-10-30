// Communication inter-fenêtres robuste pour Tauri en production
// Utilise les événements Tauri natifs au lieu de BroadcastChannel
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { modelConfigStore, type LLMModel } from './litellm';

export class TauriModelStore {
  private listeners: Set<(model: LLMModel) => void> = new Set();
  private parametersListeners: Set<(parameters: any) => void> = new Set();
  private unlisteners: UnlistenFn[] = [];
  private isInitialized = false;

  constructor() {
    this.initialize();
  }

  private async initialize() {
    if (this.isInitialized) return;

    try {
      // Écouter les événements model_changed de toutes les fenêtres
      const unlistenModel = await listen<LLMModel>('model_changed', (event) => {
        // Mettre à jour le store local
        modelConfigStore.setModel(event.payload);
        
        // Notifier tous les listeners locaux
        this.listeners.forEach(listener => {
          try {
            listener(event.payload);
          } catch (error) {
            console.error('Error in model change listener:', error);
          }
        });
      });

      // Écouter les événements parameters_changed de toutes les fenêtres
      const unlistenParameters = await listen<any>('parameters_changed', (event) => {
        console.log('🔧 TauriModelStore: Received parameters_changed event:', event.payload);
        
        // Mettre à jour le store local SILENCIEUSEMENT pour éviter la boucle
        modelConfigStore.modelParameters = {
          ...modelConfigStore.modelParameters,
          ...event.payload
        };
        modelConfigStore.save();
        
        // Notifier tous les listeners locaux
        this.parametersListeners.forEach(listener => {
          try {
            listener(event.payload);
          } catch (error) {
            console.error('Error in parameters change listener:', error);
          }
        });
      });

      this.unlisteners.push(unlistenModel, unlistenParameters);
      this.isInitialized = true;
      
    } catch (error) {
      console.error('Failed to initialize TauriModelStore:', error);
    }
  }

  // Émettre un changement de modèle vers toutes les fenêtres
  async emitModelChanged(model: LLMModel) {
    try {
      // Sauvegarder d'abord localement
      modelConfigStore.setModel(model);
      
      // Utiliser la commande Rust pour broadcaster à toutes les fenêtres
      await invoke('emit_model_changed', { model });
    } catch (error) {
      console.error('Failed to emit model change:', error);
      throw error;
    }
  }

  // Émettre un changement de paramètres vers toutes les fenêtres
  async emitParametersChanged(parameters: any) {
    try {
      console.log('🔧 TauriModelStore: Emitting parameters change:', parameters);
      
      // Utiliser la commande Rust pour émettre les paramètres
      // Les paramètres seront sauvegardés localement quand on recevra l'événement
      await invoke('emit_parameters_changed', { parameters });
      
      console.log('🔧 TauriModelStore: Parameters change broadcasted successfully');
    } catch (error) {
      console.error('Failed to emit parameters change:', error);
      // Fallback vers localStorage seulement en cas d'échec
      modelConfigStore.setModelParameters(parameters);
    }
  }

  // Broadcaster vers une fenêtre spécifique
  async emitToWindow(windowLabel: string, model: LLMModel) {
    try {
      await invoke('broadcast_to_window', {
        windowLabel,
        event: 'model_changed',
        payload: model
      });
    } catch (error) {
      console.error(`Failed to broadcast to window ${windowLabel}:`, error);
      throw error;
    }
  }

  // S'abonner aux changements de modèle
  onModelChanged(callback: (model: LLMModel) => void): () => void {
    this.listeners.add(callback);
    
    // Notifier immédiatement avec le modèle actuel
    const currentModel = modelConfigStore.currentModel;
    if (currentModel) {
      try {
        callback(currentModel);
      } catch (error) {
        console.error('Error in immediate model callback:', error);
      }
    }
    
    // Retourner une fonction de nettoyage
    return () => {
      this.listeners.delete(callback);
    };
  }

  // S'abonner aux changements de paramètres
  onParametersChanged(callback: (parameters: any) => void): () => void {
    this.parametersListeners.add(callback);
    
    // Notifier immédiatement avec les paramètres actuels
    const currentParameters = modelConfigStore.modelParameters;
    if (currentParameters) {
      try {
        callback(currentParameters);
      } catch (error) {
        console.error('Error in immediate parameters callback:', error);
      }
    }
    
    // Retourner une fonction de nettoyage
    return () => {
      this.parametersListeners.delete(callback);
    };
  }

  // Vérifier si une fenêtre existe
  async checkWindowExists(windowLabel: string): Promise<boolean> {
    try {
      await invoke('broadcast_to_window', {
        windowLabel,
        event: 'ping',
        payload: { timestamp: Date.now() }
      });
      return true;
    } catch (error) {
      return false;
    }
  }

  // Nettoyer les ressources
  cleanup() {
    this.unlisteners.forEach(unlisten => unlisten());
    this.unlisteners = [];
    this.listeners.clear();
    this.parametersListeners.clear();
    this.isInitialized = false;
  }

  // Diagnostics pour débuggage
  async getDiagnostics() {
    return {
      isInitialized: this.isInitialized,
      listenersCount: this.listeners.size,
      unlistenersCount: this.unlisteners.length,
      currentModel: modelConfigStore.currentModel,
      timestamp: new Date().toISOString()
    };
  }
}

// Instance singleton
export const tauriModelStore = new TauriModelStore();

// Nettoyage automatique lors de la fermeture de la fenêtre
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    tauriModelStore.cleanup();
  });
}