import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { DocumentInfo, NotificationState } from '@/components/rag/types';

export const useDocuments = () => {
  const [documents, setDocuments] = useState<DocumentInfo[]>([]);
  const [isLoadingDocuments, setIsLoadingDocuments] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const [notification, setNotification] = useState<NotificationState | null>(null);
  const [extractedContent, setExtractedContent] = useState<Record<string, any>>({});
  const [showExtraction, setShowExtraction] = useState<string | null>(null);
  const [editingContent, setEditingContent] = useState<string>('');
  const [isEditing, setIsEditing] = useState<boolean>(false);

  const showNotification = useCallback((message: string, type: 'success' | 'error' | 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  }, []);

  const loadDocuments = useCallback(async () => {
    setIsLoadingDocuments(true);
    try {
      console.log('📂 Loading documents...');
      const result = await invoke<any[]>('list_documents');
      console.log('📂 Documents loaded:', result);

      if (Array.isArray(result)) {
        const mappedDocs: DocumentInfo[] = result.map((doc: any, index: number) => ({
          id: `doc_${index}`,
          name: doc.name || 'Document sans nom',
          size: doc.size || '0 KB',
          sizeBytes: doc.size_bytes || 0,
          type: doc.type || 'unknown',
          status: doc.status || 'ready',
          date: doc.date || new Date().toISOString(),
          category: doc.category || 'document',
          pages: doc.pages || 1,
          extracted: doc.extracted || false,
          extractedAt: doc.extracted_at || '',
          confidence: doc.confidence || 0
        }));
        setDocuments(mappedDocs);
      } else {
        console.warn('⚠️ Documents result is not an array:', result);
        setDocuments([]);
      }
    } catch (error) {
      console.error('❌ Failed to load documents:', error);
      showNotification(`Erreur de chargement: ${error}`, 'error');
      setDocuments([]);
    } finally {
      setIsLoadingDocuments(false);
    }
  }, [showNotification]);

  const handleUploadDocument = useCallback(async () => {
    try {
      setIsUploading(true);
      console.log('📤 Opening file selector...');
      
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Documents',
          extensions: ['pdf', 'txt', 'docx', 'png', 'jpg', 'jpeg']
        }]
      });

      if (!selected) {
        console.log('📤 Upload cancelled by user');
        return;
      }

      const filePath = Array.isArray(selected) ? selected[0] : selected;
      console.log('📤 Selected file:', filePath);

      const fileName = filePath.split('/').pop() || 'unknown';
      console.log('📤 Uploading file:', fileName);
      
      const result = await invoke<string>('upload_document', {
        filePath: filePath,
        targetName: fileName
      });
      
      console.log('📤 Upload result:', result);
      await loadDocuments();
      
      showNotification(`Document "${fileName}" uploadé avec succès!`, 'success');
      
    } catch (error) {
      console.error('❌ Upload failed:', error);
      showNotification(`Erreur d'upload: ${error}`, 'error');
    } finally {
      setIsUploading(false);
    }
  }, [loadDocuments, showNotification]);

  const handleDeleteDocument = useCallback(async (docId: string, docName: string) => {
    try {
      console.log(`Deleting document: ${docName}`);
      
      await invoke<string>('delete_document', {
        filename: docName
      });
      
      setDocuments(prev => prev.filter(doc => doc.id !== docId));
      showNotification(`Document "${docName}" supprimé`, 'success');
      
    } catch (error) {
      console.error('Delete failed:', error);
      showNotification(`Erreur de suppression: ${error}`, 'error');
    }
  }, [showNotification]);

  const handleViewDocument = useCallback(async (filename: string) => {
    try {
      const result = await invoke<string>('open_document_viewer', { filename });
      showNotification(result, 'success');
    } catch (error) {
      console.error('❌ Failed to open document:', error);
      showNotification(`Erreur d'ouverture: ${error}`, 'error');
    }
  }, [showNotification]);

  const handleExtractDocument = useCallback(async (docName: string) => {
    try {
      console.log(`🔍 Extracting document: ${docName}`);
      const result = await invoke<any>('extract_document_content', {
        filename: docName
      });
      
      console.log('📄 Extraction result:', result);
      
      setExtractedContent(prev => ({
        ...prev,
        [docName]: result
      }));
      
      showNotification(`Extraction de "${docName}" terminée`, 'success');
      
    } catch (error) {
      console.error('❌ Extraction failed:', error);
      showNotification(`Erreur lors de l'extraction: ${error}`, 'error');
    }
  }, [showNotification]);

  const handleShowExtraction = useCallback((filename: string) => {
    if (extractedContent[filename]) {
      setShowExtraction(filename);
      setEditingContent(extractedContent[filename].content || '');
      setIsEditing(false);
    } else {
      showNotification('Aucune extraction disponible pour ce document', 'info');
    }
  }, [extractedContent, showNotification]);

  const handleSaveExtraction = useCallback(() => {
    if (showExtraction) {
      setExtractedContent(prev => ({
        ...prev,
        [showExtraction]: {
          ...prev[showExtraction],
          content: editingContent,
          modified: true,
          modified_at: new Date().toLocaleString()
        }
      }));
      setIsEditing(false);
      showNotification('Extraction mise à jour', 'success');
    }
  }, [showExtraction, editingContent, showNotification]);

  const handleCancelEdit = useCallback(() => {
    if (showExtraction) {
      setEditingContent(extractedContent[showExtraction].content || '');
      setIsEditing(false);
    }
  }, [showExtraction, extractedContent]);

  useEffect(() => {
    loadDocuments();
  }, [loadDocuments]);

  return {
    // State
    documents,
    isLoadingDocuments,
    isUploading,
    notification,
    extractedContent,
    showExtraction,
    editingContent,
    isEditing,
    
    // Actions
    loadDocuments,
    handleUploadDocument,
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
  };
};