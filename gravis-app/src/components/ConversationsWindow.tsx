import React, { useState, useEffect } from 'react';
import { MessageSquare, Calendar, Trash2, Download, Copy, Play } from 'lucide-react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  updatedAt: Date;
  model: string;
  tags: string[];
}

interface ConversationsWindowProps {
  onClose?: () => void;
  onResumeConversation?: (conversation: Conversation) => void;
  onCopyMessage?: (content: string) => void;
}

export const ConversationsWindow: React.FC<ConversationsWindowProps> = ({ onClose, onResumeConversation, onCopyMessage }) => {
  console.log('🎯 ConversationsWindow props:', { onClose: !!onClose, onResumeConversation: !!onResumeConversation, onCopyMessage: !!onCopyMessage });
  
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  
  // Load conversations from localStorage on mount
  useEffect(() => {
    loadConversations();
  }, []);

  const loadConversations = () => {
    try {
      const saved = localStorage.getItem('gravis-conversations');
      if (saved) {
        const parsedConversations = JSON.parse(saved).map((conv: any) => ({
          ...conv,
          createdAt: new Date(conv.createdAt),
          updatedAt: new Date(conv.updatedAt),
          messages: conv.messages.map((msg: any) => ({
            ...msg,
            timestamp: new Date(msg.timestamp)
          }))
        }));
        setConversations(parsedConversations);
      }
    } catch (error) {
      console.error('Error loading conversations:', error);
    }
  };

  const deleteConversation = (conversationId: string) => {
    const updatedConversations = conversations.filter(conv => conv.id !== conversationId);
    setConversations(updatedConversations);
    saveConversations(updatedConversations);
    
    if (selectedConversation?.id === conversationId) {
      setSelectedConversation(null);
    }
  };

  const saveConversations = (convs: Conversation[]) => {
    try {
      localStorage.setItem('gravis-conversations', JSON.stringify(convs));
    } catch (error) {
      console.error('Error saving conversations:', error);
    }
  };

  const exportConversation = (conversation: Conversation) => {
    const exportData = {
      title: conversation.title,
      model: conversation.model,
      createdAt: conversation.createdAt,
      messages: conversation.messages
    };
    
    const dataStr = JSON.stringify(exportData, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);
    
    const exportFileDefaultName = `conversation-${conversation.title.replace(/\s+/g, '-')}.json`;
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };


  const filteredConversations = conversations;

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('fr-FR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  };

  const handleCopyMessage = async (content: string) => {
    try {
      await navigator.clipboard.writeText(content);
      if (onCopyMessage) {
        onCopyMessage(content);
      }
      // Ici on pourrait ajouter une notification de succès
    } catch (error) {
      console.error('Erreur lors de la copie:', error);
    }
  };

  const handleResumeConversation = async (conversation: Conversation) => {
    console.log('🔄 Démarrage reprise conversation:', conversation.title);
    
    if (onResumeConversation) {
      onResumeConversation(conversation);
      console.log('✅ Callback onResumeConversation appelé');
    }
  };

  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f0f23 100%)',
      color: '#ffffff',
      display: 'flex',
      flexDirection: 'column'
    }}>
      {/* Header */}
      <div style={{
        padding: '16px 24px',
        borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <MessageSquare size={24} style={{ color: '#3b82f6' }} />
          <h1 style={{ margin: 0, fontSize: '20px', fontWeight: '600' }}>
            Historique des Conversations
          </h1>
          <span style={{
            backgroundColor: 'rgba(59, 130, 246, 0.2)',
            color: '#60a5fa',
            padding: '4px 8px',
            borderRadius: '4px',
            fontSize: '12px',
            fontWeight: '500'
          }}>
            {conversations.length} conversations
          </span>
        </div>
      </div>

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Sidebar */}
        <div style={{
          width: '400px',
          borderRight: '1px solid rgba(255, 255, 255, 0.1)',
          display: 'flex',
          flexDirection: 'column'
        }}>

          {/* Conversations list */}
          <div style={{ flex: 1, overflow: 'auto' }}>
            {filteredConversations.length === 0 ? (
              <div style={{
                padding: '32px 16px',
                textAlign: 'center',
                color: '#9ca3af'
              }}>
                <MessageSquare size={48} style={{ marginBottom: '16px', opacity: 0.5 }} />
                <p style={{ margin: 0, fontSize: '16px' }}>Aucune conversation trouvée</p>
                <p style={{ margin: '8px 0 0 0', fontSize: '14px' }}>
                  Démarrez une conversation pour la voir apparaître ici
                </p>
              </div>
            ) : (
              filteredConversations.map((conversation) => (
                <div
                  key={conversation.id}
                  onClick={() => {
                    setSelectedConversation(conversation);
                    handleResumeConversation(conversation);
                  }}
                  style={{
                    padding: '12px 16px',
                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                    cursor: 'pointer',
                    background: selectedConversation?.id === conversation.id 
                      ? 'rgba(59, 130, 246, 0.1)' 
                      : 'transparent'
                  }}
                  onMouseEnter={(e) => {
                    if (selectedConversation?.id !== conversation.id) {
                      e.currentTarget.style.background = 'rgba(255, 255, 255, 0.05)';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (selectedConversation?.id !== conversation.id) {
                      e.currentTarget.style.background = 'transparent';
                    }
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '4px' }}>
                    <h3 style={{
                      margin: 0,
                      fontSize: '14px',
                      fontWeight: '500',
                      color: '#ffffff',
                      lineHeight: '1.2'
                    }}>
                      {conversation.title}
                    </h3>
                    <div style={{ display: 'flex', gap: '4px' }}>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          exportConversation(conversation);
                        }}
                        style={{
                          background: 'none',
                          border: 'none',
                          color: '#9ca3af',
                          cursor: 'pointer',
                          padding: '2px'
                        }}
                        title="Exporter"
                      >
                        <Download size={12} />
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteConversation(conversation.id);
                        }}
                        style={{
                          background: 'none',
                          border: 'none',
                          color: '#ef4444',
                          cursor: 'pointer',
                          padding: '2px'
                        }}
                        title="Supprimer"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <span style={{
                      fontSize: '11px',
                      color: '#9ca3af',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px'
                    }}>
                      <Calendar size={10} />
                      {formatDate(conversation.updatedAt)}
                    </span>
                    <span style={{
                      fontSize: '11px',
                      backgroundColor: 'rgba(255, 255, 255, 0.1)',
                      color: '#9ca3af',
                      padding: '2px 6px',
                      borderRadius: '3px'
                    }}>
                      {conversation.model}
                    </span>
                  </div>
                  <p style={{
                    margin: 0,
                    fontSize: '12px',
                    color: '#6b7280',
                    lineHeight: '1.3',
                    display: '-webkit-box',
                    WebkitLineClamp: 2,
                    WebkitBoxOrient: 'vertical',
                    overflow: 'hidden'
                  }}>
                    {conversation.messages[conversation.messages.length - 1]?.content || 'Conversation vide'}
                  </p>
                  {conversation.tags.length > 0 && (
                    <div style={{ marginTop: '6px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                      {conversation.tags.map(tag => (
                        <span key={tag} style={{
                          fontSize: '10px',
                          backgroundColor: 'rgba(34, 197, 94, 0.2)',
                          color: '#10b981',
                          padding: '1px 4px',
                          borderRadius: '2px'
                        }}>
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>

        {/* Main content */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          {selectedConversation ? (
            <>
              {/* Conversation header */}
              <div style={{
                padding: '16px 24px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
                background: 'rgba(255, 255, 255, 0.02)',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center'
              }}>
                <div>
                  <h2 style={{ margin: 0, fontSize: '18px', fontWeight: '600', marginBottom: '8px' }}>
                    {selectedConversation.title}
                  </h2>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '16px', fontSize: '14px', color: '#9ca3af' }}>
                    <span>Modèle: {selectedConversation.model}</span>
                    <span>Créé le: {formatDate(selectedConversation.createdAt)}</span>
                    <span>{selectedConversation.messages.length} messages</span>
                  </div>
                </div>
                
                {/* Actions */}
                <div style={{ display: 'flex', gap: '8px' }}>
                  <button
                    onClick={async () => {
                      console.log('🖱️ Bouton Reprendre cliqué!', selectedConversation?.title);
                      
                      // Reprendre la conversation
                      if (selectedConversation && onResumeConversation) {
                        onResumeConversation(selectedConversation);
                      }
                      
                      // Fermer la fenêtre immédiatement
                      try {
                        const { invoke } = await import('@tauri-apps/api/core');
                        console.log('🚪 Tentative de fermeture...');
                        await invoke('close_specific_window', { window_label: 'conversations' });
                        console.log('✅ Commande de fermeture envoyée');
                      } catch (error) {
                        console.error('❌ Erreur fermeture:', error);
                      }
                    }}
                    style={{
                      background: 'rgba(34, 197, 94, 0.1)',
                      border: '1px solid rgba(34, 197, 94, 0.3)',
                      color: '#22c55e',
                      padding: '8px 12px',
                      borderRadius: '6px',
                      fontSize: '14px',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px'
                    }}
                    title="Reprendre cette conversation"
                  >
                    <Play size={16} />
                    Reprendre
                  </button>
                  
                  <button
                    onClick={() => {
                      const fullConversation = selectedConversation.messages
                        .map(msg => `${msg.role === 'user' ? 'Vous' : 'Assistant'}: ${msg.content}`)
                        .join('\n\n');
                      handleCopyMessage(fullConversation);
                    }}
                    style={{
                      background: 'rgba(59, 130, 246, 0.1)',
                      border: '1px solid rgba(59, 130, 246, 0.3)',
                      color: '#3b82f6',
                      padding: '8px 12px',
                      borderRadius: '6px',
                      fontSize: '14px',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px'
                    }}
                    title="Copier toute la conversation"
                  >
                    <Copy size={16} />
                    Copier tout
                  </button>
                </div>
              </div>

              {/* Messages */}
              <div style={{
                flex: 1,
                overflow: 'auto',
                padding: '24px'
              }}>
                {selectedConversation.messages.map((message) => (
                  <div
                    key={message.id}
                    style={{
                      marginBottom: '24px',
                      display: 'flex',
                      flexDirection: message.role === 'user' ? 'row-reverse' : 'row',
                      alignItems: 'flex-start',
                      gap: '12px'
                    }}
                  >
                    <div style={{
                      width: '32px',
                      height: '32px',
                      borderRadius: '50%',
                      background: message.role === 'user' 
                        ? 'linear-gradient(135deg, #3b82f6, #1d4ed8)'
                        : 'linear-gradient(135deg, #10b981, #059669)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      fontSize: '14px',
                      fontWeight: '600',
                      color: '#ffffff',
                      flexShrink: 0
                    }}>
                      {message.role === 'user' ? 'U' : 'A'}
                    </div>
                    <div style={{
                      background: message.role === 'user' 
                        ? 'rgba(59, 130, 246, 0.1)'
                        : 'rgba(255, 255, 255, 0.05)',
                      padding: '12px 16px',
                      borderRadius: '12px',
                      maxWidth: '70%',
                      border: `1px solid ${message.role === 'user' 
                        ? 'rgba(59, 130, 246, 0.2)'
                        : 'rgba(255, 255, 255, 0.1)'}`
                    }}>
                      <div style={{
                        fontSize: '14px',
                        lineHeight: '1.5',
                        whiteSpace: 'pre-wrap',
                        wordWrap: 'break-word'
                      }}>
                        {message.content}
                      </div>
                      <div style={{
                        display: 'flex',
                        justifyContent: 'flex-start',
                        alignItems: 'center',
                        marginTop: '8px'
                      }}>
                        <button
                          onClick={() => handleCopyMessage(message.content)}
                          style={{
                            background: 'rgba(255, 255, 255, 0.1)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            color: '#9ca3af',
                            padding: '6px',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            opacity: 0.7
                          }}
                          title="Copier ce message"
                          onMouseEnter={(e) => {
                            e.currentTarget.style.opacity = '1';
                            e.currentTarget.style.background = 'rgba(255, 255, 255, 0.15)';
                          }}
                          onMouseLeave={(e) => {
                            e.currentTarget.style.opacity = '0.7';
                            e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)';
                          }}
                        >
                          <Copy size={12} />
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexDirection: 'column',
              color: '#9ca3af',
              textAlign: 'center'
            }}>
              <MessageSquare size={64} style={{ marginBottom: '16px', opacity: 0.5 }} />
              <h3 style={{ margin: '0 0 8px 0', fontSize: '18px' }}>Sélectionnez une conversation</h3>
              <p style={{ margin: 0, fontSize: '14px' }}>
                Choisissez une conversation dans la liste pour voir son contenu
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};