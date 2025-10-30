// LiteLLM client configuration and models management
export interface LLMModel {
  id: string;
  name: string;
  provider: string;
  description: string;
  contextWindow: number;
  capabilities?: string[];
  pricing?: {
    input: number;
    output: number;
  };
}

export interface LLMConfig {
  apiKey: string;
  baseUrl?: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  systemPrompt?: string;
}

// Available models configuration
export const AVAILABLE_MODELS: LLMModel[] = [
  {
    id: "gpt-4o",
    name: "GPT-4o",
    provider: "OpenAI",
    description: "Latest GPT-4 with vision and improved reasoning",
    contextWindow: 128000,
    capabilities: ["vision", "tools", "reasoning"],
    pricing: { input: 0.005, output: 0.015 }
  },
  {
    id: "gpt-4o-mini",
    name: "GPT-4o Mini",
    provider: "OpenAI", 
    description: "Faster, cheaper GPT-4 variant",
    contextWindow: 128000,
    capabilities: ["vision", "tools"],
    pricing: { input: 0.0015, output: 0.006 }
  },
  {
    id: "claude-3-5-sonnet-20241022",
    name: "Claude 3.5 Sonnet",
    provider: "Anthropic",
    description: "Advanced reasoning and code analysis",
    contextWindow: 200000,
    capabilities: ["tools", "reasoning", "code"],
    pricing: { input: 0.003, output: 0.015 }
  },
  {
    id: "claude-3-5-haiku-20241022",
    name: "Claude 3.5 Haiku",
    provider: "Anthropic",
    description: "Fast and efficient for simple tasks",
    contextWindow: 200000,
    capabilities: ["tools"],
    pricing: { input: 0.001, output: 0.005 }
  },
  {
    id: "gemini-2.0-flash-exp",
    name: "Gemini 2.0 Flash",
    provider: "Google",
    description: "Experimental multimodal model",
    contextWindow: 128000,
    capabilities: ["vision", "tools", "multimodal"],
    pricing: { input: 0.001, output: 0.005 }
  },
  {
    id: "deepseek-chat",
    name: "DeepSeek Chat",
    provider: "DeepSeek",
    description: "Specialized in code and reasoning",
    contextWindow: 32000,
    capabilities: ["code", "reasoning"],
    pricing: { input: 0.0007, output: 0.002 }
  },
  {
    id: "deepseek-reasoner",
    name: "DeepSeek Reasoner",
    provider: "DeepSeek",
    description: "Advanced reasoning with visible thinking process",
    contextWindow: 32000,
    capabilities: ["thinking", "reasoning", "code"],
    pricing: { input: 0.0014, output: 0.008 }
  }
];

// LiteLLM API client
export class LiteLLMClient {
  private config: LLMConfig;
  private baseUrl: string;

  constructor(config: LLMConfig) {
    this.config = config;
    this.baseUrl = config.baseUrl || "http://localhost:4000";
  }

  private getEndpointForModel(): { baseUrl: string; apiKey: string; modelName: string } {
    const currentModel = modelConfigStore.currentModel;
    
    console.log('🔍 getEndpointForModel - Current model:', currentModel);
    console.log('🔍 Provider check:', currentModel.provider);
    console.log('🔍 ID check:', currentModel.id);
    
    // Pour les modèles Ollama, router vers localhost:11434
    if (currentModel.provider === 'Ollama' || 
        currentModel.provider === 'Ollama (Local)' || 
        currentModel.id.startsWith('ollama/')) {
      console.log('✅ Using Ollama endpoint: localhost:11434');
      return {
        baseUrl: 'http://localhost:11434',
        apiKey: '', // Ollama n'a pas besoin de clé API
        modelName: currentModel.name || currentModel.id.replace('ollama/', '')
      };
    }
    
    // Pour les autres modèles, utiliser la configuration LiteLLM
    return {
      baseUrl: this.baseUrl,
      apiKey: this.config.apiKey,
      modelName: currentModel.id.startsWith('litellm/') ? 
        currentModel.id.replace('litellm/', '') : 
        currentModel.id
    };
  }

