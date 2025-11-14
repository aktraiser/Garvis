// OCRViewerPage - Fenêtre séparée pour l'OCR Viewer
// PR #4 Phase 3 - Interface OCR avec highlighting temps réel

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { OCRViewerWithSpans, type SourceSpan, type OCRContent } from '@/components/OCRViewerWithSpans';

interface DirectChatSession {
  session_id: string;
  document_name: string;
  chunks: any[];
  ocr_content: OCRContent;
}

export function OCRViewerPage() {
  const [session, setSession] = useState<DirectChatSession | null>(null);
  const [highlightedSpans, setHighlightedSpans] = useState<SourceSpan[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Extract session ID from URL
    const params = new URLSearchParams(window.location.hash.split('?')[1]);
    const sessionId = params.get('session');

    if (!sessionId) {
      setError('Session ID not provided');
      setLoading(false);
      return;
    }

    // Load session data
    const loadSession = async () => {
      try {
        const sessionData = await invoke<DirectChatSession>('get_direct_chat_session', {
          sessionId: sessionId
        });
        setSession(sessionData);
        setLoading(false);
      } catch (err) {
        console.error('Failed to load session:', err);
        setError(`Failed to load session: ${err}`);
        setLoading(false);
      }
    };

    loadSession();

    // Listen for highlight updates from main window
    const unlisten = listen<SourceSpan[]>('update_highlights', (event) => {
      console.log('Received highlight update:', event.payload);
      setHighlightedSpans(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p>Chargement du document...</p>
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
        </div>
      </div>
    );
  }

  if (!session || !session.ocr_content) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-yellow-500 text-6xl mb-4">📄</div>
          <p className="text-xl">Aucun contenu OCR disponible</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen bg-gray-900 overflow-hidden">
      <OCRViewerWithSpans
        documentName={session.document_name}
        ocrContent={session.ocr_content}
        highlightedSpans={highlightedSpans}
        onSpanClick={(span) => {
          console.log('Span clicked:', span);
          // Could send event back to main window if needed
        }}
        onTextSelection={(text) => {
          console.log('Text selected:', text);
          // Could send selection back to main window for targeted questions
        }}
      />
    </div>
  );
}
