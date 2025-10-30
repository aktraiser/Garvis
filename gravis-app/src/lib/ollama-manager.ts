// Gestionnaire Ollama pour télécharger, supprimer et gérer les modèles
export interface OllamaModel {
  name: string;
  size: number;
  digest: string;
  modified_at: string;
  details?: {
    format: string;
    family: string;
    families: string[];
    parameter_size: string;
    quantization_level: string;
  };
}

export interface OllamaModelInfo {
  license: string;
  modelfile: string;
  parameters: string;
  template: string;
  details: {
    format: string;
    family: string;
    families: string[];
    parameter_size: string;
    quantization_level: string;
  };
}

export interface OllamaModelPullProgress {
  status: string;
  digest?: string;
  total?: number;
  completed?: number;
}

export interface AvailableOllamaModel {
  name: string;
  description: string;
  size: string;
  tags: string[];
  popular: boolean;
  category: string;
}

// Modèles populaires Ollama à proposer au téléchargement
export const POPULAR_OLLAMA_MODELS: AvailableOllamaModel[] = [
  {
    name: "llama3.2:3b",
    description: "Llama 3.2 3B - Modèle rapide et efficace",
    size: "2.0GB",
    tags: ["small", "fast", "general"],
    popular: true,
    category: "general"
  },
  {
    name: "llama3.2:1b", 
    description: "Llama 3.2 1B - Ultra léger pour tests rapides",
    size: "1.3GB",
    tags: ["tiny", "fast", "experimental"],
    popular: true,
    category: "general"
  },
  {
    name: "llama3.1:8b",
    description: "Llama 3.1 8B - Bon équilibre performance/taille",
    size: "4.7GB", 
    tags: ["medium", "balanced", "general"],
    popular: true,
    category: "general"
  },
  {
    name: "codellama:7b",
    description: "Code Llama 7B - Spécialisé en programmation",
    size: "3.8GB",
    tags: ["code", "programming", "development"],
    popular: true,
    category: "code"
  },
  {
    name: "codegemma:7b",
    description: "CodeGemma 7B - Code et mathématiques",
    size: "5.0GB",
    tags: ["code", "math", "reasoning"],
    popular: true,
    category: "code"
  },
  {
    name: "qwen2.5:7b",
    description: "Qwen2.5 7B - Multilingue performant",
    size: "4.4GB",
    tags: ["multilingual", "general", "chinese"],
    popular: true,
    category: "general"
  },
  {
    name: "phi3:3.8b",
    description: "Phi-3 3.8B - Microsoft, optimisé mobile",
    size: "2.3GB",
    tags: ["small", "microsoft", "mobile"],
    popular: false,
    category: "general"
  },
  {
    name: "gemma2:2b",
    description: "Gemma 2 2B - Google, ultra compact",
    size: "1.6GB",
    tags: ["tiny", "google", "efficient"],
    popular: false,
    category: "general"
  },
  {
    name: "mistral:7b",
    description: "Mistral 7B - Français excellent",
    size: "4.1GB",
    tags: ["french", "european", "general"],
    popular: true,
    category: "general"
  },
  {
    name: "deepseek-coder:6.7b",
    description: "DeepSeek Coder 6.7B - Code spécialisé",
    size: "3.8GB",
    tags: ["code", "programming", "specialized"],
    popular: false,
    category: "code"
  },
  {
    name: "granite-code:3b",
    description: "Granite Code 3B - IBM, code et instruction",
    size: "2.0GB",
    tags: ["code", "programming", "ibm"],
    popular: true,
    category: "code"
  },
  {
    name: "gemma3:1b",
    description: "Gemma 3 1B - Google, ultra léger et rapide",
    size: "1.3GB",
    tags: ["tiny", "google", "fast", "128k"],
    popular: true,
    category: "general"
  },
  {
    name: "deepseek-r1:1.5b",
    description: "DeepSeek R1 1.5B - Raisonnement avancé compact",
    size: "1.5GB",
    tags: ["reasoning", "small", "thinking"],
    popular: true,
    category: "reasoning"
  },
  {
    name: "qwen3-vl:2b",
    description: "Qwen 3 Vision-Language 2B - Multimodal compact",
    size: "2.0GB",
    tags: ["vision", "multimodal", "small", "vl"],
    popular: true,
    category: "multimodal"
  }
];

