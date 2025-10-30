// Hugging Face Manager - Gestion des modèles HF
export interface HuggingFaceModel {
  id: string;
  name: string;
  author: string;
  downloads: number;
  likes: number;
  modelType: string;
  size?: string;
  lastModified: string;
}

export interface PopularHFModel {
  id: string;
  name: string;
  description: string;
  author: string;
  modelType: string;
  size: string;
  tags: string[];
  popular: boolean;
  category: 'text-generation' | 'text-to-image' | 'image-to-text' | 'embedding' | 'classification';
}

// Liste des modèles populaires Hugging Face
const POPULAR_HF_MODELS: PopularHFModel[] = [
  {
    id: "microsoft/DialoGPT-medium",
    name: "DialoGPT Medium",
    description: "Modèle de dialogue conversationnel de Microsoft",
    author: "microsoft",
    modelType: "text-generation",
    size: "350MB",
    tags: ["dialogue", "conversation", "gpt"],
    popular: true,
    category: "text-generation"
  },
  {
    id: "sentence-transformers/all-MiniLM-L6-v2",
    name: "All MiniLM L6 v2",
    description: "Modèle d'embedding de phrases rapide et efficace",
    author: "sentence-transformers",
    modelType: "sentence-similarity",
    size: "90MB",
    tags: ["embedding", "sentence", "similarity"],
    popular: true,
    category: "embedding"
  },
  {
    id: "facebook/bart-large-cnn",
    name: "BART Large CNN",
    description: "Modèle de résumé de texte par Facebook",
    author: "facebook",
    modelType: "summarization",
    size: "1.6GB",
    tags: ["summarization", "news", "bart"],
    popular: true,
    category: "text-generation"
  },
  {
    id: "openai/clip-vit-base-patch32",
    name: "CLIP ViT Base",
    description: "Modèle vision-langage d'OpenAI",
    author: "openai",
    modelType: "zero-shot-image-classification",
    size: "600MB",
    tags: ["vision", "image", "classification"],
    popular: true,
    category: "image-to-text"
  },
  {
    id: "stabilityai/stable-diffusion-2-1",
    name: "Stable Diffusion 2.1",
    description: "Générateur d'images par Stability AI",
    author: "stabilityai",
    modelType: "text-to-image",
    size: "4.2GB",
    tags: ["diffusion", "image-generation", "art"],
    popular: true,
    category: "text-to-image"
  },
  {
    id: "intfloat/e5-large-v2",
    name: "E5 Large v2",
    description: "Modèle d'embedding multilingue haute performance",
    author: "intfloat",
    modelType: "feature-extraction",
    size: "1.3GB",
    tags: ["embedding", "multilingual", "large"],
    popular: true,
    category: "embedding"
  },
  {
    id: "cardiffnlp/twitter-roberta-base-sentiment-latest",
    name: "Twitter RoBERTa Sentiment",
    description: "Analyse de sentiment pour Twitter",
    author: "cardiffnlp",
    modelType: "text-classification",
    size: "500MB",
    tags: ["sentiment", "twitter", "roberta"],
    popular: true,
    category: "classification"
  },
  {
    id: "google/flan-t5-base",
    name: "FLAN-T5 Base",
    description: "Modèle de génération de texte par Google",
    author: "google",
    modelType: "text2text-generation",
    size: "990MB",
    tags: ["t5", "generation", "google"],
    popular: true,
    category: "text-generation"
  }
];

export class HuggingFaceManager {
  private baseUrl = 'https://huggingface.co/api';

  // Vérifier si HF Hub est disponible
  async isAvailable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/models?limit=1`);
      return response.ok;
    } catch (error) {
      console.error('HuggingFace API not available:', error);
      return false;
    }
  }

  // Rechercher des modèles
  async searchModels(query: string, limit: number = 20): Promise<HuggingFaceModel[]> {
    try {
      const url = `${this.baseUrl}/models?search=${encodeURIComponent(query)}&limit=${limit}`;
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`HF API error: ${response.status}`);
      }
      
      const models = await response.json();
      
      return models.map((model: any) => ({
        id: model.id,
        name: model.id.split('/').pop() || model.id,
        author: model.id.split('/')[0] || 'unknown',
        downloads: model.downloads || 0,
        likes: model.likes || 0,
        modelType: model.pipeline_tag || 'unknown',
        lastModified: model.lastModified || new Date().toISOString()
      }));
    } catch (error) {
      console.error('Error searching HF models:', error);
      return [];
    }
  }

  // Obtenir les modèles populaires
  getPopularModels(): PopularHFModel[] {
    return POPULAR_HF_MODELS;
  }

  // Obtenir les catégories disponibles
  getCategories(): string[] {
    const categories = [...new Set(POPULAR_HF_MODELS.map(model => model.category))];
    return categories.sort();
  }

  // Télécharger un modèle (simulation - dans la vraie vie, on utiliserait transformers.js ou l'API HF)
  async downloadModel(
    modelId: string, 
    onProgress?: (progress: { completed: number; total: number; status: string }) => void
  ): Promise<boolean> {
    try {
      // Simulation du téléchargement
      const steps = ['Connecting...', 'Downloading...', 'Installing...', 'Finalizing...'];
      
      for (let i = 0; i < steps.length; i++) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        if (onProgress) {
          onProgress({
            completed: i + 1,
            total: steps.length,
            status: steps[i]
          });
        }
      }
      
      console.log(`✅ Model ${modelId} downloaded successfully`);
      return true;
    } catch (error) {
      console.error(`❌ Error downloading model ${modelId}:`, error);
      return false;
    }
  }

  // Lister les modèles installés localement (simulation)
  async listLocalModels(): Promise<HuggingFaceModel[]> {
    // Dans une vraie implémentation, on scannerait le dossier ~/.cache/huggingface/transformers
    // Pour l'instant, on retourne une liste vide
    return [];
  }

  // Supprimer un modèle local (simulation)
  async deleteModel(modelId: string): Promise<boolean> {
    try {
      // Dans une vraie implémentation, on supprimerait les fichiers du cache
      console.log(`🗑️ Deleting model ${modelId}...`);
      await new Promise(resolve => setTimeout(resolve, 500));
      console.log(`✅ Model ${modelId} deleted successfully`);
      return true;
    } catch (error) {
      console.error(`❌ Error deleting model ${modelId}:`, error);
      return false;
    }
  }

  // Formater la taille des fichiers
  formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // Obtenir les détails d'un modèle
  async getModelDetails(modelId: string): Promise<any> {
    try {
      const response = await fetch(`${this.baseUrl}/models/${modelId}`);
      if (!response.ok) {
        throw new Error(`HF API error: ${response.status}`);
      }
      return await response.json();
    } catch (error) {
      console.error(`Error getting model details for ${modelId}:`, error);
      return null;
    }
  }
}

// Instance globale
export const huggingFaceManager = new HuggingFaceManager();