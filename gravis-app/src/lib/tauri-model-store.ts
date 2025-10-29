// Communication inter-fenêtres robuste pour Tauri en production
// Utilise les événements Tauri natifs au lieu de BroadcastChannel
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { modelConfigStore, type LLMModel } from './litellm';

export class TauriModelStore {
  private listeners: Set<(model: LLMModel) => void> = new Set();
  private unlisteners: UnlistenFn[] = [];
  private isInitialized = false;

  constructor() {
    this.initialize();
  }

  private async initialize() {
    if (this.isInitialized) return;

    try {
      // Écouter les événements model_changed de toutes les fenêtres
      const unlisten = await listen<LLMModel>('model_changed', (event) => {
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

      this.unlisteners.push(unlisten);
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