export class OllamaManager {
  private baseUrl = 'http://localhost:11434';

  // Vérifier si Ollama est disponible
  async isAvailable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/api/version`);
      return response.ok;
    } catch {
      return false;
    }
  }

  // Obtenir la version d'Ollama
  async getVersion(): Promise<string | null> {
    try {
      const response = await fetch(`${this.baseUrl}/api/version`);
      if (!response.ok) return null;
      const data = await response.json();
      return data.version;
    } catch {
      return null;
    }
  }

  // Lister les modèles installés
  async listModels(): Promise<OllamaModel[]> {
    try {
      const response = await fetch(`${this.baseUrl}/api/tags`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      return data.models || [];
    } catch (error) {
      console.error('Erreur lors de la récupération des modèles Ollama:', error);
      return [];
    }
  }

  // Obtenir les informations détaillées d'un modèle
  async getModelInfo(modelName: string): Promise<OllamaModelInfo | null> {
    try {
      const response = await fetch(`${this.baseUrl}/api/show`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ name: modelName }),
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      return await response.json();
    } catch (error) {
      console.error(`Erreur lors de la récupération des infos du modèle ${modelName}:`, error);
      return null;
    }
  }

  // Télécharger un modèle avec suivi du progrès
  async downloadModel(modelName: string, onProgress?: (progress: OllamaModelPullProgress) => void): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/api/pull`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ name: modelName }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      if (!response.body) {
        throw new Error('Pas de corps de réponse');
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value, { stream: true });
          const lines = chunk.split('\n').filter(line => line.trim());

          for (const line of lines) {
            try {
              const progress = JSON.parse(line);
              if (onProgress) {
                onProgress(progress);
              }
              
              // Si c'est terminé avec succès
              if (progress.status === 'success') {
                return true;
              }
              
              // Si il y a une erreur
              if (progress.error) {
                throw new Error(progress.error);
              }
            } catch (parseError) {
              // Ignorer les erreurs de parsing JSON (chunks incomplets)
            }
          }
        }
      } finally {
        reader.releaseLock();
      }

      return true;
    } catch (error) {
      console.error(`Erreur lors du téléchargement du modèle ${modelName}:`, error);
      return false;
    }
  }

  // Supprimer un modèle
  async deleteModel(modelName: string): Promise<boolean> {
    try {
      const url = `${this.baseUrl}/api/delete`;
      const body = JSON.stringify({ name: modelName });
      
      const response = await fetch(url, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
        body: body,
      });

      if (!response.ok) {
        const errorText = await response.text();
        console.error(`Erreur API Ollama: ${response.status} - ${errorText}`);
        return false;
      }

      return response.ok;
    } catch (error) {
      console.error(`Erreur fetch: ${error}`);
      return false;
    }
  }

  // Obtenir l'utilisation disque des modèles
  async getDiskUsage(): Promise<{ total: number; models: Array<{ name: string; size: number }> }> {
    try {
      const models = await this.listModels();
      const total = models.reduce((sum, model) => sum + model.size, 0);
      
      return {
        total,
        models: models.map(model => ({
          name: model.name,
          size: model.size
        }))
      };
    } catch {
      return { total: 0, models: [] };
    }
  }

  // Formatage de la taille en lecture humaine
  formatSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  // Obtenir les modèles populaires filtrés par catégorie
  getPopularModels(category?: string): AvailableOllamaModel[] {
    if (!category) {
      return POPULAR_OLLAMA_MODELS.filter(model => model.popular);
    }
    return POPULAR_OLLAMA_MODELS.filter(model => 
      model.category === category && model.popular
    );
  }

  // Obtenir toutes les catégories disponibles
  getCategories(): string[] {
    const categories = [...new Set(POPULAR_OLLAMA_MODELS.map(model => model.category))];
    return categories.sort();
  }
}

// Instance singleton
export const ollamaManager = new OllamaManager();