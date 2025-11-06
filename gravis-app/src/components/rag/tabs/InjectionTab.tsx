import React from 'react';
import { Search, Database, Trash2, X } from 'lucide-react';
import type { InjectionMetadata, NotificationState } from '../types';

interface InjectionTabProps {
  ragQuery: string;
  ragResults: any[];
  isSearching: boolean;
  ragDocuments: any[];
  isLoadingRagDocs: boolean;
  showInjectionModal: string | null;
  injectionMetadata: InjectionMetadata;
  isProcessing: Record<string, boolean>;
  notification: NotificationState | null;
  extractedContent: Record<string, any>;
  onSetRagQuery: (query: string) => void;
  onRagSearch: () => void;
  onDeleteRagDocument: (documentId: string) => void;
  onSetShowInjectionModal: (docName: string | null) => void;
  onSetInjectionMetadata: (metadata: InjectionMetadata) => void;
  onInjectDocument: (docName: string) => void;
  onPrepareInjection: (docName: string) => void;
}

export const InjectionTab: React.FC<InjectionTabProps> = ({
  ragQuery,
  ragResults,
  isSearching,
  ragDocuments,
  isLoadingRagDocs,
  showInjectionModal,
  injectionMetadata,
  isProcessing,
  notification,
  extractedContent,
  onSetRagQuery,
  onRagSearch,
  onDeleteRagDocument,
  onSetShowInjectionModal,
  onSetInjectionMetadata,
  onInjectDocument,
  onPrepareInjection
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Documents extraits prêts pour injection */}
      <div style={{
        background: 'rgba(255, 255, 255, 0.03)',
        borderRadius: '8px',
        padding: '20px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ 
          margin: '0 0 16px 0', 
          fontSize: '16px', 
          fontWeight: '600',
          display: 'flex',
          alignItems: 'center',
          gap: '8px'
        }}>
          <Database size={18} />
          Documents extraits ({Object.keys(extractedContent).length})
        </h3>

        {Object.keys(extractedContent).length === 0 ? (
          <div style={{
            textAlign: 'center',
            color: '#666',
            fontSize: '14px',
            padding: '40px'
          }}>
            Aucun document extrait. Allez dans l'onglet Documents pour extraire du contenu.
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {Object.entries(extractedContent).map(([docName, extraction]) => (
              <div key={docName} style={{
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '6px',
                padding: '16px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center'
              }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <div style={{ fontWeight: '500', fontSize: '14px' }}>
                      {docName}
                    </div>
                    <div style={{ 
                      fontSize: '11px', 
                      background: 'rgba(34, 197, 94, 0.1)',
                      color: '#22c55e',
                      padding: '2px 6px',
                      borderRadius: '3px'
                    }}>
                      Extrait
                    </div>
                  </div>
                  <div style={{ fontSize: '12px', color: '#999', marginLeft: '0px' }}>
                    {extraction.content && (
                      <div>Contenu: {extraction.content.substring(0, 100)}...</div>
                    )}
                    <div style={{ display: 'flex', gap: '16px', fontSize: '11px', color: '#666', marginTop: '4px' }}>
                      <span>Taille: {extraction.content?.length || 0} caractères</span>
                      {extraction.confidence && <span>Confiance: {Math.round(extraction.confidence * 100)}%</span>}
                      {extraction.language && <span>Langue: {extraction.language}</span>}
                    </div>
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                  <button
                    onClick={() => onPrepareInjection(docName)}
                    style={{
                      background: 'rgba(34, 197, 94, 0.1)',
                      border: '1px solid rgba(34, 197, 94, 0.3)',
                      color: '#22c55e',
                      cursor: 'pointer',
                      padding: '8px 16px',
                      borderRadius: '6px',
                      fontSize: '14px',
                      fontWeight: '500',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px'
                    }}
                    title="Configurer et injecter dans le RAG"
                  >
                    <Database size={14} />
                    Injecter
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Recherche RAG */}
      <div style={{
        background: 'rgba(255, 255, 255, 0.03)',
        borderRadius: '8px',
        padding: '20px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ 
          margin: '0 0 16px 0', 
          fontSize: '16px', 
          fontWeight: '600',
          display: 'flex',
          alignItems: 'center',
          gap: '8px'
        }}>
          <Search size={18} />
          Recherche dans le RAG
        </h3>
        
        <div style={{ display: 'flex', gap: '12px', marginBottom: '16px' }}>
          <input
            type="text"
            value={ragQuery}
            onChange={(e) => onSetRagQuery(e.target.value)}
            placeholder="Rechercher dans les documents RAG..."
            style={{
              flex: 1,
              padding: '12px',
              background: 'rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.2)',
              borderRadius: '6px',
              color: '#ffffff',
              fontSize: '14px'
            }}
            onKeyPress={(e) => {
              if (e.key === 'Enter') {
                onRagSearch();
              }
            }}
          />
          <button
            onClick={onRagSearch}
            disabled={isSearching || !ragQuery.trim()}
            style={{
              background: isSearching ? '#666' : '#0066cc',
              border: 'none',
              color: '#ffffff',
              cursor: isSearching || !ragQuery.trim() ? 'not-allowed' : 'pointer',
              padding: '12px 20px',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: '500',
              opacity: isSearching || !ragQuery.trim() ? 0.7 : 1,
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}
          >
            <Search size={16} />
            {isSearching ? 'Recherche...' : 'Rechercher'}
          </button>
        </div>

        {/* Résultats de recherche */}
        {ragResults.length > 0 && (
          <div style={{ maxHeight: '300px', overflowY: 'auto' }}>
            <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: '#999' }}>
              {ragResults.length} résultat(s) trouvé(s)
            </h4>
            {ragResults.map((result, index) => (
              <div key={index} style={{
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '6px',
                padding: '12px',
                marginBottom: '8px'
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                  <div style={{ fontWeight: '500', fontSize: '14px' }}>
                    {result.business_metadata?.title || result.document_id}
                  </div>
                  <div style={{ fontSize: '12px', color: '#999' }}>
                    Score: {(result.score || 0).toFixed(3)}
                  </div>
                </div>
                <div style={{ fontSize: '13px', color: '#ccc', lineHeight: '1.4' }}>
                  {(result.content || '').substring(0, 200)}
                  {(result.content || '').length > 200 && '...'}
                </div>
                <div style={{ marginTop: '8px', fontSize: '11px', color: '#666' }}>
                  ID: {result.document_id} | Chunk: {result.chunk_index || 0}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Documents RAG persistés */}
      <div style={{
        background: 'rgba(255, 255, 255, 0.03)',
        borderRadius: '8px',
        padding: '20px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ 
          margin: '0 0 16px 0', 
          fontSize: '16px', 
          fontWeight: '600',
          display: 'flex',
          alignItems: 'center',
          gap: '8px'
        }}>
          <Database size={18} />
          Documents dans le RAG ({ragDocuments.length})
        </h3>

        {isLoadingRagDocs ? (
          <div style={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            height: '100px',
            color: '#999'
          }}>
            Chargement des documents RAG...
          </div>
        ) : ragDocuments.length === 0 ? (
          <div style={{
            textAlign: 'center',
            color: '#666',
            fontSize: '14px',
            padding: '40px'
          }}>
            Aucun document dans le RAG
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {ragDocuments.map((doc, index) => (
              <div key={index} style={{
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '6px',
                padding: '16px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center'
              }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <Database size={16} style={{ color: '#22c55e' }} />
                    <div style={{ fontWeight: '500', fontSize: '14px' }}>
                      {doc.business_metadata?.title || doc.document_id}
                    </div>
                  </div>
                  <div style={{ fontSize: '12px', color: '#999', marginLeft: '24px' }}>
                    {doc.business_metadata?.description && (
                      <div>{doc.business_metadata.description}</div>
                    )}
                    <div style={{ display: 'flex', gap: '16px', fontSize: '11px', color: '#666', marginTop: '4px' }}>
                      <span>ID: {doc.document_id}</span>
                      <span>Chunks: {doc.chunk_count || 0}</span>
                      <span>Catégorie: {doc.business_metadata?.category || 'N/A'}</span>
                      {doc.business_metadata?.author && <span>Auteur: {doc.business_metadata.author}</span>}
                    </div>
                    {doc.business_metadata?.tags && doc.business_metadata.tags.length > 0 && (
                      <div style={{ marginTop: '4px' }}>
                        {doc.business_metadata.tags.map((tag: string, tagIndex: number) => (
                          <span key={tagIndex} style={{
                            background: 'rgba(59, 130, 246, 0.1)',
                            color: '#60a5fa',
                            padding: '2px 6px',
                            borderRadius: '3px',
                            fontSize: '10px',
                            marginRight: '4px'
                          }}>
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                  <button
                    onClick={() => onDeleteRagDocument(doc.document_id)}
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
                    title="Supprimer du RAG"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Modal d'injection avec métadonnées */}
      {showInjectionModal && (
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
            maxWidth: '600px',
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
                Configuration d'injection RAG
              </h3>
              <button
                onClick={() => onSetShowInjectionModal(null)}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: '#9ca3af',
                  cursor: 'pointer',
                  padding: '4px'
                }}
              >
                <X size={20} />
              </button>
            </div>
            
            <div style={{ padding: '20px', maxHeight: 'calc(80vh - 140px)', overflow: 'auto' }}>
              <div style={{ marginBottom: '16px', color: '#999', fontSize: '14px' }}>
                <strong>Document:</strong> {showInjectionModal}
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                {/* Informations de base */}
                <div>
                  <h4 style={{ margin: '0 0 12px 0', fontSize: '16px', fontWeight: '600', color: '#ffffff' }}>
                    📄 Informations du document
                  </h4>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div>
                      <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                        Titre du document
                      </label>
                      <input
                        type="text"
                        value={injectionMetadata.title}
                        onChange={(e) => onSetInjectionMetadata({...injectionMetadata, title: e.target.value})}
                        style={{
                          width: '100%',
                          padding: '8px 12px',
                          background: 'rgba(0, 0, 0, 0.3)',
                          border: '1px solid rgba(255, 255, 255, 0.2)',
                          borderRadius: '6px',
                          color: '#ffffff',
                          fontSize: '14px'
                        }}
                        placeholder="Titre descriptif du document"
                      />
                    </div>
                    
                    <div>
                      <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                        Description
                      </label>
                      <textarea
                        value={injectionMetadata.description}
                        onChange={(e) => onSetInjectionMetadata({...injectionMetadata, description: e.target.value})}
                        style={{
                          width: '100%',
                          padding: '8px 12px',
                          background: 'rgba(0, 0, 0, 0.3)',
                          border: '1px solid rgba(255, 255, 255, 0.2)',
                          borderRadius: '6px',
                          color: '#ffffff',
                          fontSize: '14px',
                          minHeight: '60px',
                          resize: 'vertical'
                        }}
                        placeholder="Description du contenu et du contexte"
                      />
                    </div>

                    <div style={{ display: 'flex', gap: '12px' }}>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Auteur
                        </label>
                        <input
                          type="text"
                          value={injectionMetadata.author}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, author: e.target.value})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                          placeholder="Nom de l'auteur"
                        />
                      </div>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Catégorie
                        </label>
                        <select
                          value={injectionMetadata.category}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, category: e.target.value})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                        >
                          <option value="document">Document</option>
                          <option value="facture">Facture</option>
                          <option value="contrat">Contrat</option>
                          <option value="rapport">Rapport</option>
                          <option value="manuel">Manuel</option>
                          <option value="email">Email</option>
                          <option value="note">Note</option>
                        </select>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Configuration RAG */}
                <div>
                  <h4 style={{ margin: '0 0 12px 0', fontSize: '16px', fontWeight: '600', color: '#ffffff' }}>
                    🔧 Configuration RAG
                  </h4>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div style={{ display: 'flex', gap: '12px' }}>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Groupe RAG
                        </label>
                        <input
                          type="text"
                          value={injectionMetadata.groupId}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, groupId: e.target.value})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                          placeholder="default_group"
                        />
                      </div>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Priorité
                        </label>
                        <select
                          value={injectionMetadata.priority}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, priority: e.target.value})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                        >
                          <option value="low">Basse</option>
                          <option value="normal">Normale</option>
                          <option value="high">Haute</option>
                          <option value="critical">Critique</option>
                        </select>
                      </div>
                    </div>

                    <div>
                      <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                        Tags (séparés par des virgules)
                      </label>
                      <input
                        type="text"
                        value={injectionMetadata.tags}
                        onChange={(e) => onSetInjectionMetadata({...injectionMetadata, tags: e.target.value})}
                        style={{
                          width: '100%',
                          padding: '8px 12px',
                          background: 'rgba(0, 0, 0, 0.3)',
                          border: '1px solid rgba(255, 255, 255, 0.2)',
                          borderRadius: '6px',
                          color: '#ffffff',
                          fontSize: '14px'
                        }}
                        placeholder="tag1, tag2, tag3"
                      />
                    </div>

                    <div style={{ display: 'flex', gap: '12px' }}>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Langue
                        </label>
                        <select
                          value={injectionMetadata.language}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, language: e.target.value})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                        >
                          <option value="auto">Auto-détection</option>
                          <option value="fr">Français</option>
                          <option value="en">Anglais</option>
                          <option value="es">Espagnol</option>
                          <option value="de">Allemand</option>
                          <option value="it">Italien</option>
                        </select>
                      </div>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Taille des chunks
                        </label>
                        <input
                          type="number"
                          value={injectionMetadata.chunkSize}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, chunkSize: parseInt(e.target.value) || 512})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                          min="64"
                          max="2048"
                        />
                      </div>
                      <div style={{ flex: 1 }}>
                        <label style={{ display: 'block', marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
                          Chevauchement
                        </label>
                        <input
                          type="number"
                          value={injectionMetadata.chunkOverlap}
                          onChange={(e) => onSetInjectionMetadata({...injectionMetadata, chunkOverlap: parseInt(e.target.value) || 50})}
                          style={{
                            width: '100%',
                            padding: '8px 12px',
                            background: 'rgba(0, 0, 0, 0.3)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            borderRadius: '6px',
                            color: '#ffffff',
                            fontSize: '14px'
                          }}
                          min="0"
                          max="200"
                        />
                      </div>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '12px', background: 'rgba(255, 255, 255, 0.03)', borderRadius: '8px' }}>
                      <input
                        type="checkbox"
                        id="forceOcr"
                        checked={injectionMetadata.forceOcr}
                        onChange={(e) => onSetInjectionMetadata({...injectionMetadata, forceOcr: e.target.checked})}
                        style={{
                          width: '16px',
                          height: '16px',
                          accentColor: '#22c55e'
                        }}
                      />
                      <label htmlFor="forceOcr" style={{ fontSize: '14px', color: '#ccc', cursor: 'pointer' }}>
                        Forcer l'OCR (même si le document contient déjà du texte)
                      </label>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div style={{
              padding: '20px',
              borderTop: '1px solid rgba(255, 255, 255, 0.1)',
              display: 'flex',
              justifyContent: 'flex-end',
              gap: '12px'
            }}>
              <button
                onClick={() => onSetShowInjectionModal(null)}
                style={{
                  background: 'rgba(156, 163, 175, 0.1)',
                  border: '1px solid rgba(156, 163, 175, 0.3)',
                  color: '#9ca3af',
                  cursor: 'pointer',
                  padding: '8px 16px',
                  borderRadius: '6px',
                  fontSize: '14px'
                }}
              >
                Annuler
              </button>
              <button
                onClick={() => showInjectionModal && onInjectDocument(showInjectionModal)}
                disabled={isProcessing[showInjectionModal || '']}
                style={{
                  background: 'rgba(34, 197, 94, 0.1)',
                  border: '1px solid rgba(34, 197, 94, 0.3)',
                  color: isProcessing[showInjectionModal || ''] ? '#999' : '#22c55e',
                  cursor: isProcessing[showInjectionModal || ''] ? 'not-allowed' : 'pointer',
                  padding: '8px 16px',
                  borderRadius: '6px',
                  fontSize: '14px',
                  fontWeight: '500'
                }}
              >
                {isProcessing[showInjectionModal || ''] ? 'Injection...' : 'Injecter dans le RAG'}
              </button>
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