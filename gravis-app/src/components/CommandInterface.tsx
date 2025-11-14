import { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import { RagWindow } from './RagWindow';
import { ModelSelectorWindow } from './ModelSelectorWindow';
import { FileBadge, OCRPanel } from './direct-chat';
import { useDirectChat } from '@/hooks/useDirectChat';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import {
  Plus,
  Globe,
  FileText,
  Radio,
  Mic,
  Send,
  Settings,
  Wifi,
  CheckCircle,
  XCircle,
  Loader2,
  Bot,
  Copy,
  Volume2,
  RotateCcw,
  MessageSquare,
  Database
} from "lucide-react";
import { LiteLLMClient, modelConfigStore } from "@/lib/litellm";
import { tauriModelStore } from "@/lib/tauri-model-store";
import { conversationManager } from "@/lib/conversation-manager";
import { useRagQuery, type RagContextResponse } from "@/hooks/useRagQuery";

// Fonction pour détecter les boucles infinies dans le thinking
const detectThinkingLoop = (text: string): boolean => {
  // Détecter les répétitions extrêmes de mots très courts (uniquement "the the the...")
  const extremeWordLoop = /\b(the|a|an|in|on|at|of|to|for)\s+\1(\s+\1){10,}/gi;
  if (extremeWordLoop.test(text)) {
    console.log('⚠️ Detected extreme word loop in thinking');
    return true;
  }
  
  // Détecter seulement si le thinking dépasse 3000 caractères (très long)
  if (text.length > 3000) {
    console.log('⚠️ Thinking too long, potential loop');
    return true;
  }
  
  return false;
};

// Fonction pour nettoyer le thinking des boucles
const cleanThinkingLoops = (text: string): string => {
  console.log('🧹 Cleaning thinking loops, original length:', text.length);
  
  // Supprimer seulement les répétitions extrêmes de mots très courts (10+ fois)
  text = text.replace(/\b(the|a|an|in|on|at|of|to|for)(\s+\1){10,}/gi, '$1 [loop detected and cleaned]');
  
  // Limiter la longueur du thinking à 2500 caractères max (plus généreux)
  if (text.length > 2500) {
    text = text.substring(0, 2500) + '\n\n[Thinking truncated - too long]';
  }
  
  console.log('🧹 Cleaned thinking length:', text.length);
  return text;
};

// Fonction pour parser le thinking dans le stream
const parseThinkingStream = (content: string) => {
  console.log('🔍 Parsing content length:', content.length);
  
  let thinkingContent = "";
  let mainContent = content;
  
  // Regex plus robuste pour capturer le thinking
  const thinkRegex = /<think>\s*([\s\S]*?)\s*<\/think>/g;
  let match;
  
  // Extraire tout le contenu thinking
  while ((match = thinkRegex.exec(content)) !== null) {
    const extracted = match[1].trim();
    if (extracted) {
      thinkingContent += (thinkingContent ? "\n\n" : "") + extracted;
    }
  }
  
  // Nettoyer les boucles infinies dans le thinking
  if (thinkingContent && detectThinkingLoop(thinkingContent)) {
    console.log('🚨 Loop detected in thinking, before cleaning:', thinkingContent.length, 'chars');
    thinkingContent = cleanThinkingLoops(thinkingContent);
    console.log('🔧 Cleaned thinking loops, after:', thinkingContent.length, 'chars');
  } else if (thinkingContent) {
    console.log('✅ Normal thinking, no loops detected:', thinkingContent.length, 'chars');
  }
  
  // Supprimer complètement les balises thinking du contenu principal
  mainContent = content
    .replace(/<think>\s*[\s\S]*?\s*<\/think>/g, '')  // Supprimer blocs complets
    .replace(/<think>/g, '')  // Supprimer balises ouvertes orphelines
    .replace(/<\/think>/g, '') // Supprimer balises fermées orphelines
    .trim(); // Nettoyer les espaces
  
  console.log('🧠 Thinking extracted:', thinkingContent.length, 'chars');
  console.log('📝 Main content after cleaning:', mainContent.length, 'chars');
  
  return {
    thinking: thinkingContent,
    content: mainContent
  };
};

export function CommandInterface() {
  const [query, setQuery] = useState("");
  const [isListening, setIsListening] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [response, setResponse] = useState("");
  const [thinking, setThinking] = useState("");
  const [showThinking, setShowThinking] = useState(false);
  const [isThinking, setIsThinking] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showModelSelector, setShowModelSelector] = useState(false);
  const [showRagWindow, setShowRagWindow] = useState(false);
  const [isExtracting, setIsExtracting] = useState(false);
  const [textareaHeight, setTextareaHeight] = useState(20);

  // RAG integration
  const [useRag, setUseRag] = useState(false);
  const [ragCollection] = useState('default_group'); // TODO: Add UI to select collection
  const [expandedSource, setExpandedSource] = useState<{messageId: string, sourceIdx: number} | null>(null);
  const { queryRagWithContext } = useRagQuery();

  // Direct Chat - Using custom hook
  const directChat = useDirectChat();
  
  const openRagWindow = async () => {
    try {
      await invoke('open_rag_storage_window');
    } catch (error) {
      console.error('Failed to create RAG window:', error);
      // Fallback to modal if window creation fails
      console.log('Falling back to modal');
      setShowRagWindow(true);
    }
  };

  const openConversationsWindow = async () => {
    try {
      await invoke('open_conversations_window');
    } catch (error) {
      console.error('Failed to create Conversations window:', error);
    }
  };

  const openSettingsWindow = async () => {
    try {
      await invoke('open_settings_window');
    } catch (error) {
      console.error('Failed to create Settings window:', error);
      // Fallback to modal if window creation fails
      console.log('Falling back to modal');
      setShowSettings(true);
    }
  };

  const openModelSelectorWindow = async () => {
    try {
      await invoke('open_model_selector_window');
    } catch (error) {
      console.error('Failed to create Model Selector window:', error);
      // Fallback to modal if window creation fails
      console.log('Falling back to modal');
      setShowModelSelector(true);
    }
  };
  const [currentModel, setCurrentModel] = useState(modelConfigStore.currentModel);
  const [conversationHistory, setConversationHistory] = useState<Array<{
    id: string;
    type: 'user' | 'assistant';
    content: string;
    thinking?: string;
    timestamp: Date;
    ragSources?: RagContextResponse;
    metrics?: {
      tokensPerSecond?: number;
      totalTokens?: number;
      inputTokens?: number;
      outputTokens?: number;
      timeToFirstToken?: number;
      processingTime?: number;
      stopReason?: string;
    };
  }>>([]);

  // Synchronisation immédiate au montage
  useEffect(() => {
    // Forcer une synchronisation avec le store au montage
    const storeModel = modelConfigStore.currentModel;
    if (storeModel.id !== currentModel.id) {
      console.log('🔄 Initial sync - updating to store model:', storeModel);
      setCurrentModel(storeModel);
    }
  }, []);

  // Update current model when store changes - Using Tauri Events
  useEffect(() => {
    console.log('=== COMMAND INTERFACE TAURI MODEL LISTENER SETUP ===');
    console.log('Initial model:', modelConfigStore.currentModel);
    console.log('Current model state:', currentModel);
    console.log('Window location:', window.location.href);
    
    // Écouter les événements Tauri natifs (solution principale)
    const unsubscribeTauri = tauriModelStore.onModelChanged((newModel) => {
      setCurrentModel(newModel);
    });

    // Écouter les changements de paramètres
    const unsubscribeParameters = tauriModelStore.onParametersChanged((newParameters) => {
      console.log('🔧 CommandInterface: Received parameters update:', newParameters);
      // Les paramètres sont déjà mis à jour dans modelConfigStore par tauriModelStore
      // Pas besoin de state local pour les paramètres
    });
    
    // Storage events (fallback pour compatibilité)
    const updateModelFromStorage = (event: any) => {
      if (event?.key === 'gravis-config') {
        const newModel = modelConfigStore.currentModel;
        setCurrentModel(newModel);
      }
    };
    
    window.addEventListener('storage', updateModelFromStorage);
    console.log('📦 Storage fallback listener added');
    
    // Cleanup
    return () => {
      console.log('🧹 Cleaning up model listeners in main window');
      unsubscribeTauri();
      unsubscribeParameters();
      window.removeEventListener('storage', updateModelFromStorage);
    };
  }, []);

  // Separate polling effect with currentModel dependency to avoid stale closure
  useEffect(() => {
    const pollInterval = setInterval(() => {
      const storeModel = modelConfigStore.currentModel;
      if (storeModel.id !== currentModel.id) {
        console.log('🔄 Model change detected via polling backup');
        console.log('Store model:', storeModel);
        console.log('Current model:', currentModel);
        console.log('📝 About to call setCurrentModel with:', storeModel);
        setCurrentModel(storeModel);
        console.log('✅ setCurrentModel called');
      }
    }, 2000);
    console.log('🔄 Polling backup started');
    
    return () => {
      clearInterval(pollInterval);
    };
  }, [currentModel]); // Include currentModel in dependencies to fix stale closure

  const handleVoiceInput = () => {
    setIsListening(!isListening);
    // TODO: Implement voice input
  };

  // Fonction sécurisée pour ouvrir URLs dans le navigateur externe
  const openExternalUrl = async (url: string) => {
    try {
      console.log(`🌐 Ouverture URL externe avec openUrl: ${url}`);
      await openUrl(url);
      console.log('✅ URL ouverte avec succès dans le navigateur externe');
    } catch (error) {
      console.error('❌ Erreur openUrl, tentative fallback:', error);
      try {
        // Fallback : utiliser window.open avec target _blank
        window.open(url, '_blank', 'noopener,noreferrer');
        console.log('✅ Fallback window.open réussi');
      } catch (fallbackError) {
        console.error('❌ Fallback window.open échoué:', fallbackError);
        alert(`Impossible d'ouvrir l'URL: ${url}`);
      }
    }
  };


  const handleAWCSExtraction = async (fromGlobalShortcut = false) => {
    if (isExtracting || isProcessing) return;
    
    setIsExtracting(true);
    
    try {
      // Pour le raccourci global, ajouter un petit délai pour permettre le changement de focus
      if (fromGlobalShortcut) {
        console.log('🔥 AWCS Phase 4: Extraction déclenchée par raccourci global ⌘⇧⌃L');
        await new Promise(resolve => setTimeout(resolve, 200));
      }
      
      // Déclencher l'extraction AWCS
      const context = await invoke('awcs_get_current_context') as any;
      
      if (context && context.content && context.content.fulltext) {
        const extractedText = context.content.fulltext;
        const appName = context.source.app;
        const confidence = context.confidence;
        
        // Créer un message contextuel avec le contenu extrait
        const triggerSource = fromGlobalShortcut ? " (via ⌘⇧⌃L)" : "";
        const contextMessage = `📋 Contenu extrait de ${appName}${triggerSource} (${Math.round(confidence.text_completeness * 100)}% fiable):

"${extractedText}"

Question à propos de ce contenu : `;
        
        // Injecter dans l'input du chat
        setQuery(contextMessage);
        
        if (fromGlobalShortcut) {
          console.log('✅ AWCS Phase 4: Contenu automatiquement injecté dans le chat');
        }
      } else {
        // Si pas de contenu, indiquer l'échec
        const triggerSource = fromGlobalShortcut ? " (via ⌘⇧⌃L)" : "";
        setQuery(`❌ Aucun contenu extrait${triggerSource}. Essayez de changer de fenêtre et réessayez.`);
      }
    } catch (error) {
      console.error('Erreur extraction AWCS:', error);
      const triggerSource = fromGlobalShortcut ? " (via ⌘⇧⌃L)" : "";
      setQuery(`❌ Erreur d'extraction${triggerSource}: ${error}`);
    } finally {
      setIsExtracting(false);
    }
  };

  // Debug useEffect to track states
  useEffect(() => {
    console.log('State update - isThinking:', isThinking, 'thinking length:', thinking?.length || 0, 'isProcessing:', isProcessing, 'clickable condition:', isThinking);
  }, [isThinking, thinking, isProcessing]);

  // Listen for conversation resume events from other windows
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupEventListener = async () => {
      try {
        unlisten = await listen('resume_conversation', (event: any) => {
          console.log('📥 Événement de reprise de conversation reçu:', event.payload);
          const { conversation } = event.payload;
          
          if (conversation && conversation.id) {
            // Reprendre la conversation dans le gestionnaire
            const resumedConversation = conversationManager.resumeConversation(conversation.id);
            
            if (resumedConversation) {
              // Charger l'historique dans l'interface
              const historyMessages = resumedConversation.messages.map(msg => ({
                id: msg.id,
                type: msg.role as 'user' | 'assistant',
                content: msg.content,
                timestamp: msg.timestamp
              }));
              
              setConversationHistory(historyMessages);
              console.log('✅ Conversation reprise avec succès:', resumedConversation.title);
              
              // Clear current response/thinking state
              setResponse("");
              setThinking("");
              setShowThinking(false);
              setIsThinking(false);
            }
          }
        });
      } catch (error) {
        console.error('❌ Erreur lors de l\'écoute des événements:', error);
      }
    };

    setupEventListener();
    
    // Cleanup function
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Listen for global shortcut events (AWCS Phase 4)
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupGlobalShortcutListener = async () => {
      try {
        // Tauri event listener for backend events
        unlisten = await listen('awcs-shortcut-triggered', async (event: any) => {
          console.log('🔥 AWCS Phase 4: Global shortcut triggered! Extracting content...', event);
          
          // Automatically trigger AWCS extraction when global shortcut is pressed
          await handleAWCSExtraction(true);
        });
        
        // Custom window event listener for frontend events
        const handleCustomShortcut = async (event: CustomEvent) => {
          console.log('🔥 AWCS Phase 4: Custom shortcut event triggered!', event.detail);
          await handleAWCSExtraction(true);
        };
        
        window.addEventListener('awcs-global-shortcut-triggered', handleCustomShortcut as any);
        
        console.log('✅ AWCS Phase 4: Global shortcut listeners setup completed');
      } catch (error) {
        console.error('❌ AWCS Phase 4: Failed to setup global shortcut listener:', error);
      }
    };

    setupGlobalShortcutListener();
    
    return () => {
      if (unlisten) {
        unlisten();
      }
      // Clean up custom event listener
      window.removeEventListener('awcs-global-shortcut-triggered', () => {});
    };
  }, []);

  // Listen for browser extension content events
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupExtensionContentListener = async () => {
      try {
        unlisten = await listen('extension-content-received', (event: any) => {
          console.log('📥 Extension content received:', event.payload);
          
          // Show brief extraction indicator
          setIsExtracting(true);
          
          // Inject the formatted content into the chat input
          if (typeof event.payload === 'string') {
            setQuery(event.payload);
          }
          
          // Clear extraction indicator after a short delay
          setTimeout(() => {
            setIsExtracting(false);
          }, 500);
        });
        
        console.log('✅ Extension content listener setup completed');
      } catch (error) {
        console.error('❌ Failed to setup extension content listener:', error);
      }
    };

    setupExtensionContentListener();
    
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Intercepteur ciblé pour les liens dans les réponses des modèles IA
  useEffect(() => {
    const interceptResponseLinks = () => {
      // Intercepter seulement les clics dans la zone de réponse
      const handleResponseLinkClick = (e: Event) => {
        const target = e.target as HTMLElement;
        const link = target.closest('a') as HTMLAnchorElement;
        
        if (link && link.href) {
          // Vérifier si le lien est dans une zone de contenu assistant
          const isInAssistantContent = link.closest('.assistant-content, .assistant-message, .conversation-history');
          
          // Intercepter seulement les liens externes dans les réponses de l'assistant
          if (isInAssistantContent && (link.href.startsWith('http://') || link.href.startsWith('https://'))) {
            e.preventDefault();
            e.stopPropagation();
            console.log('🔗 Lien assistant intercepté, ouverture externe:', link.href);
            openExternalUrl(link.href);
          }
        }
      };

      // Ajouter l'intercepteur avec capture
      document.addEventListener('click', handleResponseLinkClick, true);

      return () => {
        document.removeEventListener('click', handleResponseLinkClick, true);
      };
    };

    return interceptResponseLinks();
  }, []);

  // Auto-resize window based on conversation content and textarea height
  useEffect(() => {
    const resizeWindow = async () => {
      try {
        const window = getCurrentWindow();
        const fileBadgeHeight = directChat.droppedFile ? 40 : 0; // Add height for file badge if present

        if (conversationHistory.length > 0 || isProcessing) {
          // Expand to 400px when there's content + file badge height
          const totalHeight = 400 + fileBadgeHeight;
          await window.setSize(new LogicalSize(500, totalHeight));
        } else {
          // Calculate height based on textarea size + file badge height
          const baseHeight = 150; // Base window height
          const extraHeight = Math.max(0, textareaHeight - 20); // Extra height for textarea expansion
          const newHeight = baseHeight + extraHeight + fileBadgeHeight;
          await window.setSize(new LogicalSize(500, newHeight));
        }
      } catch (error) {
        console.error('Failed to resize window:', error);
      }
    };

    resizeWindow();
  }, [conversationHistory.length, isProcessing, textareaHeight, directChat.droppedFile]);

  // Helper function to add assistant response to conversation history
  const addAssistantResponse = (content: string, thinkingContent?: string, metrics?: any, ragSources?: RagContextResponse) => {
    // Ajouter la réponse de l'assistant au gestionnaire de conversations
    if (conversationManager.getCurrentConversation()) {
      conversationManager.addMessage('assistant', content);
      conversationManager.saveCurrentConversation();
      console.log('💾 Réponse assistant sauvegardée dans la conversation');
    }

    const assistantMessage = {
      id: (Date.now() + 1).toString(),
      type: 'assistant' as const,
      content,
      thinking: thinkingContent,
      timestamp: new Date(),
      ragSources,
      metrics
    };
    setConversationHistory(prev => [...prev, assistantMessage]);
  };

  // Function to start a new conversation
  const handleNewConversation = () => {
    // Terminer la conversation actuelle avant d'en démarrer une nouvelle
    conversationManager.endCurrentConversation();
    console.log('🔄 Nouvelle conversation démarrée');

    setConversationHistory([]);
    setResponse("");
    setThinking("");
    setShowThinking(false);
    setIsThinking(false);
    setQuery("");

    // Reset direct chat states
    directChat.resetDirectChat();
  };

  // Handle direct chat with dropped document - Wrapper autour du hook
  const handleDirectChat = async (userQuery: string) => {
    // Add user message
    const userMessage = {
      id: Date.now().toString(),
      type: 'user' as const,
      content: userQuery,
      timestamp: new Date(),
    };
    setConversationHistory(prev => [...prev, userMessage]);
    setQuery("");

    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = '20px';
      setTextareaHeight(20);
    }

    setIsProcessing(true);
    setResponse("");

    // Use the directChat hook
    const result = await directChat.chatWithDocument(userQuery);

    if (result.success) {
      const assistantMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant' as const,
        content: result.content,
        timestamp: new Date(),
      };
      setConversationHistory(prev => [...prev, assistantMessage]);
    } else {
      const errorMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant' as const,
        content: result.content,
        timestamp: new Date(),
      };
      setConversationHistory(prev => [...prev, errorMessage]);
    }

    setIsProcessing(false);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || isProcessing) return;

    // If direct chat session exists, use direct chat command
    if (directChat.hasActiveSession) {
      await handleDirectChat(query.trim());
      return;
    }

    // Vérifier qu'un modèle valide est sélectionné
    if (!currentModel || !currentModel.id || currentModel.id === '') {
      alert('Veuillez sélectionner un modèle avant d\'envoyer un message.');
      return;
    }

    const userQuery = query.trim();
    
    // Démarrer ou continuer une conversation avec le gestionnaire
    let conversation = conversationManager.getCurrentConversation();
    if (!conversation) {
      conversation = conversationManager.startNewConversation(userQuery, modelConfigStore.currentModel.name);
      console.log('📝 Nouvelle conversation créée:', conversation.title);
    } else {
      conversationManager.addMessage('user', userQuery);
    }
    
    // Add user message to conversation history
    const userMessage = {
      id: Date.now().toString(),
      type: 'user' as const,
      content: userQuery,
      timestamp: new Date()
    };
    
    setConversationHistory(prev => [...prev, userMessage]);
    setQuery(""); // Clear input immediately
    
    // Reset textarea height to default
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = '20px'; // Default min height
      setTextareaHeight(20); // Update state
    }

    setIsProcessing(true);
    setResponse("");
    setThinking("");
    setShowThinking(false);
    setIsThinking(false);

    let fullResponse = "";
    let fullThinking = "";
    
    // Performance tracking variables
    const startTime = Date.now();
    let firstTokenTime: number | null = null;
    let totalTokens = 0;
    let inputTokens = 0;
    let outputTokens = 0;

    try {
      console.log('🚀 Starting handleSubmit...');
      const config = modelConfigStore.getConfig();
      console.log('🔧 Got config:', config);
      console.log('🔧 System prompt from config:', config.systemPrompt);
      console.log('🔧 Model parameters from store:', modelConfigStore.modelParameters);
      
      // Utiliser les paramètres les plus récents du store
      const currentSystemPrompt = modelConfigStore.modelParameters.systemPrompt || config.systemPrompt;
      console.log('🔧 Final system prompt to use:', currentSystemPrompt);
      
      if (!config.apiKey && 
          modelConfigStore.currentModel.provider !== 'Ollama' && 
          modelConfigStore.currentModel.provider !== 'Ollama (Local)') {
        console.log('❌ Missing API key for non-Ollama model');
        setResponse("⚠️ Configuration manquante : Veuillez configurer votre clé API dans les paramètres du modèle.");
        return;
      }
      
      console.log('✅ Config validation passed');

      // Check if current model supports thinking
      const currentModel = modelConfigStore.currentModel;
      const supportsThinking = currentModel.id.includes('deepseek-reasoner') || 
                              currentModel.id.includes('thinking') ||
                              currentModel.id.includes('deepseek') ||
                              currentModel.id.includes('Qwen3-8B-FP8') || // Modal Qwen3-8B-FP8 has thinking
                              currentModel.provider?.includes('Modal') || // All Modal models potentially have thinking
                              (currentModel.description && currentModel.description.toLowerCase().includes('reasoning'));
      
      console.log('Current model:', currentModel);
      console.log('Supports thinking:', supportsThinking);
      
      if (supportsThinking) {
        setIsThinking(true);
      }

      console.log('🔧 Creating LiteLLMClient with config:', config);
      const client = new LiteLLMClient(config);
      console.log('✅ LiteLLMClient created successfully');

      // RAG Context Integration
      let ragContextText = "";
      let ragSources: RagContextResponse | null = null;

      if (useRag) {
        console.log('🔍 RAG enabled, querying collection:', ragCollection);
        const ragResponse = await queryRagWithContext({
          query: userQuery,
          groupId: ragCollection,
          limit: 5
        });

        if (ragResponse && ragResponse.total_chunks > 0) {
          ragContextText = ragResponse.formatted_context;
          ragSources = ragResponse;
          console.log(`✅ RAG context retrieved: ${ragResponse.total_chunks} chunks from ${ragResponse.sources.length} sources`);
        } else {
          console.log('⚠️ No RAG context found for query');
        }
      }

      // Build messages with optional RAG context
      const messages = [
        {
          role: "system",
          content: `RÔLE OBLIGATOIRE : ${currentSystemPrompt || "Tu es GRAVIS, un assistant spécialisé dans l'audit et l'analyse de code. Réponds de manière concise et professionnelle."} Tu DOIS impérativement respecter ce rôle dans toutes tes réponses.`
        },
        {
          role: "user",
          content: ragContextText
            ? `${ragContextText}\n\n---\n\n**Question de l'utilisateur**: ${userQuery}`
            : userQuery
        }
      ];
      
      console.log('🔧 Messages being sent to API:', JSON.stringify(messages, null, 2));

      // Use streaming for thinking models to show real-time reasoning
      if (supportsThinking) {
        const stream = await client.chatStream(messages);
        
        if (stream) {
          const reader = stream.getReader();
          const decoder = new TextDecoder();
          
          try {
            while (true) {
              const { done, value } = await reader.read();
              if (done) break;
              
              const chunk = decoder.decode(value, { stream: true });
              const lines = chunk.split('\n');
              
              for (const line of lines) {
                if (line.startsWith('data: ')) {
                  const data = line.slice(6);
                  if (data === '[DONE]') continue;
                  
                  try {
                    const parsed = JSON.parse(data);
                    if (parsed.choices && parsed.choices[0]) {
                      const delta = parsed.choices[0].delta;
                      
                      // Handle reasoning content (DeepSeek format)
                      if (delta.reasoning) {
                        fullThinking += delta.reasoning;
                        setThinking(fullThinking);
                        console.log('🧠 DeepSeek thinking received, length:', fullThinking.length);
                      }
                      
                      // Handle main content
                      if (delta.content) {
                        // Mark first token time
                        if (!firstTokenTime && delta.content.trim()) {
                          firstTokenTime = Date.now();
                        }
                        
                        fullResponse += delta.content;
                        
                        // Parse thinking from content (Modal/vLLM format)
                        const parsed = parseThinkingStream(fullResponse);
                        
                        if (parsed.thinking && parsed.thinking !== fullThinking) {
                          fullThinking = parsed.thinking;
                          
                          // Protection contre les boucles infinies en temps réel (désactivée temporairement)
                          // if (detectThinkingLoop(fullThinking)) {
                          //   fullThinking = cleanThinkingLoops(fullThinking);
                          //   console.log('🔧 Real-time loop protection activated');
                          // }
                          
                          setThinking(fullThinking);
                          console.log('🧠 Modal thinking parsed, length:', fullThinking.length);
                        }
                        
                        // Set only the main content without thinking tags
                        setResponse(parsed.content);
                      }
                    }
                  } catch (e) {
                    // Ignore parsing errors for incomplete chunks
                  }
                }
              }
            }
          } finally {
            reader.releaseLock();
          }
        }
        
        // Calculate metrics for streaming response
        const endTime = Date.now();
        const processingTime = endTime - startTime;
        const timeToFirstToken = firstTokenTime ? firstTokenTime - startTime : 0;
        const estimatedInputTokens = Math.round(userQuery.length / 4); // Rough estimation
        const estimatedOutputTokens = Math.round(fullResponse.length / 4); // Rough estimation
        const estimatedTotalTokens = estimatedInputTokens + estimatedOutputTokens;
        const tokensPerSecond = processingTime > 0 ? (estimatedOutputTokens / (processingTime / 1000)) : 0;
        
        const metrics = {
          tokensPerSecond: Math.round(tokensPerSecond * 100) / 100,
          totalTokens: estimatedTotalTokens,
          inputTokens: estimatedInputTokens,
          outputTokens: estimatedOutputTokens,
          timeToFirstToken: timeToFirstToken,
          processingTime: processingTime
        };

        // Add streaming response to conversation history (with cleaned content)
        if (fullResponse) {
          const finalParsed = parseThinkingStream(fullResponse);
          addAssistantResponse(finalParsed.content, fullThinking || finalParsed.thinking, metrics, ragSources || undefined);
        }
      } else {
        // Non-thinking models use regular chat
        const result = await client.chat(messages);
        
        if (result.choices && result.choices[0]) {
          const choice = result.choices[0];
          
          console.log('Choice message:', choice.message);
          
          // Calculate metrics for non-streaming response
          const endTime = Date.now();
          const processingTime = endTime - startTime;
          
          // Extract usage information from API response
          if (result.usage) {
            inputTokens = result.usage.prompt_tokens || 0;
            outputTokens = result.usage.completion_tokens || 0;
            totalTokens = result.usage.total_tokens || (inputTokens + outputTokens);
          } else {
            // Fallback estimation if usage not provided
            inputTokens = Math.round(userQuery.length / 4);
            outputTokens = Math.round(choice.message.content.length / 4);
            totalTokens = inputTokens + outputTokens;
          }
          
          const tokensPerSecond = processingTime > 0 ? (outputTokens / (processingTime / 1000)) : 0;
          
          const metrics = {
            tokensPerSecond: Math.round(tokensPerSecond * 100) / 100,
            totalTokens: totalTokens,
            inputTokens: inputTokens,
            outputTokens: outputTokens,
            timeToFirstToken: processingTime, // For non-streaming, this is the full time
            processingTime: processingTime,
            stopReason: choice.finish_reason || 'completed'
          };
          
          // Handle thinking models (like DeepSeek) - fallback for non-streaming
          let finalThinking = "";
          let finalContent = choice.message.content;
          
          if (choice.message.reasoning) {
            console.log('Found reasoning:', choice.message.reasoning);
            finalThinking = choice.message.reasoning;
            setThinking(finalThinking);
          } else {
            // Parse thinking from content if present (Modal format)
            const parsed = parseThinkingStream(choice.message.content);
            if (parsed.thinking) {
              finalThinking = parsed.thinking;
              finalContent = parsed.content;
              setThinking(finalThinking);
            }
          }
          
          setResponse(finalContent);

          // Add assistant response to conversation history
          addAssistantResponse(finalContent, finalThinking, metrics, ragSources || undefined);
        } else {
          const errorMessage = "❌ Erreur: Réponse invalide du modèle";
          setResponse(errorMessage);
          addAssistantResponse(errorMessage);
        }
      }
    } catch (error) {
      console.error("Error processing command:", error);
      const errorMessage = `❌ Erreur: ${error instanceof Error ? error.message : "Erreur inconnue"}`;
      setResponse(errorMessage);
      
      // Add error response to conversation history
      addAssistantResponse(errorMessage);
    } finally {
      setIsProcessing(false);
    }
  };

  // Éviter l'erreur TypeScript - utiliser response
  if (response.length > 10000) console.log('Long response detected');

  return (
    <div
      className="search-popup"
      {...directChat.dragHandlers}
    >
      {/* Drag area on top */}
      <div className="top-bar" data-tauri-drag-region>
      </div>

      {/* Search container */}
      <div className="search-container">
        <div className="drag-handle" data-tauri-drag-region></div>
        <form onSubmit={handleSubmit}>
          <div
            className="search-input-wrapper"
            style={{
              ...(directChat.isDragging && {
                border: '2px dashed #3b82f6',
                boxShadow: '0 0 0 3px rgba(59, 130, 246, 0.2)',
              })
            }}
          >
            <textarea
              ref={textareaRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Poser une question"
              className="search-input"
              autoFocus
              disabled={isProcessing}
              rows={1}
              style={{
                resize: 'none',
                overflow: 'hidden',
                minHeight: '20px',
                maxHeight: '120px'
              }}
              onInput={(e) => {
                const target = e.target as HTMLTextAreaElement;
                target.style.height = 'auto';
                const newHeight = Math.min(target.scrollHeight, 120);
                target.style.height = newHeight + 'px';
                setTextareaHeight(newHeight);
              }}
            />
            {!query.trim() ? (
              <button
                type="button"
                onClick={handleVoiceInput}
                className={`mic-button ${isListening ? "listening" : ""}`}
                title="Microphone"
              >
                <Mic size={16} />
              </button>
            ) : (
              <button
                type="submit"
                className="send-button"
                title="Envoyer"
                disabled={isProcessing}
              >
                <Send size={16} />
              </button>
            )}
          </div>
        </form>

        {/* Dropped File Badge */}
        {directChat.droppedFile && (
          <FileBadge
            fileName={directChat.droppedFile.name}
            onRemove={directChat.removeDroppedFile}
          />
        )}

        {/* AWCS Global Shortcut Tip */}
        <div className="shortcut-tip" style={{
          fontSize: '11px',
          color: '#666',
          textAlign: 'left',
          marginTop: '4px',
          opacity: 0.8
        }}>
          Astuce : Utilisez <kbd style={{
            background: '#f0f0f0',
            padding: '1px 4px',
            borderRadius: '3px',
            fontSize: '10px'
          }}>⌘⇧⌃L</kbd> depuis n'importe quelle app pour extraire du contenu
        </div>

        {/* RAG Status Indicator */}
        {useRag && (
          <div style={{
            fontSize: '11px',
            color: '#10b981',
            textAlign: 'left',
            marginTop: '4px',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}>
            <Database size={12} />
            <span>RAG activé • Collection: {ragCollection}</span>
          </div>
        )}
      </div>

      {/* Action buttons */}
      <div className="action-buttons">
        <div className="button-group">
          <button 
            type="button" 
            className="icon-button" 
            title="Nouvelle conversation"
            onClick={handleNewConversation}
          >
            <Plus size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Ouvrir une URL dans le navigateur externe"
            onClick={async () => {
              const url = prompt('Entrez l\'URL à ouvrir dans le navigateur externe:');
              if (url) {
                // Ajouter https:// si pas de protocole
                const finalUrl = url.startsWith('http://') || url.startsWith('https://') 
                  ? url 
                  : `https://${url}`;
                await openExternalUrl(finalUrl);
              }
            }}
          >
            <Globe size={14} />
          </button>
          <button
            type="button"
            className="icon-button"
            title="RAG - Gestion des documents"
            onClick={openRagWindow}
          >
            <FileText size={14} />
          </button>
          <button
            type="button"
            className={`icon-button ${useRag ? 'active' : ''}`}
            title={useRag ? "RAG activé - Cliquez pour désactiver" : "RAG désactivé - Cliquez pour activer"}
            onClick={() => setUseRag(!useRag)}
            style={{
              backgroundColor: useRag ? '#10b981' : 'transparent',
              color: useRag ? 'white' : 'inherit'
            }}
          >
            <Database size={14} />
          </button>
          <button type="button" className="icon-button" title="MCP">
            <Radio size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Historique des conversations"
            onClick={openConversationsWindow}
          >
            <MessageSquare size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Sélectionner le modèle"
            onClick={openModelSelectorWindow}
          >
            <Bot size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Configuration"
            onClick={openSettingsWindow}
          >
            <Settings size={14} />
          </button>
          
          
          <div 
            className={`model-name-display ${isThinking ? 'clickable' : ''}`}
            onClick={isThinking ? () => {
              console.log('Badge clicked! isThinking:', isThinking, 'thinking:', !!thinking, 'showThinking:', showThinking);
              setShowThinking(!showThinking);
            } : undefined}
            style={{ cursor: isThinking ? 'pointer' : 'default' }}
            title={isExtracting ? 'Extracting content...' : (isProcessing ? (isThinking ? 'Processing... (click to view thinking)' : 'Processing...') : `Ready - ${currentModel?.name || currentModel?.id}`)}
          >
            <div 
              className={`status-dot ${isExtracting ? 'extracting' : (isProcessing ? (isThinking ? 'thinking' : 'processing') : 'ready')}`}
            ></div>
            <span>
              {isExtracting
                ? 'Extracting'
                : (isProcessing 
                  ? 'Processing' 
                  : (currentModel?.name || currentModel?.id || 'No Model'))
              }
            </span>
          </div>
        </div>

      </div>

      {/* Chat conversation area */}
      {(conversationHistory.length > 0 || isProcessing) && (
        <div className="chat-container">
          {/* Conversation history */}
          {conversationHistory.map((message) => (
            <div key={message.id} className={`chat-message ${message.type}`}>
              {message.type === 'user' ? (
                <div className="user-message">
                  {message.content}
                </div>
              ) : (
                <div className="assistant-message">
                  {/* Thinking section */}
                  {message.thinking && (
                    <div className="thinking-section">
                      <div className="thinking-header" onClick={() => {
                        const thinkingContent = document.getElementById(`thinking-${message.id}`);
                        if (thinkingContent) {
                          thinkingContent.style.display = thinkingContent.style.display === 'none' ? 'block' : 'none';
                        }
                      }}>
                        <span>🧠 Réflexion du modèle</span>
                        <span className="thinking-toggle">▼</span>
                      </div>
                      <div id={`thinking-${message.id}`} className="thinking-content" style={{display: 'none'}}>
                        <pre>{message.thinking}</pre>
                      </div>
                    </div>
                  )}
                  <div className="assistant-content">
                    <ReactMarkdown 
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeHighlight]}
                      components={{
                        a: ({ href, children, ...props }) => {
                          // Ouvrir les liens externes dans le navigateur
                          if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
                            return (
                              <a 
                                {...props}
                                href={href}
                                onClick={(e) => {
                                  e.preventDefault();
                                  console.log('🔗 Lien ReactMarkdown cliqué:', href);
                                  openExternalUrl(href);
                                }}
                                className="external-link"
                                title={`Ouvrir dans le navigateur: ${href}`}
                              >
                                {children}
                              </a>
                            );
                          }
                          // Liens internes normaux
                          return <a {...props} href={href}>{children}</a>;
                        }
                      }}
                    >
                      {message.content}
                    </ReactMarkdown>
                  </div>
                  
                  {/* RAG Sources */}
                  {message.ragSources && message.ragSources.sources.length > 0 && (
                    <div style={{
                      marginTop: '12px',
                      padding: '12px',
                      backgroundColor: 'rgba(16, 185, 129, 0.1)',
                      borderLeft: '3px solid #10b981',
                      borderRadius: '4px',
                      fontSize: '12px'
                    }}>
                      <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#10b981' }}>
                        📚 Sources RAG ({message.ragSources.total_chunks} chunks en {message.ragSources.search_time_ms}ms)
                      </div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                        {message.ragSources.sources.map((source, idx) => {
                          const isExpanded = expandedSource?.messageId === message.id && expandedSource?.sourceIdx === idx;
                          return (
                            <div key={source.chunk_id} style={{
                              padding: '8px',
                              backgroundColor: 'rgba(255, 255, 255, 0.5)',
                              borderRadius: '4px',
                              fontSize: '11px',
                              cursor: 'pointer',
                              border: isExpanded ? '2px solid #10b981' : '1px solid rgba(0,0,0,0.1)',
                              transition: 'all 0.2s'
                            }}
                            onClick={() => {
                              if (isExpanded) {
                                setExpandedSource(null);
                              } else {
                                setExpandedSource({ messageId: message.id, sourceIdx: idx });
                              }
                            }}
                            >
                              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                                <span style={{ fontWeight: 'bold', display: 'flex', alignItems: 'center', gap: '6px' }}>
                                  <span style={{
                                    backgroundColor: '#10b981',
                                    color: 'white',
                                    padding: '2px 6px',
                                    borderRadius: '3px',
                                    fontSize: '10px'
                                  }}>
                                    Source {idx + 1}
                                  </span>
                                  {source.source_file || 'Document'}
                                </span>
                                <span style={{ color: '#10b981', fontWeight: 'bold' }}>
                                  {(source.score * 100).toFixed(1)}%
                                </span>
                              </div>
                              {source.document_category && (
                                <div style={{
                                  marginBottom: '6px',
                                  fontSize: '10px',
                                  display: 'inline-block',
                                  padding: '2px 6px',
                                  backgroundColor: '#e0e7ff',
                                  color: '#4338ca',
                                  borderRadius: '3px'
                                }}>
                                  {source.document_category}
                                </div>
                              )}
                              <div style={{
                                color: '#555',
                                fontStyle: 'italic',
                                marginTop: '6px',
                                maxHeight: isExpanded ? 'none' : '40px',
                                overflow: isExpanded ? 'visible' : 'hidden',
                                lineHeight: '1.4'
                              }}>
                                {source.content_preview}
                              </div>
                              {!isExpanded && (
                                <div style={{
                                  marginTop: '4px',
                                  color: '#10b981',
                                  fontSize: '10px',
                                  textAlign: 'right'
                                }}>
                                  📖 Cliquez pour voir plus...
                                </div>
                              )}
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  )}

                  {/* Performance metrics */}
                  {message.metrics && (
                    <div className="performance-metrics">
                      <div className="metrics-row">
                        {message.metrics.tokensPerSecond && (
                          <span className="metric metric-speed">
                            <strong>{message.metrics.tokensPerSecond}</strong> <span className="metric-label">tok/sec</span>
                          </span>
                        )}
                        {(message.metrics.totalTokens || message.metrics.outputTokens) && (
                          <span className="metric metric-tokens">
                            <strong>{message.metrics.totalTokens || message.metrics.outputTokens}</strong> <span className="metric-label">tokens</span>
                            {message.metrics.totalTokens && message.metrics.inputTokens && message.metrics.outputTokens && (
                              <span style={{ color: 'rgba(255, 255, 255, 0.5)', marginLeft: '4px', fontSize: '10px' }}>
                                ({message.metrics.inputTokens}+{message.metrics.outputTokens})
                              </span>
                            )}
                          </span>
                        )}
                        {message.metrics.timeToFirstToken && (
                          <span className="metric metric-timing">
                            <strong>{(message.metrics.timeToFirstToken / 1000).toFixed(2)}s</strong> <span className="metric-label">to first token</span>
                          </span>
                        )}
                      </div>
                    </div>
                  )}
                  
                  {/* Action buttons */}
                  <div className="message-actions">
                    <button className="action-btn" title="Copier">
                      <Copy size={14} />
                    </button>
                    <button className="action-btn" title="Audio">
                      <Volume2 size={14} />
                    </button>
                    <button className="action-btn" title="Régénérer">
                      <RotateCcw size={14} />
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}
          
          {/* Processing indicator */}
          {isProcessing && (
            <div className="chat-message assistant">
              <div className="assistant-message">
                {/* Thinking section en temps réel */}
                {thinking && (
                  <div className="thinking-section live">
                    <div className="thinking-header live">
                      <span>🧠 Réflexion en cours...</span>
                      <Loader2 className="thinking-spinner" size={16} style={{animation: 'spin 1s linear infinite'}} />
                    </div>
                    <div className="thinking-content live">
                      <pre>{thinking}</pre>
                    </div>
                  </div>
                )}
                
                {/* Réponse en temps réel */}
                {response && (
                  <div className="assistant-content live">
                    <ReactMarkdown 
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeHighlight]}
                    >
                      {response}
                    </ReactMarkdown>
                  </div>
                )}
                
                {!response && !thinking && (
                  <div className="processing-indicator">
                    <div className="processing-dots">
                      <span></span>
                      <span></span>
                      <span></span>
                    </div>
                    <span>Traitement en cours...</span>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Settings Modal - rendered outside via portal */}
      {showSettings && createPortal(
        <SettingsModal onClose={() => setShowSettings(false)} />,
        document.body
      )}

      {/* Model Selection Modal - rendered outside via portal */}
      {showModelSelector && createPortal(
        <ModelSelectorWindow 
          onClose={() => setShowModelSelector(false)}
        />,
        document.body
      )}

      {/* Thinking Modal - rendered outside via portal */}
      {showThinking && createPortal(
        <div style={{ 
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0,0,0,0.8)',
          color: '#ffffff',
          zIndex: 10000,
          overflow: 'auto',
          padding: '20px'
        }}>
          <div style={{ maxWidth: '800px', margin: '0 auto' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
              <h2>Réflexion du Modèle</h2>
              <button 
                onClick={() => {
                  console.log('Closing thinking modal');
                  setShowThinking(false);
                }}
                style={{ 
                  padding: '10px',
                  background: '#ef4444',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer'
                }}
              >
                ×
              </button>
            </div>
            <pre style={{ 
              background: '#1f2937',
              padding: '20px',
              borderRadius: '8px',
              overflow: 'auto',
              fontSize: '12px',
              lineHeight: '1.4',
              whiteSpace: 'pre-wrap'
            }}>
              {thinking || 'En attente de la réflexion du modèle...'}
            </pre>
          </div>
        </div>,
        document.body
      )}

      {/* RAG Window - Full screen overlay */}
      {showRagWindow && (
        <RagWindow
          onClose={() => setShowRagWindow(false)}
        />
      )}

      {/* OCR Viewer Panel - Split panel for direct chat */}
      {directChat.showOCRViewer && directChat.ocrContent && directChat.directChatSession && (
        <OCRPanel
          documentName={directChat.directChatSession.document_name}
          ocrContent={directChat.ocrContent}
          highlightedSpans={directChat.highlightedSpans}
          onSpanClick={(span) => {
            console.log('Span clicked:', span);
          }}
          onTextSelection={(text) => {
            console.log('Text selected:', text);
          }}
        />
      )}

    </div>
  );
}

function SettingsModal({ onClose }: { onClose: () => void }) {
  const [apiKey, setApiKey] = useState(modelConfigStore.apiKey);
  const [baseUrl, setBaseUrl] = useState(modelConfigStore.baseUrl);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const [testMessage, setTestMessage] = useState('');
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved'>('idle');

  const handleSave = () => {
    setSaveStatus('saving');
    
    try {
      modelConfigStore.setApiKey(apiKey);
      modelConfigStore.setBaseUrl(baseUrl);
      
      setSaveStatus('saved');
      
      // Show success message for 2 seconds then close
      setTimeout(() => {
        setSaveStatus('idle');
        onClose();
      }, 2000);
    } catch (error) {
      setSaveStatus('idle');
      console.error('Error saving configuration:', error);
    }
  };

  const handleTestConnection = async () => {
    if (!baseUrl.trim()) {
      setTestStatus('error');
      setTestMessage('URL requise');
      return;
    }

    setTestStatus('testing');
    setTestMessage('Test en cours...');

    try {
      const testClient = new LiteLLMClient({
        apiKey: apiKey || 'test',
        baseUrl: baseUrl.trim(),
        model: 'test'
      });

      // Test avec un appel simple aux modèles disponibles
      await testClient.getModels();
      
      setTestStatus('success');
      setTestMessage('Connexion réussie !');
      
      // Reset après 3 secondes
      setTimeout(() => {
        setTestStatus('idle');
        setTestMessage('');
      }, 3000);
      
    } catch (error) {
      setTestStatus('error');
      const errorMsg = error instanceof Error ? error.message : 'Erreur de connexion';
      setTestMessage(errorMsg);
      
      // Reset après 5 secondes
      setTimeout(() => {
        setTestStatus('idle');
        setTestMessage('');
      }, 5000);
    }
  };

  const getTestIcon = () => {
    switch (testStatus) {
      case 'testing':
        return <Loader2 size={14} className="animate-spin" />;
      case 'success':
        return <CheckCircle size={14} />;
      case 'error':
        return <XCircle size={14} />;
      default:
        return <Wifi size={14} />;
    }
  };

  const getTestText = () => {
    if (testMessage) return testMessage;
    return 'Tester';
  };

  return (
    <>
      <div className="dropdown-overlay" onClick={onClose} />
      <div className="settings-modal-fixed">
        <div className="settings-header">
          <h3>Configuration LiteLLM</h3>
          <button onClick={onClose} className="close-button">×</button>
        </div>
        
        <div className="settings-content">
          <div className="setting-group">
            <label>Base URL</label>
            <input
              type="text"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="http://localhost:4000"
              className="setting-input"
            />
            <div className="setting-help">
              URL de votre serveur LiteLLM (proxy ou direct)
            </div>
          </div>
          
          <div className="setting-group">
            <label>API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              className="setting-input"
            />
            <div className="setting-help">
              Clé API pour l'authentification (optionnelle selon config)
            </div>
          </div>
        </div>
        
        <div className="settings-footer">
          <button onClick={onClose} className="cancel-button">
            Annuler
          </button>
          <button 
            onClick={handleTestConnection}
            disabled={testStatus === 'testing'}
            className={`test-button ${testStatus}`}
          >
            {getTestIcon()}
            <span>{getTestText()}</span>
          </button>
          <button 
            onClick={handleSave} 
            className={`save-button ${saveStatus}`}
            disabled={saveStatus === 'saving'}
          >
            {saveStatus === 'saving' ? (
              <>
                <Loader2 size={14} className="animate-spin" />
                <span>Sauvegarde...</span>
              </>
            ) : saveStatus === 'saved' ? (
              <>
                <CheckCircle size={14} />
                <span>Sauvegardé !</span>
              </>
            ) : (
              'Sauvegarder'
            )}
          </button>
        </div>
      </div>
    </>
  );
}

