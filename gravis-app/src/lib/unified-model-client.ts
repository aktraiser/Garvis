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

    // 1. Récupérer les modèles depuis les connexions actives EN PARALLÈLE
    const activeConnections = modelConfigStore.activeConnections;

    // Track si Ollama a déjà été détecté pour éviter double détection
    let ollamaAlreadyDetected = false;

    // Créer toutes les promesses de détection en parallèle
    const connectionPromises = activeConnections.map(async (connection: any) => {
      try {
        let connectionModels: LLMModel[] = [];
        let isAvailable = false;

        if (connection.type === 'ollama') {
          // Récupérer les modèles Ollama
          const provider = await localModelDetector.detectOllamaModels();
          if (provider.isAvailable) {
            connectionModels = provider.models;
            isAvailable = true;
            ollamaAlreadyDetected = true; // Marquer Ollama comme déjà détecté
          }
        } else if (connection.type === 'localai') {
          // Récupérer les modèles LocalAI
          const provider = await localModelDetector.detectLocalAIModels();
          if (provider.isAvailable) {
            connectionModels = provider.models;
            isAvailable = true;
          }
        } else if (connection.type?.toLowerCase() === 'litellm' || !connection.type) {
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
        } else if (connection.type === 'Modal') {
          // Récupérer les modèles Modal via l'API compatible OpenAI
          const client = new LiteLLMClient({
            apiKey: connection.apiKey || 'not-needed',
            baseUrl: connection.baseUrl,
            model: 'llm'
          });

          try {
            const response = await client.getModels();
            if (response.data && Array.isArray(response.data)) {
              // Filtrer l'alias 'llm' - ne garder que les vrais noms de modèles
              connectionModels = response.data
                .filter((model: any) => model.id !== 'llm' && !model.id.endsWith('/llm'))
                .map((model: any): LLMModel => ({
                  id: model.id,
                  name: model.name || model.id,
                  provider: connection.name,
                  description: `Modal vLLM model from ${connection.name}`,
                  contextWindow: model.context_window || 32768,
                  capabilities: ['chat', 'streaming']
                }));

              // Si après filtrage il ne reste aucun modèle, utiliser le nom de connexion
              if (connectionModels.length === 0) {
                connectionModels = [{
                  id: connection.name.toLowerCase().replace(/\s+/g, '-'),
                  name: connection.name,
                  provider: 'Modal vLLM',
                  description: `Modal vLLM model: ${connection.name}`,
                  contextWindow: 32768,
                  capabilities: ['chat', 'streaming']
                }];
              }

              isAvailable = true;
            } else {
              // Fallback si pas de liste de modèles
              connectionModels = [{
                id: connection.name.toLowerCase().replace(/\s+/g, '-'),
                name: connection.name,
                provider: 'Modal vLLM',
                description: `Modal vLLM model: ${connection.name}`,
                contextWindow: 32768,
                capabilities: ['chat', 'streaming']
              }];
              isAvailable = true;
            }
          } catch (error) {
            console.warn(`Failed to fetch models from Modal ${connection.name}:`, error);
            // Fallback même en cas d'erreur
            connectionModels = [{
              id: connection.name.toLowerCase().replace(/\s+/g, '-'),
              name: connection.name,
              provider: 'Modal vLLM',
              description: `Modal vLLM model: ${connection.name}`,
              contextWindow: 32768,
              capabilities: ['chat', 'streaming']
            }];
            isAvailable = true;
          }
        }

        // Ajouter le préfixe de connexion aux IDs des modèles
        const prefixedModels = connectionModels.map(model => ({
          ...model,
          id: model.id.startsWith(connection.type || 'custom') ? model.id : `${connection.type || 'custom'}/${model.id}`,
          provider: connection.name
        }));

        return {
          models: prefixedModels,
          source: {
            name: connection.name,
            type: connection.type || 'custom',
            baseUrl: connection.baseUrl,
            modelCount: connectionModels.length,
            isAvailable
          },
          isOllama: connection.type === 'ollama' && isAvailable
        };

      } catch (error) {
        console.error(`Error fetching models from ${connection.name}:`, error);
        return {
          models: [],
          source: {
            name: connection.name,
            type: connection.type || 'custom',
            baseUrl: connection.baseUrl,
            modelCount: 0,
            isAvailable: false
          },
          isOllama: false
        };
      }
    });

    // Attendre toutes les connexions en parallèle
    const connectionResults = await Promise.all(connectionPromises);

    // Fusionner les résultats
    for (const result of connectionResults) {
      allModels.push(...result.models);
      sources.push(result.source);
      if (result.isOllama) {
        ollamaAlreadyDetected = true;
      }
    }

    // 1.5. Essayer de détecter Ollama automatiquement SEULEMENT si pas déjà détecté
    if (!ollamaAlreadyDetected) {
      try {
        const ollamaDetection = await localModelDetector.detectOllamaModels();
        if (ollamaDetection.isAvailable && ollamaDetection.models.length > 0) {
        // Ajouter les modèles Ollama avec préfixe
        const ollamaModels = ollamaDetection.models.map(model => ({
          ...model,
          id: model.id.startsWith('ollama/') ? model.id : `ollama/${model.id}`,
          provider: 'Ollama (Local)'
        }));
        
        allModels.push(...ollamaModels);
        
        sources.push({
          name: 'Ollama (Détecté automatiquement)',
          type: 'ollama',
          baseUrl: 'http://localhost:11434',
          modelCount: ollamaModels.length,
          isAvailable: true
        });
      }
      } catch (error) {
        console.log('Ollama auto-detection failed:', error);
      }
    }

    // 2. Ne PAS ajouter de modèles par défaut si les connexions échouent
    // Cela force l'utilisateur à vérifier et corriger ses connexions
    // Une UX honnête est préférable à un fallback silencieux trompeur

    // Si aucun modèle n'a été trouvé, retourner un tableau vide
    // L'utilisateur verra un message d'erreur clair dans l'UI
    if (allModels.length === 0) {
      console.warn('⚠️ Aucun modèle disponible depuis les connexions configurées');
      if (activeConnections.length > 0) {
        console.warn('Connexions configurées mais aucune n\'est accessible:',
          activeConnections.map((c: any) => `${c.name} (${c.type})`).join(', ')
        );
      } else {
        console.warn('Aucune connexion configurée. L\'utilisateur doit ajouter une connexion.');
      }
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