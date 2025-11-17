// DirectChatPage - Page dédiée pour le chat direct avec OCR viewer
// PR #4 Phase 2 - Interface Avancée avec Split Panel

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SimplePdfViewer from '@/components/SimplePdfViewer';
import './DirectChatPage.css';

// Types simplifiés
interface ProcessDocumentResponse {
  session: {
    session_id: string;
    document_name: string;
    chunks: any[];
  };
  processing_time_ms: number;
  chunks_created: number;
  embedded_chunks: number;
  confidence_score: number;
}

interface ChatResponse {
  response: string;
  confidence_score: number;
  session_id: string;
  search_time_ms: number;
  chunks_used: number;
  sources_summary: SourceSummary[];
}

interface SourceSummary {
  chunk_id: string;
  content_preview: string;
  score: number;
  confidence: number;
  span_count: number;
}

interface Message {
  id: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

interface SelectionContext {
  text?: string;
  bounding_rect?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

export const DirectChatPage: React.FC = () => {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [documentName, setDocumentName] = useState<string>('');
  const [messages, setMessages] = useState<Message[]>([]);
  const [query, setQuery] = useState<string>('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [selectionContext, setSelectionContext] = useState<SelectionContext | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Drag & Drop handlers
  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = e.dataTransfer.files;
    if (files && files.length > 0) {
      const file = files[0];
      await processDroppedDocument(file);
    }
  };

  // Process document
  const processDroppedDocument = async (file: File) => {
    setIsProcessing(true);

    try {
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      const response = await invoke<ProcessDocumentResponse>('process_dropped_document', {
        filePath: file.name,
        fileData: Array.from(uint8Array),
        mimeType: file.type || 'application/pdf'
      });

      if (response.session) {
        setSessionId(response.session.session_id);
        setDocumentName(response.session.document_name);
        setMessages([]);
      }
    } catch (error) {
      console.error('Erreur traitement:', error);
      const errorMessage: Message = {
        id: Date.now().toString(),
        type: 'system',
        content: `❌ Erreur: ${error}`,
        timestamp: new Date(),
      };
      setMessages([errorMessage]);
    } finally {
      setIsProcessing(false);
    }
  };

  // Send message
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!query.trim() || !sessionId || isProcessing) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      type: 'user',
      content: query.trim(),
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setQuery('');
    setIsProcessing(true);

    try {
      const response = await invoke<ChatResponse>('chat_with_dropped_document', {
        request: {
          session_id: sessionId,
          query: userMessage.content,
          selection: selectionContext,
          limit: null,
        }
      });

      // Plus de highlighted spans - on utilise juste la réponse

      // Format response
      let responseContent = response.response;

      if (response.sources_summary && response.sources_summary.length > 0) {
        responseContent += "\n\n**📚 Sources :**\n";
        response.sources_summary.forEach((source, idx) => {
          const score = Math.round(source.score * 100);
          const confidence = Math.round(source.confidence * 100);
          responseContent += `\n${idx + 1}. **[${score}%]** ${source.content_preview.substring(0, 100)}... *(confiance: ${confidence}%, ${source.span_count} spans)*`;
        });
        responseContent += `\n\n*Confiance: ${Math.round(response.confidence_score * 100)}% • Recherche: ${response.search_time_ms}ms*`;
      }

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: responseContent,
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, assistantMessage]);

