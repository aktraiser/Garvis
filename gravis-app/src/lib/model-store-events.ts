import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { modelConfigStore, LLMModel } from './litellm';

export class ModelStoreEvents {
  private listeners: UnlistenFn[] = [];

  // Émettre un changement de modèle vers toutes les fenêtres
  async emitModelChanged(model: LLMModel) {
    try {
      await invoke('emit_model_changed', { model });
      console.log('Model change event emitted:', model);
    } catch (error) {
      console.error('Failed to emit model change:', error);
    }
  }

  // Écouter les changements de modèle
  async listenForModelChanges(callback: (model: LLMModel) => void) {
    try {
      const unlisten = await listen<LLMModel>('model_changed', (event) => {
        console.log('Received model_changed event:', event.payload);
        // Mettre à jour le store local
        modelConfigStore.setModel(event.payload);
        callback(event.payload);
      });
      
      this.listeners.push(unlisten);
      return unlisten;
    } catch (error) {
      console.error('Failed to listen for model changes:', error);
      return () => {};
    }
  }

  // Broadcaster vers une fenêtre spécifique
  async broadcastToWindow(windowLabel: string, event: string, payload: any) {
    try {
      await invoke('broadcast_to_window', { 
        windowLabel, 
        event, 
        payload 
      });
      console.log(`Broadcasted ${event} to ${windowLabel}:`, payload);
    } catch (error) {
      console.error(`Failed to broadcast ${event} to ${windowLabel}:`, error);
    }
  }

  // Nettoyer tous les listeners
  cleanup() {
    this.listeners.forEach(unlisten => unlisten());
    this.listeners = [];
  }
}

export const modelStoreEvents = new ModelStoreEvents();