  async chat(messages: Array<{ role: string; content: string }>) {
    try {
      const endpoint = this.getEndpointForModel();
      const headers: { [key: string]: string } = {
        "Content-Type": "application/json",
      };
      
      // Ajouter l'autorisation seulement si une clé API est fournie
      if (endpoint.apiKey) {
        headers["Authorization"] = `Bearer ${endpoint.apiKey}`;
      }
      
      // Utiliser l'endpoint correct selon le provider
      const currentModel = modelConfigStore.currentModel;
      const isOllamaProvider = currentModel.provider === 'Ollama' || 
                              currentModel.provider === 'Ollama (Local)' || 
                              currentModel.id.startsWith('ollama/');
      const apiEndpoint = isOllamaProvider ? 
        `${endpoint.baseUrl}/v1/chat/completions` : 
        `${endpoint.baseUrl}/chat/completions`;
      
      console.log('🎯 Final API endpoint:', apiEndpoint);
      
      const response = await fetch(apiEndpoint, {
        method: "POST",
        headers,
        body: JSON.stringify({
          model: endpoint.modelName,
          messages,
          temperature: this.config.temperature || 0.7,
          max_tokens: this.config.maxTokens || 2000,
          top_p: this.config.topP || 1.0,
          frequency_penalty: this.config.frequencyPenalty || 0.0,
          presence_penalty: this.config.presencePenalty || 0.0,
          stream: false,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("LiteLLM API error:", error);
      throw error;
    }
  }

  async chatStream(messages: Array<{ role: string; content: string }>) {
    try {
      const endpoint = this.getEndpointForModel();
      const headers: { [key: string]: string } = {
        "Content-Type": "application/json",
      };
      
      // Ajouter l'autorisation seulement si une clé API est fournie
      if (endpoint.apiKey) {
        headers["Authorization"] = `Bearer ${endpoint.apiKey}`;
      }
      
      // Utiliser l'endpoint correct selon le provider
      const currentModel = modelConfigStore.currentModel;
      const isOllamaProvider = currentModel.provider === 'Ollama' || 
                              currentModel.provider === 'Ollama (Local)' || 
                              currentModel.id.startsWith('ollama/');
      const apiEndpoint = isOllamaProvider ? 
        `${endpoint.baseUrl}/v1/chat/completions` : 
        `${endpoint.baseUrl}/chat/completions`;
      
      console.log('🎯 Final API endpoint:', apiEndpoint);
      
      const response = await fetch(apiEndpoint, {
        method: "POST",
        headers,
        body: JSON.stringify({
          model: endpoint.modelName,
          messages,
          temperature: this.config.temperature || 0.7,
          max_tokens: this.config.maxTokens || 2000,
          top_p: this.config.topP || 1.0,
          frequency_penalty: this.config.frequencyPenalty || 0.0,
          presence_penalty: this.config.presencePenalty || 0.0,
          stream: true,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return response.body;
    } catch (error) {
      console.error("LiteLLM Stream API error:", error);
      throw error;
    }
  }

  async getModels() {
    // Si aucune connexion n'est configurée, retourner une liste vide
    if (modelConfigStore.activeConnections.length === 0 && !modelConfigStore.selectedConnectionId) {
      return { data: [] };
    }

    try {
      const response = await fetch(`${this.baseUrl}/models`, {
        headers: {
          "Authorization": `Bearer ${this.config.apiKey}`,
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Failed to fetch models:", error);
      // Si on a des connexions mais que ça échoue, retourner les modèles par défaut
      if (modelConfigStore.activeConnections.length > 0 || modelConfigStore.selectedConnectionId) {
        return { data: AVAILABLE_MODELS };
      }
      // Sinon, liste vide
      return { data: [] };
    }
  }

  async getModelInfo() {
    try {
      const response = await fetch(`${this.baseUrl}/model/info`, {
        headers: {
          "Authorization": `Bearer ${this.config.apiKey}`,
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      console.log('Model Info API Response:', JSON.stringify(result, null, 2));
      return result;
    } catch (error) {
      console.error("Failed to fetch model info:", error);
      return null;
    }
  }
}

// Model selection utilities
export const getModelById = async (id: string): Promise<LLMModel | undefined> => {
  // 1. Chercher dans les modèles statiques d'abord
  const staticModel = AVAILABLE_MODELS.find(model => model.id === id);
  if (staticModel) {
    return staticModel;
  }
  
  // 2. Si c'est un modèle Ollama, essayer de le créer dynamiquement
  if (id.startsWith('ollama/')) {
    try {
      const { localModelDetector } = await import('./local-models');
      const ollamaDetection = await localModelDetector.detectOllamaModels();
      
      if (ollamaDetection.isAvailable) {
        const modelName = id.replace('ollama/', '');
        const ollamaModel = ollamaDetection.models.find(model => 
          model.name === modelName || model.id === modelName || model.id === id
        );
        
        if (ollamaModel) {
          return {
            ...ollamaModel,
            id: id,
            provider: 'Ollama (Local)'
          };
        }
      }
    } catch (error) {
      console.warn('Failed to detect Ollama model for getModelById:', error);
    }
  }
  
  return undefined;
};

export const getModelsByProvider = (provider: string): LLMModel[] => {
  return AVAILABLE_MODELS.filter(model => model.provider === provider);
};

export const getDefaultModel = (): LLMModel => {
  return AVAILABLE_MODELS[0]; // GPT-4o as default
};

// Store configuration with localStorage persistence
export const modelConfigStore: any = {
  currentModel: getDefaultModel(),
  apiKey: "",
  baseUrl: "http://localhost:4000",
  activeConnections: [] as Array<{id: string, name: string, baseUrl: string, apiKey: string, type: string}>,
  selectedConnectionId: null as string | null,
  
  // Paramètres par défaut pour les modèles
  modelParameters: {
    temperature: 0.7,
    maxTokens: 2000,
    topP: 1.0,
    frequencyPenalty: 0.0,
    presencePenalty: 0.0,
    systemPrompt: ''
  },
  
  // Initialize from localStorage
  init: async () => {
    try {
      const saved = localStorage.getItem('gravis-config');
      if (saved) {
        const config = JSON.parse(saved);
        modelConfigStore.apiKey = config.apiKey || "";
        modelConfigStore.baseUrl = config.baseUrl || "http://localhost:4000";
        modelConfigStore.activeConnections = config.activeConnections || [];
        modelConfigStore.selectedConnectionId = config.selectedConnectionId || null;
        
        // Restore model parameters
        if (config.modelParameters) {
          modelConfigStore.modelParameters = {
            ...modelConfigStore.modelParameters,
            ...config.modelParameters
          };
        }
        
        // Restore model if it exists in our available models (now supports dynamic models)
        if (config.currentModel) {
          const foundModel = await getModelById(config.currentModel.id);
          if (foundModel) {
            console.log('🔄 Restored model from localStorage:', foundModel);
            modelConfigStore.currentModel = foundModel;
          } else {
            console.warn('⚠️ Model not found, keeping default:', config.currentModel.id);
          }
        }
      }
    } catch (error) {
      console.warn('Failed to load config from localStorage:', error);
    }
  },
  
  // Save to localStorage
  save: () => {
    try {
      const config = {
        apiKey: modelConfigStore.apiKey,
        baseUrl: modelConfigStore.baseUrl,
        currentModel: modelConfigStore.currentModel,
        activeConnections: modelConfigStore.activeConnections,
        selectedConnectionId: modelConfigStore.selectedConnectionId,
        modelParameters: modelConfigStore.modelParameters,
      };
      localStorage.setItem('gravis-config', JSON.stringify(config));
    } catch (error) {
      console.warn('Failed to save config to localStorage:', error);
    }
  },
  
  setModel: (model: LLMModel) => {
    modelConfigStore.currentModel = model;
    modelConfigStore.save();
  },
  
  setApiKey: (key: string) => {
    modelConfigStore.apiKey = key;
    modelConfigStore.save();
  },
  
  setBaseUrl: (url: string) => {
    modelConfigStore.baseUrl = url;
    modelConfigStore.save();
  },
  
  setActiveConnections: (connections: Array<{id: string, name: string, baseUrl: string, apiKey: string, type: string}>) => {
    modelConfigStore.activeConnections = connections;
    modelConfigStore.save();
  },
  
  setSelectedConnection: (connectionId: string | null) => {
    modelConfigStore.selectedConnectionId = connectionId;
    modelConfigStore.save();
  },
  
  setModelParameters: (params: Partial<typeof modelConfigStore.modelParameters>) => {
    console.log('🔧 Setting model parameters:', params);
    modelConfigStore.modelParameters = {
      ...modelConfigStore.modelParameters,
      ...params
    };
    console.log('🔧 Updated model parameters:', modelConfigStore.modelParameters);
    modelConfigStore.save();
    console.log('🔧 Model parameters saved to localStorage');
  },
  
  getConfig: (): LLMConfig => {
    console.log('🔧 getConfig called');
    console.log('🔧 Current model ID:', modelConfigStore.currentModel.id);
    console.log('🔧 Selected connection ID:', modelConfigStore.selectedConnectionId);
    console.log('🔧 Active connections:', modelConfigStore.activeConnections);
    
    // Utiliser la connexion sélectionnée si elle existe
    if (modelConfigStore.selectedConnectionId) {
      const selectedConnection = modelConfigStore.activeConnections.find(
        (conn: any) => conn.id === modelConfigStore.selectedConnectionId
      );
      if (selectedConnection) {
        console.log('✅ Using selected connection config:', selectedConnection);
        return {
          apiKey: selectedConnection.apiKey,
          baseUrl: selectedConnection.baseUrl,
          model: modelConfigStore.currentModel.id,
          ...modelConfigStore.modelParameters,
        };
      }
    }
    
    // Détection automatique pour modèles Ollama
    if (modelConfigStore.currentModel.id.startsWith('ollama/')) {
      console.log('✅ Auto-detected Ollama model, using localhost:11434');
      return {
        apiKey: '', // Ollama n'a pas besoin de clé API
        baseUrl: 'http://localhost:11434',
        model: modelConfigStore.currentModel.id,
        ...modelConfigStore.modelParameters,
      };
    }
    
    // Fallback vers les valeurs directes (legacy)
    console.log('⚠️ Using fallback config - baseUrl:', modelConfigStore.baseUrl);
    return {
      apiKey: modelConfigStore.apiKey,
      baseUrl: modelConfigStore.baseUrl,
      model: modelConfigStore.currentModel.id,
      ...modelConfigStore.modelParameters,
    };
  }
};

// Initialize the store on module load
modelConfigStore.init().catch((error: any) => {
  console.warn('Failed to initialize modelConfigStore:', error);
});