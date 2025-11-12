// Service pour détecter les modèles locaux disponibles
import { LLMModel } from './litellm';

export interface LocalModelProvider {
  name: string;
  baseUrl: string;
  isAvailable: boolean;
  models: LLMModel[];
}

export interface OllamaModel {
  name: string;
  size: number;
  digest: string;
  modified_at: string;
  details?: {
    family?: string;
    parameter_size?: string;
    quantization_level?: string;
  };
}

export class LocalModelDetector {
  private readonly TIMEOUT_MS = 3000; // 3 secondes max pour détection locale

  // Helper pour fetch avec timeout
  private async fetchWithTimeout(url: string, options: RequestInit = {}): Promise<Response> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.TIMEOUT_MS);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal
      });
      clearTimeout(timeoutId);
      return response;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error(`Timeout après ${this.TIMEOUT_MS}ms`);
      }
      throw error;
    }
  }

  // Détecter les modèles Ollama
  async detectOllamaModels(): Promise<LocalModelProvider> {
    const provider: LocalModelProvider = {
      name: 'Ollama',
      baseUrl: 'http://localhost:11434',
      isAvailable: false,
      models: []
    };

    try {
      const response = await this.fetchWithTimeout(`${provider.baseUrl}/api/tags`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      provider.isAvailable = true;

      if (data.models && Array.isArray(data.models)) {
        provider.models = data.models.map((model: OllamaModel): LLMModel => ({
          id: `ollama/${model.name}`,
          name: model.name,
          provider: 'Ollama',
          description: `Local model - ${this.formatSize(model.size)}`,
          contextWindow: this.estimateContextWindow(model.name),
          capabilities: this.getModelCapabilities(model.name)
        }));
      }
    } catch (error) {
      console.warn('Ollama not available:', error);
    }

    return provider;
  }

  // Détecter d'autres modèles locaux (placeholders pour futurs fournisseurs)
  async detectLocalAIModels(): Promise<LocalModelProvider> {
    const provider: LocalModelProvider = {
      name: 'LocalAI',
      baseUrl: 'http://localhost:8080',
      isAvailable: false,
      models: []
    };

    try {
      const response = await this.fetchWithTimeout(`${provider.baseUrl}/v1/models`);
      if (response.ok) {
        const data = await response.json();
        provider.isAvailable = true;
        
        if (data.data && Array.isArray(data.data)) {
          provider.models = data.data.map((model: any): LLMModel => ({
            id: `localai/${model.id}`,
            name: model.id,
            provider: 'LocalAI',
            description: 'Local AI model',
            contextWindow: 4096,
            capabilities: ['tools']
          }));
        }
      }
    } catch (error) {
      console.warn('LocalAI not available:', error);
    }

    return provider;
  }

  // Créer une connexion pour modèles IBM Watson
  createIBMWatsonProvider(baseUrl: string, _apiKey: string): LocalModelProvider {
    return {
      name: 'IBM Watson',
      baseUrl,
      isAvailable: true,
      models: [
        {
          id: 'ibm/watson-assistant',
          name: 'Watson Assistant',
          provider: 'IBM Watson',
          description: 'IBM Watson conversational AI',
          contextWindow: 8192,
          capabilities: ['tools', 'reasoning']
        },
        {
          id: 'ibm/watson-discovery',
          name: 'Watson Discovery',
          provider: 'IBM Watson',
          description: 'IBM Watson document analysis',
          contextWindow: 4096,
          capabilities: ['reasoning']
        }
      ]
    };
  }

  // Détecter tous les fournisseurs locaux
  async detectAllLocalProviders(): Promise<LocalModelProvider[]> {
    const providers = await Promise.all([
      this.detectOllamaModels(),
      this.detectLocalAIModels()
    ]);

    return providers.filter(provider => provider.isAvailable || provider.models.length > 0);
  }

  // Utilitaires
  private formatSize(bytes: number): string {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }

  private estimateContextWindow(modelName: string): number {
    const name = modelName.toLowerCase();
    if (name.includes('llama3') || name.includes('llama-3')) return 8192;
    if (name.includes('llama2') || name.includes('llama-2')) return 4096;
    if (name.includes('codellama')) return 8192;
    if (name.includes('mistral')) return 8192;
    if (name.includes('qwen')) return 8192;
    if (name.includes('gemma')) return 8192;
    return 4096; // default
  }

  private getModelCapabilities(modelName: string): string[] {
    const name = modelName.toLowerCase();
    const capabilities: string[] = [];
    
    if (name.includes('code') || name.includes('coder')) {
      capabilities.push('code');
    }
    if (name.includes('instruct') || name.includes('chat')) {
      capabilities.push('tools');
    }
    if (name.includes('vision') || name.includes('visual')) {
      capabilities.push('vision');
    }
    
    capabilities.push('reasoning'); // Most local models support basic reasoning
    
    return capabilities;
  }

  // Tester la connectivité d'un fournisseur
  async testProviderConnection(provider: LocalModelProvider): Promise<{ success: boolean; message: string }> {
    try {
      let testUrl: string;
      
      if (provider.name === 'Ollama') {
        testUrl = `${provider.baseUrl}/api/tags`;
      } else if (provider.name === 'LocalAI') {
        testUrl = `${provider.baseUrl}/v1/models`;
      } else {
        testUrl = `${provider.baseUrl}/health`; // Generic health endpoint
      }

      const response = await fetch(testUrl, {
        method: 'GET',
        timeout: 5000
      } as any);

      if (response.ok) {
        return { success: true, message: `${provider.name} connecté avec succès` };
      } else {
        return { success: false, message: `${provider.name} répond avec erreur ${response.status}` };
      }
    } catch (error) {
      return { 
        success: false, 
        message: `${provider.name} non accessible: ${error instanceof Error ? error.message : 'Erreur inconnue'}` 
      };
    }
  }
}

export const localModelDetector = new LocalModelDetector();