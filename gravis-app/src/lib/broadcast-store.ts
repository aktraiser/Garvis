// Communication inter-fenêtres avec BroadcastChannel
import { LLMModel, modelConfigStore } from './litellm';

class BroadcastModelStore {
  private channel: BroadcastChannel;
  private listeners: Set<(model: LLMModel) => void> = new Set();

  constructor() {
    this.channel = new BroadcastChannel('model-changes');
    
    // Écouter les messages de ce canal
    this.channel.onmessage = (event) => {
      console.log('Received broadcast message:', event.data);
      
      if (event.data.type === 'model_changed') {
        const model = event.data.model;
        
        // Mettre à jour le store local
        modelConfigStore.setModel(model);
        
        // Notifier tous les listeners
        this.listeners.forEach(listener => {
          try {
            listener(model);
          } catch (error) {
            console.error('Error in model change listener:', error);
          }
        });
      }
    };
  }

  // Émettre un changement de modèle
  emitModelChanged(model: LLMModel) {
    console.log('Broadcasting model change:', model);
    
    // Sauvegarder d'abord localement
    modelConfigStore.setModel(model);
    
    // Puis broadcaster
    this.channel.postMessage({
      type: 'model_changed',
      model: model,
      timestamp: Date.now()
    });
  }

  // S'abonner aux changements de modèle
  onModelChanged(callback: (model: LLMModel) => void): () => void {
    this.listeners.add(callback);
    
    // Retourner une fonction de nettoyage
    return () => {
      this.listeners.delete(callback);
    };
  }

  // Nettoyer les ressources
  cleanup() {
    this.channel.close();
    this.listeners.clear();
  }
}

export const broadcastModelStore = new BroadcastModelStore();