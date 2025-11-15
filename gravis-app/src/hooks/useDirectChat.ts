// useDirectChat - Hook pour gérer le drag & drop et le chat direct avec documents
// Centralise toute la logique Direct Chat (PR #4)

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
// Types simplifiés sans composants OCR complexes

interface DirectChatSession {
  session_id: string;
  document_name: string;
}

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
  sources_summary: Array<{
    chunk_id: string;
    content_preview: string;
    score: number;
    confidence: number;
    span_count: number;
  }>;
}

export function useDirectChat() {
  // States
  const [isDragging, setIsDragging] = useState(false);
  const [_dragCounter, setDragCounter] = useState(0);
  const [droppedFile, setDroppedFile] = useState<File | null>(null);
  const [directChatSession, setDirectChatSession] = useState<DirectChatSession | null>(null);
  const [showOCRViewer, setShowOCRViewer] = useState(false);

  // Drag & Drop handlers - Use counter to handle nested elements correctly
  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragCounter(prev => {
      const newCount = prev + 1;
      if (newCount === 1) setIsDragging(true);
      return newCount;
    });
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragCounter(prev => {
      const newCount = prev - 1;
      if (newCount === 0) setIsDragging(false);
      return newCount;
    });
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
      setDroppedFile(file);
      await processDroppedDocument(file);
    }
  };

  // Process dropped document
  const processDroppedDocument = async (file: File): Promise<{ success: boolean; message?: string }> => {
    try {
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      const response = await invoke<ProcessDocumentResponse>('process_dropped_document', {
        filePath: file.name,
        fileData: Array.from(uint8Array),
        mimeType: file.type || 'application/pdf'
      });

      if (response.session) {
        setDirectChatSession({
          session_id: response.session.session_id,
          document_name: response.session.document_name,
        });

        // Open OCR viewer window positioned next to main window
        try {
          await invoke('open_ocr_viewer_window', {
            sessionId: response.session.session_id
          });
        } catch (error) {
          console.warn('Failed to open OCR viewer window:', error);
          // Fallback to inline panel if window creation fails
          setShowOCRViewer(true);
        }

        return {
          success: true,
          message: `📄 Document chargé: ${response.session.document_name} (${response.chunks_created} sections, ${response.processing_time_ms}ms)`
        };
      }

      return { success: false, message: 'Erreur: Pas de session retournée' };
    } catch (error) {
      console.error('Erreur traitement:', error);
      return { success: false, message: `❌ Erreur: ${error}` };
    }
  };

  // Handle direct chat with dropped document
  const chatWithDocument = async (userQuery: string): Promise<{
    success: boolean;
    content: string;
  }> => {
    if (!directChatSession) {
      return { success: false, content: 'Aucune session active' };
    }

    try {
      const response = await invoke<ChatResponse>('chat_with_dropped_document', {
        request: {
          session_id: directChatSession.session_id,
          query: userQuery,
          selection: null,
          limit: null,
        }
      });

      // Plus de spans highlighting - système simplifié

      // Format response with sources
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

      return {
        success: true,
        content: responseContent
      };
    } catch (error) {
      console.error('Erreur chat direct:', error);
      return {
        success: false,
        content: `❌ Erreur: ${error}`
      };
    }
  };

  // Remove dropped file and reset session
  const removeDroppedFile = async () => {
    // Close OCR viewer window if open
    try {
      await invoke('close_ocr_viewer_window');
    } catch (error) {
      console.warn('Failed to close OCR viewer window:', error);
    }

    setDroppedFile(null);
    setDirectChatSession(null);
    setShowOCRViewer(false);
    setIsDragging(false);
    setDragCounter(0);
  };

  // Reset all direct chat state
  const resetDirectChat = async () => {
    // Close OCR viewer window if open
    try {
      await invoke('close_ocr_viewer_window');
    } catch (error) {
      console.warn('Failed to close OCR viewer window:', error);
    }

    setDroppedFile(null);
    setDirectChatSession(null);
    setShowOCRViewer(false);
    setIsDragging(false);
    setDragCounter(0);
  };

  return {
    // State
    isDragging,
    droppedFile,
    directChatSession,
    showOCRViewer,

    // Handlers
    dragHandlers: {
      onDragEnter: handleDragEnter,
      onDragLeave: handleDragLeave,
      onDragOver: handleDragOver,
      onDrop: handleDrop,
    },

    // Actions
    processDroppedDocument,
    chatWithDocument,
    removeDroppedFile,
    resetDirectChat,

    // Computed
    hasActiveSession: !!directChatSession,
  };
}
