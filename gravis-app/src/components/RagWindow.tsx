import React, { useState, useEffect } from 'react';
import { FileText, Database } from 'lucide-react';
import { DocumentsTab } from './rag/tabs/DocumentsTab';
import { InjectionTab } from './rag/tabs/InjectionTab';
import { useDocuments } from '../hooks/useDocuments';
import { useRagLogic } from '../hooks/useRagLogic';
import type { TabType } from './rag/types';

interface RagWindowProps {
  onClose?: () => void;
}

export const RagWindow: React.FC<RagWindowProps> = () => {
  console.log('🎯 RagWindow component mounting...');

  const [activeTab, setActiveTab] = useState<TabType>('documents');
  
  // Hooks pour la logique
  const {
    documents,
    isLoadingDocuments,
    notification,
    extractedContent,
    showExtraction,
    editingContent,
    isEditing,
    handleDeleteDocument,
    handleViewDocument,
    handleExtractDocument,
    handleShowExtraction,
    handleSaveExtraction,
    handleCancelEdit,
    showNotification,
    setShowExtraction,
    setEditingContent,
    setIsEditing
  } = useDocuments();
  
  const {
    isProcessing,
    ragQuery,
    ragResults,
    isSearching,
    ragDocuments,
    isLoadingRagDocs,
    showInjectionModal,
    injectionMetadata,
    setRagQuery,
    setShowInjectionModal,
    setInjectionMetadata,
    setChunkProfile,
    prepareInjectionMetadata,
    handleInjectDocumentWithMetadata,
    handleRagSearch,
    loadRagDocuments,
    handleDeleteRagDocument
  } = useRagLogic();

  // Wrapper functions pour les callbacks
  const handlePrepareInjection = (docName: string) => {
    prepareInjectionMetadata(docName, documents, extractedContent);
    setShowInjectionModal(docName);
  };

  const handleInjectDocument = (docName: string) => {
    handleInjectDocumentWithMetadata(docName, extractedContent, showNotification);
  };

  const handleRagSearchWrapper = () => {
    handleRagSearch(showNotification);
  };

  const handleDeleteRagDocumentWrapper = (documentId: string) => {
    handleDeleteRagDocument(documentId, showNotification);
  };

  // Auto-charger les documents RAG au montage et au changement d'onglet vers "injection"
  useEffect(() => {
    if (activeTab === 'injection') {
      console.log('📚 Auto-loading RAG documents...');
      loadRagDocuments(showNotification);
    }
  }, [activeTab]); // Se déclenche quand on passe à l'onglet injection

  // Aussi charger au montage initial si on est déjà sur injection
  useEffect(() => {
    if (activeTab === 'injection') {
      console.log('📚 Initial load of RAG documents...');
      loadRagDocuments(showNotification);
    }
  }, []); // Une seule fois au montage

  return (
    <>
      <style>{`
        .icon-button {
          background: rgba(255, 255, 255, 0.1);
          border: 1px solid rgba(255, 255, 255, 0.2);
          color: #ffffff;
          cursor: pointer;
          border-radius: 6px;
          font-size: 14px;
          transition: all 0.2s ease;
        }
        .icon-button:hover {
          background: rgba(255, 255, 255, 0.15);
          border-color: rgba(255, 255, 255, 0.3);
        }
        .search-container {
          background: rgba(255, 255, 255, 0.03);
          border-radius: 8px;
          margin-bottom: 20px;
        }
        .search-input-wrapper {
          display: flex;
          align-items: center;
          position: relative;
        }
        
        @keyframes slideIn {
          from {
            transform: translateX(-100%);
            opacity: 0;
          }
          to {
            transform: translateX(0);
            opacity: 1;
          }
        }

        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }

        .spin {
          animation: spin 1s linear infinite;
        }
      `}</style>
      <div style={{ 
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: '#1a1a1a',
        color: '#ffffff',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Inter", sans-serif',
        zIndex: 9999,
        display: 'flex',
        flexDirection: 'column'
      }}>
      {/* Header avec onglets style Settings */}
      <div style={{ 
        background: 'linear-gradient(90deg, #1e293b 0%, #334155 100%)',
        borderBottom: '1px solid #475569',
        padding: '16px 24px 0 24px',
        display: 'flex',
        alignItems: 'end',
        justifyContent: 'space-between'
      }}>
        <div style={{ display: 'flex', gap: '2px', marginBottom: '-1px' }}>
          <button
            onClick={() => setActiveTab('documents')}
            style={{
              padding: '12px 24px 16px 24px',
              background: activeTab === 'documents' 
                ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
              color: activeTab === 'documents' ? '#ffffff' : '#d1d5db',
              border: '1px solid #475569',
              borderBottom: activeTab === 'documents' ? '1px solid #0f172a' : '1px solid #475569',
              borderTopLeftRadius: '12px',
              borderTopRightRadius: '12px',
              borderBottomLeftRadius: '0',
              borderBottomRightRadius: '0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease',
              position: 'relative',
              zIndex: activeTab === 'documents' ? 2 : 1,
              boxShadow: activeTab === 'documents' 
                ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                : '0 2px 4px rgba(0, 0, 0, 0.1)',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}
          >
            <FileText size={16} />
            Documents
          </button>
          <button
            onClick={() => setActiveTab('injection')}
            style={{
              padding: '12px 24px 16px 24px',
              background: activeTab === 'injection' 
                ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
              color: activeTab === 'injection' ? '#ffffff' : '#d1d5db',
              border: '1px solid #475569',
              borderBottom: activeTab === 'injection' ? '1px solid #0f172a' : '1px solid #475569',
              borderTopLeftRadius: '12px',
              borderTopRightRadius: '12px',
              borderBottomLeftRadius: '0',
              borderBottomRightRadius: '0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease',
              position: 'relative',
              zIndex: activeTab === 'injection' ? 2 : 1,
              boxShadow: activeTab === 'injection' 
                ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                : '0 2px 4px rgba(0, 0, 0, 0.1)',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}
          >
            <Database size={16} />
            Injection
          </button>
        </div>
      </div>

      {/* Contenu des onglets - Scrollable */}
      <div style={{ 
        flex: 1, 
        overflow: 'auto',
        background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f1629 100%)'
      }}>
        <div style={{ padding: '24px' }}>
          {activeTab === 'documents' ? (
            <DocumentsTab
              documents={documents}
              isLoadingDocuments={isLoadingDocuments}
              extractedContent={extractedContent}
              showExtraction={showExtraction}
              editingContent={editingContent}
              isEditing={isEditing}
              notification={notification}
              onDeleteDocument={handleDeleteDocument}
              onViewDocument={handleViewDocument}
              onExtractDocument={handleExtractDocument}
              onShowExtraction={handleShowExtraction}
              onSaveExtraction={handleSaveExtraction}
              onCancelEdit={handleCancelEdit}
              onSetShowExtraction={setShowExtraction}
              onSetEditingContent={setEditingContent}
              onSetIsEditing={setIsEditing}
            />
          ) : (
            <InjectionTab
              ragQuery={ragQuery}
              ragResults={ragResults}
              isSearching={isSearching}
              ragDocuments={ragDocuments}
              isLoadingRagDocs={isLoadingRagDocs}
              showInjectionModal={showInjectionModal}
              injectionMetadata={injectionMetadata}
              isProcessing={isProcessing}
              notification={notification}
              extractedContent={extractedContent}
              onSetRagQuery={setRagQuery}
              onRagSearch={handleRagSearchWrapper}
              onDeleteRagDocument={handleDeleteRagDocumentWrapper}
              onSetShowInjectionModal={setShowInjectionModal}
              onSetInjectionMetadata={setInjectionMetadata}
              onSetChunkProfile={setChunkProfile}
              onInjectDocument={handleInjectDocument}
              onPrepareInjection={handlePrepareInjection}
            />
          )}
        </div>
      </div>
      </div>
    </>
  );
};

export default RagWindow;