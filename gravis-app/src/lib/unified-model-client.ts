// Client unifié pour récupérer les modèles de toutes les connexions actives
import { LiteLLMClient, LLMModel, modelConfigStore } from './litellm';
import { localModelDetector } from './local-models';

export interface UnifiedModelResponse {
  models: LLMModel[];
  sources: Array<{
    name: string;
    type: string;
    baseUrl: string;
    modelCount: number;
    isAvailable: boolean;
  }>;
}

export class UnifiedModelClient {
  
  async getAllAvailableModels(): Promise<UnifiedModelResponse> {
    const allModels: LLMModel[] = [];
    const sources: Array<{
      name: string;
      type: string;
      baseUrl: string;
      modelCount: number;
      isAvailable: boolean;
    }> = [];

    // 1. Récupérer les modèles depuis les connexions actives
    const activeConnections = modelConfigStore.activeConnections;
    
    for (const connection of activeConnections) {
      try {
        let connectionModels: LLMModel[] = [];
        let isAvailable = false;

        if (connection.type === 'ollama') {
          // Récupérer les modèles Ollama
          const provider = await localModelDetector.detectOllamaModels();
          if (provider.isAvailable) {
            connectionModels = provider.models;
            isAvailable = true;
          }
        } else if (connection.type === 'localai') {
          // Récupérer les modèles LocalAI
          const provider = await localModelDetector.detectLocalAIModels();
          if (provider.isAvailable) {
            connectionModels = provider.models;
            isAvailable = true;
          }
        } else if (connection.type === 'litellm' || !connection.type) {
          // Récupérer les modèles LiteLLM
          const client = new LiteLLMClient({
            apiKey: connection.apiKey,
            baseUrl: connection.baseUrl,
            model: 'test'
          });
          
          try {
            const response = await client.getModels();
            if (response.data && Array.isArray(response.data)) {
              connectionModels = response.data.map((model: any): LLMModel => ({
                id: model.id || model.name,
                name: model.name || model.id,
                provider: connection.name,
                description: `From ${connection.name}`,
                contextWindow: model.context_window || 4096,
                capabilities: model.capabilities || []
              }));
              isAvailable = true;
            }
          } catch (error) {
            console.warn(`Failed to fetch models from ${connection.name}:`, error);
          }
        } else if (connection.type === 'ibm') {
          // Modèles IBM Watson prédéfinis
          connectionModels = [
            {
              id: `ibm/${connection.name.toLowerCase().replace(/\s+/g, '-')}`,
              name: connection.name,
              provider: 'IBM Watson',
              description: 'IBM Watson AI service',
              contextWindow: 8192,
              capabilities: ['tools', 'reasoning']
            }
          ];
          isAvailable = true;
        }

        // Ajouter le préfixe de connexion aux IDs des modèles
        const prefixedModels = connectionModels.map(model => ({
          ...model,
          id: model.id.startsWith(connection.type || 'custom') ? model.id : `${connection.type || 'custom'}/${model.id}`,
          provider: connection.name
        }));

        allModels.push(...prefixedModels);

        sources.push({
          name: connection.name,
          type: connection.type || 'custom',
          baseUrl: connection.baseUrl,
          modelCount: connectionModels.length,
          isAvailable
        });

      } catch (error) {
        console.error(`Error fetching models from ${connection.name}:`, error);
        sources.push({
          name: connection.name,
          type: connection.type || 'custom',
          baseUrl: connection.baseUrl,
          modelCount: 0,
          isAvailable: false
        });
      }
    }

    // 2. Ajouter les modèles par défaut si aucune connexion active
    if (allModels.length === 0) {
      // Fallback vers les modèles statiques
      const { AVAILABLE_MODELS } = await import('./litellm');
      allModels.push(...AVAILABLE_MODELS);
      
      sources.push({
        name: 'Modèles par défaut',
        type: 'default',
        baseUrl: 'built-in',
        modelCount: AVAILABLE_MODELS.length,
        isAvailable: true
      });
    }

    return {
      models: allModels,
      sources
    };
  }

  // Créer un client pour un modèle spécifique
  createClientForModel(modelId: string): LiteLLMClient | null {
    const activeConnections = modelConfigStore.activeConnections;
    
    // Trouver la connexion qui correspond au modèle
    for (const connection of activeConnections) {
      const prefix = connection.type || 'custom';
      if (modelId.startsWith(`${prefix}/`)) {
        return new LiteLLMClient({
          apiKey: connection.apiKey,
          baseUrl: connection.baseUrl,
          model: modelId.replace(`${prefix}/`, '')
        });
      }
    }

    // Fallback vers la configuration par défaut
    return new LiteLLMClient(modelConfigStore.getConfig());
  }
}

export const unifiedModelClient = new UnifiedModelClient();