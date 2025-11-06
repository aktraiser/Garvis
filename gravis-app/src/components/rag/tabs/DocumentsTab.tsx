import React from 'react';
import { FileText, Trash2, Eye, Download, Edit3, Save, X } from 'lucide-react';
import type { DocumentInfo, NotificationState } from '../types';

interface DocumentsTabProps {
  documents: DocumentInfo[];
  isLoadingDocuments: boolean;
  extractedContent: Record<string, any>;
  showExtraction: string | null;
  editingContent: string;
  isEditing: boolean;
  notification: NotificationState | null;
  onDeleteDocument: (docId: string, docName: string) => void;
  onViewDocument: (filename: string) => void;
  onExtractDocument: (docName: string) => void;
  onShowExtraction: (filename: string) => void;
  onSaveExtraction: () => void;
  onCancelEdit: () => void;
  onSetShowExtraction: (filename: string | null) => void;
  onSetEditingContent: (content: string) => void;
  onSetIsEditing: (editing: boolean) => void;
}

export const DocumentsTab: React.FC<DocumentsTabProps> = ({
  documents,
  isLoadingDocuments,
  extractedContent,
  showExtraction,
  editingContent,
  isEditing,
  notification,
  onDeleteDocument,
  onViewDocument,
  onExtractDocument,
  onShowExtraction,
  onSaveExtraction,
  onCancelEdit,
  onSetShowExtraction,
  onSetEditingContent,
  onSetIsEditing
}) => {
  return (
    <div>
      {/* Loading indicator */}
      {isLoadingDocuments && (
        <div style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '200px',
          color: '#999'
        }}>
          Chargement des documents...
        </div>
      )}
      
      {/* Documents table */}
      {!isLoadingDocuments && (
        <div style={{
          background: 'rgba(255, 255, 255, 0.03)',
          borderRadius: '8px',
          overflow: 'hidden',
          border: '1px solid rgba(255, 255, 255, 0.1)'
        }}>
          {/* Table header */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: '2fr 1fr 1fr 1fr 1fr 2fr',
            gap: '16px',
            padding: '16px',
            background: 'rgba(255, 255, 255, 0.05)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            fontSize: '14px',
            fontWeight: '500',
            color: '#999'
          }}>
            <div>Document</div>
            <div>Taille</div>
            <div>Type</div>
            <div>Pages</div>
            <div>Statut</div>
            <div>Actions</div>
          </div>
          
          {/* Table body */}
          {documents.length === 0 ? (
            <div style={{
              padding: '48px',
              textAlign: 'center',
              color: '#666',
              fontSize: '16px'
            }}>
              Aucun document trouvé
            </div>
          ) : (
            documents.map((doc) => (
              <div key={doc.id} style={{
                display: 'grid',
                gridTemplateColumns: '2fr 1fr 1fr 1fr 1fr 2fr',
                gap: '16px',
                padding: '16px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                alignItems: 'center',
                transition: 'background 0.2s ease',
                cursor: 'pointer'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(255, 255, 255, 0.02)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'transparent';
              }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <FileText size={20} style={{ color: '#0066cc', flexShrink: 0 }} />
                  <div>
                    <div style={{ fontWeight: '500', marginBottom: '4px' }}>{doc.name}</div>
                    <div style={{ fontSize: '12px', color: '#999' }}>
                      {new Date(doc.date).toLocaleDateString()}
                    </div>
                  </div>
                </div>
                
                <div style={{ fontSize: '14px', color: '#ccc' }}>
                  {doc.size}
                </div>
                
                <div style={{
                  fontSize: '12px',
                  background: 'rgba(59, 130, 246, 0.1)',
                  color: '#60a5fa',
                  padding: '4px 8px',
                  borderRadius: '4px',
                  textAlign: 'center',
                  textTransform: 'uppercase',
                  fontWeight: '500'
                }}>
                  {doc.type}
                </div>
                
                <div style={{ fontSize: '14px', color: '#ccc', textAlign: 'center' }}>
                  {doc.pages}
                </div>
                
                <div style={{
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: '4px'
                }}>
                  <div style={{
                    fontSize: '12px',
                    background: doc.extracted ? 'rgba(34, 197, 94, 0.1)' : 'rgba(156, 163, 175, 0.1)',
                    color: doc.extracted ? '#22c55e' : '#9ca3af',
                    padding: '4px 8px',
                    borderRadius: '4px',
                    textAlign: 'center',
                    fontWeight: '500'
                  }}>
                    {doc.extracted ? 'Extrait' : 'Non extrait'}
                  </div>
                  {doc.extracted && doc.confidence > 0 && (
                    <div style={{
                      fontSize: '11px',
                      color: '#999',
                      marginLeft: '4px'
                    }}>
                      {Math.round(doc.confidence * 100)}%
                    </div>
                  )}
                </div>
                
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <div style={{
                    display: 'flex',
                    gap: '8px',
                    alignItems: 'center'
                  }}>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onViewDocument(doc.name);
                      }}
                      style={{
                        background: 'rgba(59, 130, 246, 0.1)',
                        border: '1px solid rgba(59, 130, 246, 0.3)',
                        color: '#60a5fa',
                        cursor: 'pointer',
                        padding: '6px',
                        borderRadius: '4px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                      }}
                      title="Voir le document"
                    >
                      <Eye size={14} />
                    </button>
                    
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onExtractDocument(doc.name);
                      }}
                      style={{
                        background: 'rgba(249, 115, 22, 0.1)',
                        border: '1px solid rgba(249, 115, 22, 0.3)',
                        color: '#fb923c',
                        cursor: 'pointer',
                        padding: '6px',
                        borderRadius: '4px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                      }}
                      title="Extraire le contenu"
                    >
                      <Download size={14} />
                    </button>
                    
                    {extractedContent[doc.name] && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onShowExtraction(doc.name);
                        }}
                        style={{
                          background: 'rgba(168, 85, 247, 0.1)',
                          border: '1px solid rgba(168, 85, 247, 0.3)',
                          color: '#a855f7',
                          cursor: 'pointer',
                          padding: '6px',
                          borderRadius: '4px',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center'
                        }}
                        title="Voir l'extraction"
                      >
                        <Edit3 size={14} />
                      </button>
                    )}
                    
                    
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onDeleteDocument(doc.id, doc.name);
                      }}
                      style={{
                        background: 'rgba(239, 68, 68, 0.1)',
                        border: '1px solid rgba(239, 68, 68, 0.3)',
                        color: '#ef4444',
                        cursor: 'pointer',
                        padding: '6px',
                        borderRadius: '4px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                      }}
                      title="Supprimer le document"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Extraction Modal */}
      {showExtraction && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.8)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 10000,
          padding: '20px'
        }}>
          <div style={{
            background: 'linear-gradient(135deg, #1e293b 0%, #334155 100%)',
            borderRadius: '12px',
            width: '90%',
            maxWidth: '800px',
            maxHeight: '80vh',
            overflow: 'hidden',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          }}>
            <div style={{
              padding: '20px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center'
            }}>
              <h3 style={{ margin: 0, fontSize: '18px', fontWeight: '600' }}>
                Extraction de {showExtraction}
              </h3>
              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                {isEditing ? (
                  <>
                    <button
                      onClick={onSaveExtraction}
                      style={{
                        background: 'rgba(34, 197, 94, 0.1)',
                        border: '1px solid rgba(34, 197, 94, 0.3)',
                        color: '#22c55e',
                        cursor: 'pointer',
                        padding: '8px 12px',
                        borderRadius: '6px',
                        fontSize: '14px',
                        fontWeight: '500',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px'
                      }}
                    >
                      <Save size={16} />
                      Sauvegarder
                    </button>
                    <button
                      onClick={onCancelEdit}
                      style={{
                        background: 'rgba(156, 163, 175, 0.1)',
                        border: '1px solid rgba(156, 163, 175, 0.3)',
                        color: '#9ca3af',
                        cursor: 'pointer',
                        padding: '8px 12px',
                        borderRadius: '6px',
                        fontSize: '14px'
                      }}
                    >
                      Annuler
                    </button>
                  </>
                ) : (
                  <button
                    onClick={() => onSetIsEditing(true)}
                    style={{
                      background: 'rgba(249, 115, 22, 0.1)',
                      border: '1px solid rgba(249, 115, 22, 0.3)',
                      color: '#fb923c',
                      cursor: 'pointer',
                      padding: '8px 12px',
                      borderRadius: '6px',
                      fontSize: '14px',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px'
                    }}
                  >
                    <Edit3 size={16} />
                    Éditer
                  </button>
                )}
                <button
                  onClick={() => onSetShowExtraction(null)}
                  style={{
                    background: 'rgba(156, 163, 175, 0.1)',
                    border: '1px solid rgba(156, 163, 175, 0.3)',
                    color: '#9ca3af',
                    cursor: 'pointer',
                    padding: '8px',
                    borderRadius: '6px'
                  }}
                >
                  <X size={16} />
                </button>
              </div>
            </div>
            <div style={{ padding: '20px', maxHeight: 'calc(80vh - 100px)', overflow: 'auto' }}>
              {isEditing ? (
                <textarea
                  value={editingContent}
                  onChange={(e) => onSetEditingContent(e.target.value)}
                  style={{
                    width: '100%',
                    height: '400px',
                    background: 'rgba(0, 0, 0, 0.3)',
                    border: '1px solid rgba(255, 255, 255, 0.2)',
                    borderRadius: '6px',
                    padding: '12px',
                    color: '#ffffff',
                    fontSize: '14px',
                    lineHeight: '1.5',
                    fontFamily: 'Monaco, Consolas, monospace',
                    resize: 'vertical'
                  }}
                />
              ) : (
                <div style={{
                  background: 'rgba(0, 0, 0, 0.3)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '6px',
                  padding: '16px',
                  fontSize: '14px',
                  lineHeight: '1.6',
                  whiteSpace: 'pre-wrap',
                  maxHeight: '400px',
                  overflow: 'auto'
                }}>
                  {extractedContent[showExtraction]?.content || 'Aucun contenu extrait'}
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Notification */}
      {notification && (
        <div style={{
          position: 'fixed',
          top: '20px',
          right: '20px',
          background: notification.type === 'success' ? 'rgba(34, 197, 94, 0.9)' : 
                     notification.type === 'error' ? 'rgba(239, 68, 68, 0.9)' : 
                     'rgba(59, 130, 246, 0.9)',
          color: '#ffffff',
          padding: '12px 16px',
          borderRadius: '8px',
          fontSize: '14px',
          fontWeight: '500',
          zIndex: 10001,
          maxWidth: '300px',
          border: `1px solid ${
            notification.type === 'success' ? 'rgba(34, 197, 94, 0.3)' : 
            notification.type === 'error' ? 'rgba(239, 68, 68, 0.3)' : 
            'rgba(59, 130, 246, 0.3)'
          }`
        }}>
          {notification.message}
        </div>
      )}
    </div>
  );
};