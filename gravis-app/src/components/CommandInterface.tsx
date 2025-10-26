import { useState, useEffect } from "react";
import { createPortal } from "react-dom";
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import { 
  Plus, 
  Globe, 
  FileText,
  Radio, 
  Search, 
  Mic,
  Settings,
  Wifi,
  CheckCircle,
  XCircle,
  Loader2,
  Bot,
  Zap,
  Brain,
  ChevronDown,
  ChevronUp,
  Copy,
  Volume2,
  ThumbsUp,
  ThumbsDown,
  RotateCcw
} from "lucide-react";
import { LiteLLMClient, modelConfigStore, AVAILABLE_MODELS } from "@/lib/litellm";
import { RagStore, RagClient, DocumentGroup } from "@/lib/rag";

export function CommandInterface() {
  const [query, setQuery] = useState("");
  const [isListening, setIsListening] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [response, setResponse] = useState("");
  const [thinking, setThinking] = useState("");
  const [showThinking, setShowThinking] = useState(false);
  const [isThinking, setIsThinking] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showModelSelector, setShowModelSelector] = useState(false);
  const [showRagModal, setShowRagModal] = useState(false);
  const [currentModel, setCurrentModel] = useState(modelConfigStore.currentModel);
  const [conversationHistory, setConversationHistory] = useState<Array<{
    id: string;
    type: 'user' | 'assistant';
    content: string;
    thinking?: string;
    timestamp: Date;
  }>>([]);

  // Update current model when store changes
  useEffect(() => {
    console.log('Initial model:', modelConfigStore.currentModel);
    console.log('Current model state:', currentModel);
    
    const updateModel = () => {
      console.log('Updating model from storage');
      setCurrentModel(modelConfigStore.currentModel);
    };
    
    // Listen for storage changes
    window.addEventListener('storage', updateModel);
    
    // Cleanup
    return () => {
      window.removeEventListener('storage', updateModel);
    };
  }, []);

  const handleVoiceInput = () => {
    setIsListening(!isListening);
    // TODO: Implement voice input
  };

  // Debug useEffect to track states
  useEffect(() => {
    console.log('State update - isThinking:', isThinking, 'thinking length:', thinking?.length || 0, 'isProcessing:', isProcessing, 'clickable condition:', isThinking);
  }, [isThinking, thinking, isProcessing]);

  // Auto-resize window based on conversation content
  useEffect(() => {
    const resizeWindow = async () => {
      try {
        const window = getCurrentWindow();
        if (conversationHistory.length > 0 || isProcessing) {
          // Expand to 400px when there's content
          await window.setSize(new LogicalSize(500, 400));
        } else {
          // Compact to 130px when no conversation
          await window.setSize(new LogicalSize(500, 130));
        }
      } catch (error) {
        console.error('Failed to resize window:', error);
      }
    };

    resizeWindow();
  }, [conversationHistory.length, isProcessing]);

  // Helper function to add assistant response to conversation history
  const addAssistantResponse = (content: string, thinkingContent?: string) => {
    const assistantMessage = {
      id: (Date.now() + 1).toString(),
      type: 'assistant' as const,
      content,
      thinking: thinkingContent,
      timestamp: new Date()
    };
    setConversationHistory(prev => [...prev, assistantMessage]);
  };

  // Function to start a new conversation
  const handleNewConversation = () => {
    setConversationHistory([]);
    setResponse("");
    setThinking("");
    setShowThinking(false);
    setIsThinking(false);
    setQuery("");
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || isProcessing) return;

    const userQuery = query.trim();
    
    // Add user message to conversation history
    const userMessage = {
      id: Date.now().toString(),
      type: 'user' as const,
      content: userQuery,
      timestamp: new Date()
    };
    
    setConversationHistory(prev => [...prev, userMessage]);
    setQuery(""); // Clear input immediately

    setIsProcessing(true);
    setResponse("");
    setThinking("");
    setShowThinking(false);
    setIsThinking(false);

    let fullResponse = "";
    let fullThinking = "";

    try {
      const config = modelConfigStore.getConfig();
      
      if (!config.apiKey) {
        setResponse("⚠️ Configuration manquante : Veuillez configurer votre clé API dans les paramètres du modèle.");
        return;
      }

      // Check if current model supports thinking
      const currentModel = modelConfigStore.currentModel;
      const supportsThinking = currentModel.id.includes('deepseek-reasoner') || 
                              currentModel.id.includes('thinking') ||
                              currentModel.id.includes('deepseek') ||
                              (currentModel.description && currentModel.description.toLowerCase().includes('reasoning'));
      
      console.log('Current model:', currentModel);
      console.log('Supports thinking:', supportsThinking);
      
      if (supportsThinking) {
        setIsThinking(true);
      }

      const client = new LiteLLMClient(config);
      
      const messages = [
        {
          role: "system",
          content: "Tu es GRAVIS, un assistant spécialisé dans l'audit et l'analyse de code. Réponds de manière concise et professionnelle."
        },
        {
          role: "user", 
          content: userQuery
        }
      ];

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
                      
                      // Handle reasoning content
                      if (delta.reasoning) {
                        fullThinking += delta.reasoning;
                        setThinking(fullThinking);
                        console.log('Thinking received, length:', fullThinking.length);
                        console.log('States - isThinking:', isThinking, 'thinking exists:', !!fullThinking);
                      }
                      
                      // Handle main content
                      if (delta.content) {
                        fullResponse += delta.content;
                        setResponse(fullResponse);
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
        
        // Add streaming response to conversation history
        if (fullResponse) {
          addAssistantResponse(fullResponse, fullThinking);
        }
      } else {
        // Non-thinking models use regular chat
        const result = await client.chat(messages);
        
        console.log('API Response:', result);
        
        if (result.choices && result.choices[0]) {
          const choice = result.choices[0];
          
          console.log('Choice message:', choice.message);
          
          // Handle thinking models (like DeepSeek) - fallback for non-streaming
          if (choice.message.reasoning) {
            console.log('Found reasoning:', choice.message.reasoning);
            setThinking(choice.message.reasoning);
            setShowThinking(true);
          }
          
          setResponse(choice.message.content);
          
          // Add assistant response to conversation history
          addAssistantResponse(choice.message.content, choice.message.reasoning);
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
      const assistantMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant' as const,
        content: errorMessage,
        timestamp: new Date()
      };
      setConversationHistory(prev => [...prev, assistantMessage]);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="search-popup">
      {/* Drag area on top */}
      <div className="top-bar" data-tauri-drag-region>
      </div>

      {/* Search container */}
      <div className="search-container">
        <div className="drag-handle" data-tauri-drag-region></div>
        <form onSubmit={handleSubmit}>
          <div className="search-input-wrapper">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Poser une question"
              className="search-input"
              autoFocus
              disabled={isProcessing}
            />
            <button
              type="button"
              onClick={handleVoiceInput}
              className={`mic-button ${isListening ? "listening" : ""}`}
              title="Microphone"
            >
              <Mic size={16} />
            </button>
          </div>
        </form>
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
          <button type="button" className="icon-button" title="Web">
            <Globe size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="RAG - Gestion des documents"
            onClick={() => setShowRagModal(true)}
          >
            <FileText size={14} />
          </button>
          <button type="button" className="icon-button" title="MCP">
            <Radio size={14} />
          </button>
          <button type="button" className="icon-button" title="Recherche">
            <Search size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Sélectionner le modèle"
            onClick={() => setShowModelSelector(!showModelSelector)}
          >
            <Bot size={14} />
          </button>
          <button 
            type="button" 
            className="icon-button" 
            title="Configuration"
            onClick={() => setShowSettings(!showSettings)}
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
            title={isProcessing ? (isThinking ? 'Processing... (click to view thinking)' : 'Processing...') : `Ready - ${currentModel?.name || currentModel?.id}`}
          >
            <div 
              className={`status-dot ${isProcessing ? (isThinking ? 'thinking' : 'processing') : 'ready'}`}
            ></div>
            <span>
              {isProcessing 
                ? 'Processing' 
                : (currentModel?.name || currentModel?.id || 'No Model')
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
                  <div className="assistant-content">
                    <ReactMarkdown 
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeHighlight]}
                    >
                      {message.content}
                    </ReactMarkdown>
                  </div>
                  
                  {/* Action buttons */}
                  <div className="message-actions">
                    <button className="action-btn" title="Copier">
                      <Copy size={14} />
                    </button>
                    <button className="action-btn" title="Audio">
                      <Volume2 size={14} />
                    </button>
                    <button className="action-btn" title="J'aime">
                      <ThumbsUp size={14} />
                    </button>
                    <button className="action-btn" title="Je n'aime pas">
                      <ThumbsDown size={14} />
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
                <div className="processing-indicator">
                  <div className="processing-dots">
                    <span></span>
                    <span></span>
                    <span></span>
                  </div>
                  <span>Traitement en cours...</span>
                </div>
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
        <ModelSelectionModal 
          onClose={() => setShowModelSelector(false)}
          onModelChange={(model) => setCurrentModel(model)}
        />,
        document.body
      )}

      {/* Thinking Modal - rendered outside via portal */}
      {showThinking && createPortal(
        <ThinkingModal 
          thinking={thinking || 'En attente de la réflexion du modèle...'}
          onClose={() => {
            console.log('Closing thinking modal');
            setShowThinking(false);
          }} 
        />,
        document.body
      )}

      {/* RAG Modal - rendered outside via portal */}
      {showRagModal && createPortal(
        <RagModal 
          onClose={() => setShowRagModal(false)}
        />,
        document.body
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

function ModelSelectionModal({ onClose, onModelChange }: { onClose: () => void; onModelChange?: (model: any) => void }) {
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedModel, setSelectedModel] = useState(modelConfigStore.currentModel.id);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    setIsLoading(true);
    setError('');

    try {
      const config = modelConfigStore.getConfig();
      
      if (!config.apiKey && !config.baseUrl) {
        setError('Configuration manquante : veuillez d\'abord configurer votre API');
        setAvailableModels([]);
        return;
      }

      const client = new LiteLLMClient(config);
      const result = await client.getModels();
      
      if (result.data && Array.isArray(result.data)) {
        setAvailableModels(result.data);
      } else {
        // Fallback vers les modèles par défaut
        setAvailableModels([]);
        setError('Impossible de récupérer les modèles du serveur');
      }
    } catch (err) {
      console.error('Error loading models:', err);
      setError(err instanceof Error ? err.message : 'Erreur de connexion');
      setAvailableModels([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleModelSelect = (modelId: string) => {
    setSelectedModel(modelId);
  };

  const handleSave = () => {
    // Chercher le modèle d'abord dans les modèles du serveur, puis dans notre liste locale
    let foundModel = availableModels.find(m => m.id === selectedModel) || 
                     AVAILABLE_MODELS.find(m => m.id === selectedModel) ||
                     modelConfigStore.currentModel;
    
    // Si le modèle du serveur n'a pas de 'name', utiliser l'id
    if (foundModel && !foundModel.name) {
      foundModel = {
        ...foundModel,
        name: foundModel.id
      };
    }
    
    console.log('Saving model:', foundModel);
    modelConfigStore.setModel(foundModel);
    
    // Notify parent component
    onModelChange?.(foundModel);
    
    onClose();
  };

  return (
    <>
      <div className="dropdown-overlay" onClick={onClose} />
      <div className="settings-modal-fixed">
        <div className="settings-header">
          <h3>Sélection du modèle LLM</h3>
          <button onClick={onClose} className="close-button">×</button>
        </div>
        
        <div className="settings-content">
          {isLoading ? (
            <div className="loading-state">
              <Loader2 size={20} className="animate-spin" />
              <span>Chargement des modèles...</span>
            </div>
          ) : error ? (
            <div className="error-state">
              <XCircle size={20} />
              <span>{error}</span>
              <button onClick={loadModels} className="retry-button">
                Réessayer
              </button>
            </div>
          ) : (
            <div className="models-list">
              {(availableModels.length > 0 ? availableModels : AVAILABLE_MODELS).map((model) => (
                <div
                  key={model.id}
                  className={`model-item ${selectedModel === model.id ? 'selected' : ''}`}
                  onClick={() => handleModelSelect(model.id)}
                >
                  <div className="model-info">
                    <div className="model-name">{model.id}</div>
                    {model.object && (
                      <div className="model-type">{model.object}</div>
                    )}
                  </div>
                  {selectedModel === model.id && (
                    <CheckCircle size={16} className="check-icon" />
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
        
        <div className="settings-footer">
          <button onClick={onClose} className="cancel-button">
            Annuler
          </button>
          <button 
            onClick={loadModels}
            disabled={isLoading}
            className="test-button"
          >
            <Zap size={14} />
            <span>Actualiser</span>
          </button>
          <button 
            onClick={handleSave} 
            className="save-button"
            disabled={!selectedModel}
          >
            Sélectionner
          </button>
        </div>
      </div>
    </>
  );
}

function ThinkingModal({ thinking, onClose }: { thinking: string; onClose: () => void }) {
  console.log('ThinkingModal rendering with thinking:', thinking.slice(0, 50));
  return (
    <>
      <div className="dropdown-overlay" onClick={onClose} />
      <div className="thinking-modal">
        <div className="thinking-modal-header">
          <div className="thinking-modal-title">
            <Brain size={16} />
            <h3>Processus de réflexion</h3>
          </div>
          <button onClick={onClose} className="close-button">×</button>
        </div>
        
        <div className="thinking-modal-content">
          <ReactMarkdown 
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeHighlight]}
          >
            {thinking}
          </ReactMarkdown>
        </div>
      </div>
    </>
  );
}

function RagModal({ onClose }: { onClose: () => void }) {
  const [selectedGroup, setSelectedGroup] = useState<string>('');
  const [chunkSize, setChunkSize] = useState(512);
  const [overlap, setOverlap] = useState(64);
  const [strategy, setStrategy] = useState('AST-First');
  const [tags, setTags] = useState('');
  const [priority, setPriority] = useState('Normal');
  const [language, setLanguage] = useState('Auto-detect');
  const [groups, setGroups] = useState<DocumentGroup[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Load groups on mount
  useEffect(() => {
    const loadGroups = async () => {
      setIsLoading(true);
      try {
        await RagStore.loadGroups();
      } catch (error) {
        console.error('Error loading groups:', error);
      }
      setIsLoading(false);
    };

    loadGroups();

    // Subscribe to groups changes
    const unsubscribe = RagStore.subscribe((updatedGroups) => {
      setGroups(updatedGroups);
    });

    return unsubscribe;
  }, []);

  const createNewGroup = async () => {
    const name = prompt('Nom du nouveau groupe:');
    if (name) {
      try {
        setIsLoading(true);
        await RagStore.createGroup(name, {
          chunk_size: chunkSize,
          overlap: overlap,
          strategy: strategy as any
        });
      } catch (error) {
        console.error('Error creating group:', error);
        alert('Erreur lors de la création du groupe');
      }
      setIsLoading(false);
    }
  };

  const toggleGroup = async (groupId: string) => {
    try {
      setIsLoading(true);
      await RagStore.toggleGroup(groupId);
    } catch (error) {
      console.error('Error toggling group:', error);
      alert('Erreur lors de la modification du groupe');
    }
    setIsLoading(false);
  };

  const deleteGroup = async (groupId: string) => {
    if (confirm('Êtes-vous sûr de vouloir supprimer ce groupe ?')) {
      try {
        setIsLoading(true);
        await RagStore.deleteGroup(groupId);
      } catch (error) {
        console.error('Error deleting group:', error);
        alert('Erreur lors de la suppression du groupe');
      }
      setIsLoading(false);
    }
  };

  return (
    <>
      <div className="dropdown-overlay" onClick={onClose} />
      <div className="rag-modal">
        <div className="rag-modal-header">
          <div className="rag-modal-title">
            <FileText size={20} />
            <h2>RAG - Gestion des Documents</h2>
          </div>
          <button onClick={onClose} className="close-button">×</button>
        </div>
        
        <div className="rag-modal-content">
          {/* Section Groupes */}
          <div className="rag-section">
            <h3>Groupes de Documents</h3>
            <div className="groups-list">
              {isLoading && groups.length === 0 ? (
                <div style={{ textAlign: 'center', padding: '20px', color: '#9ca3af' }}>
                  Chargement des groupes...
                </div>
              ) : groups.length === 0 ? (
                <div style={{ textAlign: 'center', padding: '20px', color: '#9ca3af' }}>
                  Aucun groupe créé. Créez votre premier groupe !
                </div>
              ) : (
                groups.map((group) => (
                  <div key={group.id} className={`group-item ${group.active ? 'active' : ''}`}>
                    <div className="group-info">
                      <span className="group-icon">📁</span>
                      <span className="group-name">{group.name}</span>
                      <span className={`group-status ${group.active ? 'active' : 'inactive'}`}>
                        {group.active ? '●' : '○'}
                      </span>
                      <span className="group-count">{group.documents?.length || 0} docs</span>
                    </div>
                    <div className="group-actions">
                      <button 
                        className="group-action-btn"
                        onClick={() => toggleGroup(group.id)}
                        title={group.active ? 'Désactiver' : 'Activer'}
                      >
                        {group.active ? 'ON' : 'OFF'}
                      </button>
                      <button 
                        className="group-action-btn edit"
                        title="Éditer"
                      >
                        Edit
                      </button>
                      <button 
                        className="group-action-btn delete"
                        onClick={() => deleteGroup(group.id)}
                        title="Supprimer"
                      >
                        Del
                      </button>
                    </div>
                  </div>
                ))
              )}
              
              <button className="new-group-btn" onClick={createNewGroup}>
                + Nouveau Groupe
              </button>
            </div>
          </div>

          {/* Section Upload & Configuration */}
          <div className="rag-section">
            <h3>Upload & Configuration</h3>
            
            {/* Zone d'upload */}
            <div className="upload-zone">
              <div className="upload-placeholder">
                📁 Glissez-déposez vos fichiers ici...
                <br />
                <small>Support: PDF, TXT, MD, JS, TS, PY</small>
              </div>
              <button className="browse-btn">Parcourir</button>
            </div>

            {/* Configuration */}
            <div className="upload-config">
              <div className="config-row">
                <label>Groupe cible:</label>
                <select value={selectedGroup} onChange={(e) => setSelectedGroup(e.target.value)}>
                  <option value="">Sélectionner un groupe</option>
                  {groups.map(group => (
                    <option key={group.id} value={group.id}>{group.name}</option>
                  ))}
                </select>
              </div>

              <div className="config-section">
                <h4>⚙️ Paramètres de Chunking</h4>
                <div className="config-row">
                  <label>Chunk Size: {chunkSize} tokens</label>
                  <input 
                    type="range" 
                    min="256" 
                    max="1024" 
                    value={chunkSize} 
                    onChange={(e) => setChunkSize(Number(e.target.value))}
                  />
                </div>
                <div className="config-row">
                  <label>Overlap: {overlap} tokens</label>
                  <input 
                    type="range" 
                    min="32" 
                    max="128" 
                    value={overlap} 
                    onChange={(e) => setOverlap(Number(e.target.value))}
                  />
                </div>
                <div className="config-row">
                  <label>Strategy:</label>
                  <select value={strategy} onChange={(e) => setStrategy(e.target.value)}>
                    <option value="AST-First">AST-First</option>
                    <option value="Heuristic">Heuristic</option>
                    <option value="Hybrid">Hybrid</option>
                  </select>
                </div>
              </div>

              <div className="config-section">
                <h4>🏷️ Métadonnées</h4>
                <div className="config-row">
                  <label>Tags:</label>
                  <input 
                    type="text" 
                    placeholder="frontend, react, components" 
                    value={tags}
                    onChange={(e) => setTags(e.target.value)}
                  />
                </div>
                <div className="config-row">
                  <label>Priority:</label>
                  <select value={priority} onChange={(e) => setPriority(e.target.value)}>
                    <option value="Low">Low</option>
                    <option value="Normal">Normal</option>
                    <option value="High">High</option>
                  </select>
                </div>
                <div className="config-row">
                  <label>Language:</label>
                  <select value={language} onChange={(e) => setLanguage(e.target.value)}>
                    <option value="Auto-detect">Auto-detect</option>
                    <option value="JavaScript">JavaScript</option>
                    <option value="TypeScript">TypeScript</option>
                    <option value="Python">Python</option>
                    <option value="Rust">Rust</option>
                  </select>
                </div>
              </div>

              <button className="index-btn" disabled={!selectedGroup}>
                Indexer Documents
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}