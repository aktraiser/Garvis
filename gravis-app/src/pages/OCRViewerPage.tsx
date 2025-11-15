// OCRViewerPage - Fenêtre séparée pour l'OCR Viewer
// PR #4 Phase 3 - Interface OCR avec highlighting temps réel

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SimplePdfViewer from '@/components/SimplePdfViewer';

interface DirectChatSession {
  session_id: string;
  document_name: string;
}


export function OCRViewerPage() {
  console.log('🚀 OCRViewerPage component mounting...');
  
  const [session, setSession] = useState<DirectChatSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    console.log('🔍 OCRViewerPage: Extracting session ID from URL...');
    console.log('🔍 Current URL:', window.location.href);
    console.log('🔍 URL hash:', window.location.hash);
    
    // Extract session ID from URL
    const params = new URLSearchParams(window.location.hash.split('?')[1]);
    const sessionId = params.get('session');
    
    console.log('🔍 Extracted sessionId:', sessionId);

    if (!sessionId) {
      console.log('❌ No session ID found in URL');
      setError('Session ID not provided');
      setLoading(false);
      return;
    }

    // Load session data
    const loadSession = async () => {
      try {
        console.log('🔄 Loading session data for:', sessionId);
        const sessionData = await invoke<DirectChatSession>('get_direct_chat_session', {
          sessionId: sessionId
        });
        console.log('✅ Session data loaded:', sessionData);
        setSession(sessionData);
        setLoading(false);
      } catch (err) {
        console.error('❌ Failed to load session:', err);
        setError(`Failed to load session: ${err}`);
        setLoading(false);
      }
    };

    loadSession();

    // Plus besoin d'écouter les highlights avec SimplePdfViewer
  }, []);

  // Gérer les actions sur le texte sélectionné - Memoized pour éviter re-renders
  const handleTextAction = useCallback(async (action: 'explain' | 'summarize', text: string) => {
    try {
      console.log(`🎯 OCRViewerPage: ${action} requested for text:`, text);
      console.log('🎯 OCRViewerPage: session exists:', !!session);
      
      if (!session) {
        console.error('❌ No session available for RAG query');
        return;
      }
      
      // 🎯 ENVOYER LA QUESTION À LA FENÊTRE PRINCIPALE DE CONVERSATION
      const question = action === 'explain' 
        ? `Explique ce concept ou terme : "${text}"`
        : `Résume cette section ou information : "${text}"`;
        
      console.log(`📤 Sending question to main conversation window: "${question}"`);
      
      // Envoyer la question automatiquement à la fenêtre de conversation principale
      await invoke('broadcast_to_window', {
        windowLabel: 'main',
        event: 'auto_question_from_ocr',
        payload: {
          question: question,
          selected_text: text,
          action: action,
          session_id: session.session_id,
          document_name: session.document_name
        }
      });
      
      console.log(`✅ ${action.toUpperCase()} SUCCESS: Question envoyée à la fenêtre principale !`);
      
      // Note: La réponse complète sera affichée dans l'interface principale
      // selon le pattern actuel de l'application
      
    } catch (error) {
      console.error(`❌ Error sending ${action} request:`, error);
      // Log l'erreur au lieu d'utiliser notification pour éviter re-renders
      console.error(`❌ ${action.toUpperCase()} FAILED:`, error);
    }
  }, [session]); // Dépendance sur session seulement

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p>Chargement du document...</p>
          <p className="text-xs mt-2 opacity-70">OCR Viewer Window</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <p className="text-xl">{error}</p>
          <p className="text-xs mt-2 opacity-70">OCR Viewer Window - Error</p>
        </div>
      </div>
    );
  }

  if (!session) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-yellow-500 text-6xl mb-4">📄</div>
          <p className="text-xl">Session non trouvée</p>
          <p className="text-xs mt-2 opacity-70">OCR Viewer Window - No Session</p>
        </div>
      </div>
    );
  }

  return (
    <div style={{ height: '100vh', width: '100vw', overflow: 'hidden', position: 'relative' }}>
      {/* 🎯 Nouveau viewer simple avec interactions texte natives */}
      <SimplePdfViewer
        sessionId={session.session_id}
        onTextAction={(action, text) => {
          console.log('🔗 OCRViewerPage: onTextAction wrapper called with:', { action, text });
          handleTextAction(action, text);
        }}
      />

      {/* Plus de notifications - utilise la console pour éviter re-renders */}

      {/* Debug simple */}
      {process.env.NODE_ENV === 'development' && (
        <div className="absolute top-4 left-4 bg-black bg-opacity-50 text-white text-xs p-2 rounded z-20">
          <div>🎯 SimplePdfViewer Mode</div>
          <div>📄 Session: {session.session_id.substring(0, 8)}...</div>
        </div>
      )}
    </div>
  );
}
