// Gestionnaire des conversations pour GRAVIS

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  updatedAt: Date;
  model: string;
  tags: string[];
}

export class ConversationManager {
  private static instance: ConversationManager;
  private currentConversation: Conversation | null = null;
  private readonly storageKey = 'gravis-conversations';

  static getInstance(): ConversationManager {
    if (!ConversationManager.instance) {
      ConversationManager.instance = new ConversationManager();
    }
    return ConversationManager.instance;
  }

  // Générer un titre de conversation basé sur le premier message
  private generateTitle(firstMessage: string): string {
    // Nettoyer et truncquer le message pour faire un titre
    const title = firstMessage
      .replace(/[^\w\s]/g, '') // Supprimer la ponctuation
      .trim()
      .split(' ')
      .slice(0, 6) // Prendre les 6 premiers mots
      .join(' ');
    
    return title || 'Nouvelle conversation';
  }

  // Démarrer une nouvelle conversation
  startNewConversation(firstUserMessage: string, model: string): Conversation {
    const conversationId = `conv_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const userMessage: Message = {
      id: messageId,
      role: 'user',
      content: firstUserMessage,
      timestamp: new Date()
    };

    const conversation: Conversation = {
      id: conversationId,
      title: this.generateTitle(firstUserMessage),
      messages: [userMessage],
      createdAt: new Date(),
      updatedAt: new Date(),
      model: model,
      tags: this.extractTags(firstUserMessage)
    };

    this.currentConversation = conversation;
    return conversation;
  }

  // Extraire des tags automatiquement du contenu
  private extractTags(content: string): string[] {
    const tags: string[] = [];
    const lowerContent = content.toLowerCase();

    // Tags basés sur des mots clés
    const tagKeywords = {
      'code': ['code', 'programming', 'fonction', 'script', 'debug', 'error'],
      'documentation': ['doc', 'documentation', 'readme', 'guide', 'tutorial'],
      'analyse': ['analyse', 'analyser', 'étudier', 'examiner', 'rapport'],
      'création': ['créer', 'générer', 'faire', 'construire', 'développer'],
      'question': ['comment', 'pourquoi', 'que', 'quoi', 'quel', '?'],
      'technique': ['api', 'base de données', 'serveur', 'réseau', 'système']
    };

    for (const [tag, keywords] of Object.entries(tagKeywords)) {
      if (keywords.some(keyword => lowerContent.includes(keyword))) {
        tags.push(tag);
      }
    }

    return tags.length > 0 ? tags : ['général'];
  }

  // Ajouter un message à la conversation actuelle
  addMessage(role: 'user' | 'assistant', content: string): Message {
    if (!this.currentConversation) {
      throw new Error('Aucune conversation active');
    }

    const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const message: Message = {
      id: messageId,
      role,
      content,
      timestamp: new Date()
    };

    this.currentConversation.messages.push(message);
    this.currentConversation.updatedAt = new Date();

    // Mettre à jour les tags si nécessaire
    if (role === 'user') {
      const newTags = this.extractTags(content);
      for (const tag of newTags) {
        if (!this.currentConversation.tags.includes(tag)) {
          this.currentConversation.tags.push(tag);
        }
      }
    }

    return message;
  }

  // Sauvegarder la conversation actuelle
  saveCurrentConversation(): void {
    if (!this.currentConversation) {
      return;
    }

    try {
      const conversations = this.loadConversations();
      const existingIndex = conversations.findIndex(conv => conv.id === this.currentConversation!.id);
      
      if (existingIndex >= 0) {
        // Mettre à jour la conversation existante
        conversations[existingIndex] = { ...this.currentConversation };
      } else {
        // Ajouter une nouvelle conversation
        conversations.unshift({ ...this.currentConversation });
      }

      // Limiter à 100 conversations max pour éviter de surcharger le localStorage
      const limitedConversations = conversations.slice(0, 100);

      localStorage.setItem(this.storageKey, JSON.stringify(limitedConversations));
      console.log('✅ Conversation sauvegardée:', this.currentConversation.title);
    } catch (error) {
      console.error('❌ Erreur lors de la sauvegarde de la conversation:', error);
    }
  }

  // Charger toutes les conversations
  loadConversations(): Conversation[] {
    try {
      const saved = localStorage.getItem(this.storageKey);
      if (!saved) return [];

      const conversations = JSON.parse(saved);
      return conversations.map((conv: any) => ({
        ...conv,
        createdAt: new Date(conv.createdAt),
        updatedAt: new Date(conv.updatedAt),
        messages: conv.messages.map((msg: any) => ({
          ...msg,
          timestamp: new Date(msg.timestamp)
        }))
      }));
    } catch (error) {
      console.error('❌ Erreur lors du chargement des conversations:', error);
      return [];
    }
  }

  // Obtenir la conversation actuelle
  getCurrentConversation(): Conversation | null {
    return this.currentConversation;
  }

  // Finir la conversation actuelle
  endCurrentConversation(): void {
    if (this.currentConversation) {
      this.saveCurrentConversation();
      this.currentConversation = null;
      console.log('📝 Conversation terminée et sauvegardée');
    }
  }

  // Reprendre une conversation existante
  resumeConversation(conversationId: string): Conversation | null {
    try {
      const conversations = this.loadConversations();
      const conversation = conversations.find(conv => conv.id === conversationId);
      
      if (conversation) {
        // Sauvegarder la conversation actuelle avant de reprendre une autre
        this.endCurrentConversation();
        
        // Reprendre la conversation sélectionnée
        this.currentConversation = { ...conversation };
        console.log('🔄 Conversation reprise:', conversation.title);
        return this.currentConversation;
      } else {
        console.error('❌ Conversation non trouvée:', conversationId);
        return null;
      }
    } catch (error) {
      console.error('❌ Erreur lors de la reprise de conversation:', error);
      return null;
    }
  }

  // Supprimer une conversation
  deleteConversation(conversationId: string): void {
    try {
      const conversations = this.loadConversations();
      const filtered = conversations.filter(conv => conv.id !== conversationId);
      localStorage.setItem(this.storageKey, JSON.stringify(filtered));
      
      if (this.currentConversation?.id === conversationId) {
        this.currentConversation = null;
      }
      
      console.log('🗑️ Conversation supprimée:', conversationId);
    } catch (error) {
      console.error('❌ Erreur lors de la suppression:', error);
    }
  }

  // Exporter une conversation
  exportConversation(conversationId: string): Conversation | null {
    const conversations = this.loadConversations();
    return conversations.find(conv => conv.id === conversationId) || null;
  }

  // Statistiques
  getStats() {
    const conversations = this.loadConversations();
    const totalMessages = conversations.reduce((total, conv) => total + conv.messages.length, 0);
    const totalConversations = conversations.length;
    
    // Modèles les plus utilisés
    const modelUsage = conversations.reduce((acc, conv) => {
      acc[conv.model] = (acc[conv.model] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    // Tags les plus populaires
    const tagUsage = conversations.reduce((acc, conv) => {
      conv.tags.forEach(tag => {
        acc[tag] = (acc[tag] || 0) + 1;
      });
      return acc;
    }, {} as Record<string, number>);

    return {
      totalConversations,
      totalMessages,
      modelUsage,
      tagUsage,
      averageMessagesPerConversation: totalConversations > 0 ? Math.round(totalMessages / totalConversations) : 0
    };
  }
}

// Instance singleton
export const conversationManager = ConversationManager.getInstance();