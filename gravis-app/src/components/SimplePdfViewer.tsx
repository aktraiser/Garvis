// SimplePdfViewer - PDF natif avec interactions texte simples
// Utilise les événements natifs react-pdf pour hover et sélection

import React, { useRef, useState, useCallback } from 'react';
import { Document, Page, pdfjs } from 'react-pdf';
import pdfjsWorker from 'pdfjs-dist/build/pdf.worker?url';
import 'react-pdf/dist/Page/AnnotationLayer.css';
import 'react-pdf/dist/Page/TextLayer.css';

// Configure PDF.js worker
pdfjs.GlobalWorkerOptions.workerSrc = pdfjsWorker;

interface SimplePdfViewerProps {
  sessionId: string;
  onTextAction?: (action: 'explain' | 'summarize', text: string) => void;
}

export const SimplePdfViewer: React.FC<SimplePdfViewerProps> = ({
  sessionId,
  onTextAction,
}) => {
  console.log('🔄 SimplePdfViewer mounted with sessionId:', sessionId);
  console.log('🔄 Component props:', { sessionId, onTextAction: !!onTextAction });
  const containerRef = useRef<HTMLDivElement | null>(null);
  const contextMenuRef = useRef<{ x: number; y: number; text: string } | null>(null);
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [numPages, setNumPages] = useState<number | null>(null);
  const [pageWidth, setPageWidth] = useState<number>(600);
  const [selectedText, setSelectedText] = useState<string>('');
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    text: string;
  } | null>(null);

  // Memoize the file prop
  const pdfFile = React.useMemo(() => {
    if (!pdfData) return null;
    return { data: pdfData };
  }, [pdfData]);

  // Calculer largeur responsive
  React.useEffect(() => {
    const updateWidth = () => {
      if (containerRef.current) {
        const availableWidth = containerRef.current.clientWidth * 0.7;
        const maxWidth = Math.min(availableWidth, 800);
        setPageWidth(maxWidth);
      }
    };

    setTimeout(updateWidth, 100);
    window.addEventListener('resize', updateWidth);
    return () => window.removeEventListener('resize', updateWidth);
  }, []);

  // Charger le PDF via Tauri
  React.useEffect(() => {
    const loadPdf = async () => {
      try {
        setLoading(true);
        const { invoke } = await import('@tauri-apps/api/core');

        console.log('📄 Loading PDF for session:', sessionId);
        const pdfBytes = await invoke<number[]>('get_pdf_for_session', {
          sessionId
        });

        const uint8Array = new Uint8Array(pdfBytes);
        console.log('✅ PDF data loaded:', uint8Array.length, 'bytes');
        setPdfData(uint8Array);
        setLoading(false);
      } catch (err) {
        console.error('❌ Failed to load PDF:', err);
        setError(`Failed to load PDF: ${err}`);
        setLoading(false);
      }
    };

    loadPdf();
  }, [sessionId]);

  // 🎯 Événements de sélection de texte natifs
  React.useEffect(() => {
    console.log('🔧 SimplePdfViewer: Setting up text selection listeners');
    
    const handleTextSelection = (e: MouseEvent) => {
      console.log('🖱️ Mouse up detected on:', e.target);
      console.log('🖱️ Event type:', e.type);
      
      // Petit délai pour laisser la sélection se faire
      setTimeout(() => {
        const selection = window.getSelection();
        console.log('📍 Selection object:', selection);
        console.log('📍 Selection text:', selection?.toString());
        console.log('📍 Selection range count:', selection?.rangeCount);
        
        if (!selection || selection.toString().trim() === '') {
          console.log('❌ No text selected');
          contextMenuRef.current = null;
          contextMenuRef.current = null;
      setContextMenu(null);
          return;
        }

        const text = selection.toString().trim();
        console.log('✅ Text selected:', text);
        setSelectedText(text);

        // Obtenir position de la sélection pour le menu contextuel
        if (selection.rangeCount > 0) {
          const range = selection.getRangeAt(0);
          const rect = range.getBoundingClientRect();

          console.log('📍 Selection rect:', rect);

          const menuData = {
            x: rect.right + 10,
            y: rect.top,
            text: text,
          };
          
          contextMenuRef.current = menuData;
          setContextMenu(menuData);

          console.log('✅ Context menu should appear at:', { x: rect.right + 10, y: rect.top });
        }
      }, 100);
    };

    // Test multiple events
    const handleMouseDown = () => {
      console.log('🖱️ Mouse down detected');
    };

    const handleSelectionChange = () => {
      console.log('📝 Selection change detected');
      const selection = window.getSelection();
      console.log('📝 Current selection:', selection?.toString());
    };

    // Fermer menu si clic ailleurs
    const handleClickOutside = (e: MouseEvent) => {
      if (contextMenuRef.current && !(e.target as Element).closest('.context-menu')) {
        contextMenuRef.current = null;
        contextMenuRef.current = null;
      setContextMenu(null);
      }
    };

    // Ajouter les listeners
    document.addEventListener('mouseup', handleTextSelection);
    document.addEventListener('mousedown', handleMouseDown);
    document.addEventListener('selectionchange', handleSelectionChange);
    document.addEventListener('click', handleClickOutside);

    return () => {
      document.removeEventListener('mouseup', handleTextSelection);
      document.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('selectionchange', handleSelectionChange);
      document.removeEventListener('click', handleClickOutside);
    };
  }, []); // Remove contextMenu dependency to prevent infinite re-renders

  // 🎯 Custom text renderer pour highlight au hover
  const customTextRenderer = useCallback((textItem: any) => {
    // Pour l'instant, on retourne le texte normal
    // Plus tard on pourra ajouter des highlights dynamiques
    return textItem.str;
  }, []);

  // 🎯 Gérer les actions sur le texte
  const handleTextAction = (action: 'explain' | 'summarize') => {
    console.log('🎯 handleTextAction called with:', action);
    console.log('🎯 selectedText:', selectedText);
    console.log('🎯 onTextAction exists:', !!onTextAction);
    
    if (selectedText && onTextAction) {
      console.log('✅ Calling onTextAction with:', { action, text: selectedText });
      onTextAction(action, selectedText);
      contextMenuRef.current = null;
      setContextMenu(null);
    } else {
      console.log('❌ Cannot call onTextAction:', { 
        hasSelectedText: !!selectedText, 
        hasOnTextAction: !!onTextAction 
      });
      
      // Fallback pour test
      if (selectedText) {
        console.log(`🔄 Fallback: Would ${action} "${selectedText}"`);
        console.log(`📝 FALLBACK ${action.toUpperCase()}: "${selectedText}"`);
        contextMenuRef.current = null;
      setContextMenu(null);
      }
    }
  };

  // Callbacks PDF.js
  const onDocumentLoadSuccess = ({ numPages }: { numPages: number }) => {
    console.log('✅ PDF loaded successfully:', numPages, 'pages');
    setNumPages(numPages);
  };

  const onPageLoadSuccess = () => {
    console.log('✅ Page rendered successfully');
  };

  const onGetTextSuccess = ({ items }: { items: any[] }) => {
    console.log('✅ Text layer loaded:', items.length, 'items');
    console.log('🎯 PDF should now be selectable!');
    console.log('📝 Sample text items:', items.slice(0, 3).map(item => item.str));
  };

  // États de chargement
  if (loading) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p>Chargement du PDF...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <p className="text-xl text-red-400">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        width: '100%',
        height: '100%',
        overflow: 'auto',
        backgroundColor: '#525252',
      }}
    >
      {pdfFile ? (
        <div style={{
          position: 'relative',
          width: '100%',
          minHeight: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          padding: '20px 0',
          gap: '20px',
        }}>
          <Document
            file={pdfFile}
            onLoadSuccess={onDocumentLoadSuccess}
            onLoadError={(error) => {
              console.error('❌ PDF.js load error:', error);
              setError(`Failed to load PDF: ${error.message}`);
            }}
          >
            {/* Afficher toutes les pages */}
            {Array.from(new Array(numPages), (_el, index) => (
              <div key={`page_${index + 1}`} style={{ position: 'relative' }}>
                <Page
                  pageNumber={index + 1}
                  width={pageWidth}
                  renderTextLayer={true}
                  renderAnnotationLayer={true}
                  customTextRenderer={customTextRenderer}
                  onLoadSuccess={onPageLoadSuccess}
                  onGetTextSuccess={onGetTextSuccess}
                  onGetTextError={(error) => {
                    console.error('Text layer error:', error);
                  }}
                />
              </div>
            ))}
          </Document>

          {/* 🎯 Menu contextuel pour sélection de texte */}
          {contextMenu && (
            <div
              className="context-menu"
              style={{
                position: 'fixed',
                left: `${contextMenu.x}px`,
                top: `${contextMenu.y}px`,
                background: '#1f2937',
                border: '1px solid #374151',
                borderRadius: '8px',
                padding: '8px',
                boxShadow: '0 10px 25px rgba(0, 0, 0, 0.3)',
                zIndex: 1000,
                display: 'flex',
                gap: '8px',
              }}
            >
              <button
                onClick={() => handleTextAction('explain')}
                style={{
                  background: '#3b82f6',
                  color: 'white',
                  border: 'none',
                  padding: '6px 12px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  cursor: 'pointer',
                  transition: 'background 0.2s',
                }}
                onMouseEnter={(e) => e.currentTarget.style.background = '#2563eb'}
                onMouseLeave={(e) => e.currentTarget.style.background = '#3b82f6'}
              >
                Expliquer
              </button>
              <button
                onClick={() => handleTextAction('summarize')}
                style={{
                  background: '#10b981',
                  color: 'white',
                  border: 'none',
                  padding: '6px 12px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  cursor: 'pointer',
                  transition: 'background 0.2s',
                }}
                onMouseEnter={(e) => e.currentTarget.style.background = '#059669'}
                onMouseLeave={(e) => e.currentTarget.style.background = '#10b981'}
              >
                Résumer
              </button>
            </div>
          )}
        </div>
      ) : (
        <div className="flex items-center justify-center h-full bg-yellow-100">
          <div className="text-center">
            <p className="text-yellow-700 font-semibold">🚧 PDF en cours de chargement</p>
            <p className="text-sm text-gray-600">Session: {sessionId}</p>
          </div>
        </div>
      )}

      {/* Debug info */}
      {process.env.NODE_ENV === 'development' && (
        <div className="absolute top-4 right-4 bg-black bg-opacity-75 text-white text-xs p-3 rounded z-30 max-w-xs">
          <div>📄 PDF Pages: {numPages}</div>
          <div>🖥️ PDF Width: {pageWidth}px</div>
          <div>📍 Selected: {selectedText.length > 0 ? `"${selectedText.substring(0, 30)}..."` : 'None'}</div>
        </div>
      )}
    </div>
  );
};

export default SimplePdfViewer;