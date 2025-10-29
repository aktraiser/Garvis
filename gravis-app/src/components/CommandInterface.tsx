import { useState, useEffect } from "react";
import { createPortal } from "react-dom";
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { RagWindow } from './RagWindow';
import { ModelSelectorWindow } from './ModelSelectorWindow';
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
  Copy,
  Volume2,
  ThumbsUp,
  ThumbsDown,
  RotateCcw
} from "lucide-react";
import { LiteLLMClient, modelConfigStore } from "@/lib/litellm";
import { tauriModelStore } from "@/lib/tauri-model-store";

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
  const [showRagWindow, setShowRagWindow] = useState(false);
  
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
    
    // Storage events (fallback pour compatibilité)
    const updateModelFromStorage = (event: any) => {
      if (event?.key === 'gravis-config') {
        const newModel = modelConfigStore.currentModel;
        setCurrentModel(newModel);
      }
    };
    
    window.addEventListener('storage', updateModelFromStorage);
    console.log('📦 Storage fallback listener added');
    
    // Polling de sauvegarde (dernier recours)
    const pollInterval = setInterval(() => {
      const storeModel = modelConfigStore.currentModel;
      if (storeModel.id !== currentModel.id) {
        console.log('🔄 Model change detected via polling backup');
        console.log('Store model:', storeModel);
        console.log('Current model:', currentModel);
        setCurrentModel(storeModel);
      }
    }, 2000); // Moins fréquent car les événements Tauri sont prioritaires
    console.log('🔄 Polling backup started');
    
    
    // Cleanup
    return () => {
      console.log('🧹 Cleaning up model listeners in main window');
      unsubscribeTauri();
      window.removeEventListener('storage', updateModelFromStorage);
      clearInterval(pollInterval);
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
  const addAssistantResponse = (content: string, thinkingContent?: string, metrics?: any) => {
    const assistantMessage = {
      id: (Date.now() + 1).toString(),
      type: 'assistant' as const,
      content,
      thinking: thinkingContent,
      timestamp: new Date(),
      metrics
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
    
    // Performance tracking variables
    const startTime = Date.now();
    let firstTokenTime: number | null = null;
    let totalTokens = 0;
    let inputTokens = 0;
    let outputTokens = 0;

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
                        // Mark first token time
                        if (!firstTokenTime && delta.content.trim()) {
                          firstTokenTime = Date.now();
                        }
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

        // Add streaming response to conversation history
        if (fullResponse) {
          addAssistantResponse(fullResponse, fullThinking, metrics);
        }
      } else {
        // Non-thinking models use regular chat
        const result = await client.chat(messages);
        
        console.log('API Response:', result);
        
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
          if (choice.message.reasoning) {
            console.log('Found reasoning:', choice.message.reasoning);
            setThinking(choice.message.reasoning);
            setShowThinking(true);
          }
          
          setResponse(choice.message.content);
          
          // Add assistant response to conversation history
          addAssistantResponse(choice.message.content, choice.message.reasoning, metrics);
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

  // Éviter l'erreur TypeScript - utiliser response
  if (response.length > 10000) console.log('Long response detected');
  
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
            onClick={openRagWindow}
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
                        {message.metrics.stopReason && (
                          <span className="metric metric-stop">
                            <span className="metric-label">Stop reason:</span> <strong>{message.metrics.stopReason}</strong>
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
              <h2>🧠 Réflexion du Modèle</h2>
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

