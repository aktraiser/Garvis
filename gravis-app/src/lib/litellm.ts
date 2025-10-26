// LiteLLM client configuration and models management
export interface LLMModel {
  id: string;
  name: string;
  provider: string;
  description: string;
  contextWindow: number;
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
}

// Available models configuration
export const AVAILABLE_MODELS: LLMModel[] = [
  {
    id: "gpt-4o",
    name: "GPT-4o",
    provider: "OpenAI",
    description: "Latest GPT-4 with vision and improved reasoning",
    contextWindow: 128000,
    pricing: { input: 0.005, output: 0.015 }
  },
  {
    id: "gpt-4o-mini",
    name: "GPT-4o Mini",
    provider: "OpenAI", 
    description: "Faster, cheaper GPT-4 variant",
    contextWindow: 128000,
    pricing: { input: 0.0015, output: 0.006 }
  },
  {
    id: "claude-3-5-sonnet-20241022",
    name: "Claude 3.5 Sonnet",
    provider: "Anthropic",
    description: "Advanced reasoning and code analysis",
    contextWindow: 200000,
    pricing: { input: 0.003, output: 0.015 }
  },
  {
    id: "claude-3-5-haiku-20241022",
    name: "Claude 3.5 Haiku",
    provider: "Anthropic",
    description: "Fast and efficient for simple tasks",
    contextWindow: 200000,
    pricing: { input: 0.001, output: 0.005 }
  },
  {
    id: "gemini-2.0-flash-exp",
    name: "Gemini 2.0 Flash",
    provider: "Google",
    description: "Experimental multimodal model",
    contextWindow: 128000,
    pricing: { input: 0.001, output: 0.005 }
  },
  {
    id: "deepseek-chat",
    name: "DeepSeek Chat",
    provider: "DeepSeek",
    description: "Specialized in code and reasoning",
    contextWindow: 32000,
    pricing: { input: 0.0007, output: 0.002 }
  },
  {
    id: "deepseek-reasoner",
    name: "DeepSeek Reasoner",
    provider: "DeepSeek",
    description: "Advanced reasoning with visible thinking process",
    contextWindow: 32000,
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

  async chat(messages: Array<{ role: string; content: string }>) {
    try {
      const response = await fetch(`${this.baseUrl}/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.config.apiKey}`,
        },
        body: JSON.stringify({
          model: this.config.model,
          messages,
          temperature: this.config.temperature || 0.7,
          max_tokens: this.config.maxTokens || 2000,
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
      const response = await fetch(`${this.baseUrl}/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.config.apiKey}`,
        },
        body: JSON.stringify({
          model: this.config.model,
          messages,
          temperature: this.config.temperature || 0.7,
          max_tokens: this.config.maxTokens || 2000,
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
      return { data: AVAILABLE_MODELS };
    }
  }
}

// Model selection utilities
export const getModelById = (id: string): LLMModel | undefined => {
  return AVAILABLE_MODELS.find(model => model.id === id);
};

export const getModelsByProvider = (provider: string): LLMModel[] => {
  return AVAILABLE_MODELS.filter(model => model.provider === provider);
};

export const getDefaultModel = (): LLMModel => {
  return AVAILABLE_MODELS[0]; // GPT-4o as default
};

// Store configuration with localStorage persistence
export const modelConfigStore = {
  currentModel: getDefaultModel(),
  apiKey: "",
  baseUrl: "http://localhost:4000",
  
  // Initialize from localStorage
  init: () => {
    try {
      const saved = localStorage.getItem('gravis-config');
      if (saved) {
        const config = JSON.parse(saved);
        modelConfigStore.apiKey = config.apiKey || "";
        modelConfigStore.baseUrl = config.baseUrl || "http://localhost:4000";
        
        // Restore model if it exists in our available models
        if (config.currentModel) {
          const foundModel = getModelById(config.currentModel.id);
          if (foundModel) {
            modelConfigStore.currentModel = foundModel;
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
  
  getConfig: (): LLMConfig => ({
    apiKey: modelConfigStore.apiKey,
    baseUrl: modelConfigStore.baseUrl,
    model: modelConfigStore.currentModel.id,
  })
};

// Initialize the store on module load
modelConfigStore.init();