      // Reset selection context after using it
      setSelectionContext(null);

    } catch (error) {
      console.error('Erreur chat:', error);
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        type: 'system',
        content: `❌ Erreur: ${error}`,
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsProcessing(false);
    }
  };

  // Handle text selection dans SimplePdfViewer
  const handleTextAction = React.useCallback(async (action: 'explain' | 'summarize', text: string) => {
    if (!sessionId) return;
    
    try {
      console.log(`🎯 ${action} requested for text:`, text);
      
      // Construire la question selon l'action
      const question = action === 'explain' 
        ? `Explique ce passage : "${text}"` 
        : `Résume ce passage : "${text}"`;
      
      // Envoyer directement au système RAG
      setQuery(question);
      
      // Auto-submit la question
      const userMessage: Message = {
        id: Date.now().toString(),
        type: 'user',
        content: question,
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, userMessage]);
      setIsProcessing(true);

      const response = await invoke<ChatResponse>('chat_with_dropped_document', {
        request: {
          session_id: sessionId,
          query: question,
          selection: { text },
          limit: null,
        }
      });

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: response.response,
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, assistantMessage]);
      setQuery(''); // Clear input

    } catch (error) {
      console.error('Error in text action:', error);
      const errorMessage: Message = {
        id: (Date.now() + 2).toString(),
        type: 'system',
        content: `❌ Erreur: ${error}`,
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsProcessing(false);
    }
  }, [sessionId]);

  // Ref stable pour éviter les re-renders du SimplePdfViewer
  const handleTextActionRef = React.useRef(handleTextAction);
  handleTextActionRef.current = handleTextAction;

  const stableHandleTextAction = React.useCallback((action: 'explain' | 'summarize', text: string) => {
    return handleTextActionRef.current(action, text);
  }, []);

  return (
    <div
      className="direct-chat-page"
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      {isDragging && (
        <div className="drag-overlay">
          <div className="drag-overlay-content">
            <div className="drag-overlay-icon">📄</div>
            <div className="drag-overlay-text">Déposez votre document ici</div>
          </div>
        </div>
      )}

      {!sessionId ? (
        /* Empty state - no document loaded */
        <div className="empty-state">
          <div className="empty-state-icon">📄</div>
          <div className="empty-state-title">Chat Direct avec Documents</div>
          <div className="empty-state-description">
            Glissez-déposez un document PDF ou image pour commencer
          </div>
          <div className="empty-state-features">
            <div className="feature-item">✓ Analyse OCR automatique</div>
            <div className="feature-item">✓ Chat contextuel intelligent</div>
            <div className="feature-item">✓ Highlighting des sources</div>
          </div>
        </div>
      ) : (
        /* Split panel - chat + OCR viewer */
        <div className="split-panel">
          {/* Left panel - Chat */}
          <div className="chat-panel">
            <div className="chat-header">
              <div className="chat-title">💬 Chat Direct</div>
              <div className="chat-document-name">{documentName}</div>
            </div>

            <div className="chat-messages">
              {messages.map((message) => (
                <div key={message.id} className={`message message-${message.type}`}>
                  <div className="message-content">
                    {message.content.split('\n').map((line, idx) => (
                      <div key={idx}>{line || '\u00A0'}</div>
                    ))}
                  </div>
                </div>
              ))}
            </div>

            <form className="chat-input-container" onSubmit={handleSubmit}>
              {selectionContext && (
                <div className="selection-context-badge">
                  📌 Sélection: "{selectionContext.text?.substring(0, 50)}..."
                  <button
                    type="button"
                    onClick={() => setSelectionContext(null)}
                    className="selection-context-clear"
                  >
                    ×
                  </button>
                </div>
              )}
              <textarea
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={selectionContext ? "Poser une question sur la sélection..." : "Poser une question sur le document..."}
                className="chat-input"
                disabled={isProcessing}
                rows={2}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    handleSubmit(e);
                  }
                }}
              />
              <button
                type="submit"
                className="chat-submit"
                disabled={isProcessing || !query.trim()}
              >
                {isProcessing ? '...' : '➤'}
              </button>
            </form>
          </div>

          {/* Right panel - PDF Viewer */}
          {sessionId && (
            <div className="pdf-viewer-panel">
              <SimplePdfViewer
                key={sessionId} 
                sessionId={sessionId}
                onTextAction={stableHandleTextAction}
              />
            </div>
          )}
        </div>
      )}
    </div>
  );
};
