// useDirectChat - Hook pour gérer le drag & drop et le chat direct avec documents
// Centralise toute la logique Direct Chat (PR #4)
// Sprint 1 Niveau 1: LLM Synthesis intégré

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { chatWithLlmSynthesis } from '@/lib/llm-synthesis';
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

// ChatResponse legacy type removed - now using LLM synthesis

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

  // Handle direct chat with dropped document - Sprint 1 Niveau 1: LLM Synthesis
  const chatWithDocument = async (userQuery: string): Promise<{
    success: boolean;
    content: string;
  }> => {
    if (!directChatSession) {
      return { success: false, content: 'Aucune session active' };
    }

    try {
      console.log('🤖 Using LLM Synthesis for query:', userQuery);

      // Sprint 1 Niveau 1: Appel LLM synthesis au lieu de chunks bruts
      // TEST A/B: Temporarily back to top-10 to debug "16x compressor" recall issue
      const llmResponse = await chatWithLlmSynthesis(
        directChatSession.session_id,
        userQuery,
        null,  // selection
        10     // TEST: back to 10 chunks to check recall
      );

      // Formater la réponse avec sources
      let responseContent = llmResponse.answer;

      // Ajouter les sources si disponibles
      if (llmResponse.sources && llmResponse.sources.length > 0) {
        responseContent += "\n\n**📚 Sources :**\n";
        llmResponse.sources.slice(0, 5).forEach((source, idx) => {
          const score = Math.round(source.score * 100);
          const confidence = Math.round(source.confidence * 100);
          responseContent += `\n${idx + 1}. **[${score}%]** [${source.source_label}] ${source.content.substring(0, 80)}... *(confiance: ${confidence}%)*`;
        });
      }

      // Ajouter métriques
      const totalTime = llmResponse.search_time_ms + llmResponse.llm_time_ms;
      responseContent += `\n\n*Confiance: ${Math.round(llmResponse.confidence * 100)}% • RAG: ${llmResponse.search_time_ms}ms • LLM: ${llmResponse.llm_time_ms}ms • Total: ${totalTime}ms*`;

      console.log(`✅ LLM synthesis complete: ${totalTime}ms total`);

      return {
        success: true,
        content: responseContent
      };
    } catch (error) {
      console.error('❌ Erreur LLM synthesis:', error);
      return {
        success: false,
        content: `❌ Erreur LLM: ${error}`
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
