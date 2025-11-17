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
  const containerRef = useRef<HTMLDivElement | null>(null);
  const contextMenuRef = useRef<{ x: number; y: number; text: string } | null>(null);
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [numPages, setNumPages] = useState<number | null>(null);
  const [pageWidth, setPageWidth] = useState<number>(600);
  const [zoomLevel, setZoomLevel] = useState<number>(1.0);
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
        const { invoke } = await import('@tauri-apps/api/core');

        const pdfBytes = await invoke<number[]>('get_pdf_for_session', {
          sessionId
        });

        const uint8Array = new Uint8Array(pdfBytes);
        setPdfData(uint8Array);
      } catch (err) {
        console.error('❌ Failed to load PDF:', err);
        setError(`Failed to load PDF: ${err}`);
      }
    };

    loadPdf();
  }, [sessionId]);

  // 🎯 Événements de sélection de texte natifs avec améliorations pour grandes sélections
  React.useEffect(() => {
    let selectionTimeout: NodeJS.Timeout | null = null;
    let isDragging = false;
    let dragStartTime = 0;

    const handleMouseDown = (_e: MouseEvent) => {
      isDragging = true;
      dragStartTime = Date.now();

      // Cacher le menu pendant la sélection
      if (contextMenuRef.current) {
        contextMenuRef.current = null;
        setContextMenu(null);
      }
    };

    const handleTextSelection = (_e?: MouseEvent | KeyboardEvent) => {
      isDragging = false;
      const dragDuration = Date.now() - dragStartTime;

      // Effacer le timeout précédent
      if (selectionTimeout) {
        clearTimeout(selectionTimeout);
      }

      // Délai adaptatif basé sur la durée de drag
      const adaptiveDelay = Math.min(500, Math.max(150, dragDuration * 0.3));

      selectionTimeout = setTimeout(() => {
        const selection = window.getSelection();

        if (!selection || selection.toString().trim() === '') {
          contextMenuRef.current = null;
          setContextMenu(null);
          return;
        }

        const text = selection.toString().trim();
        setSelectedText(text);

        // Obtenir position de la sélection pour le menu contextuel
        if (selection.rangeCount > 0) {
          const range = selection.getRangeAt(0);
          const rect = range.getBoundingClientRect();

          // Positionner le menu au-dessus de la sélection, centré
          const menuHeight = 50;
          let menuX = rect.left + (rect.width / 2);
          let menuY = rect.top - menuHeight;

          // Protection contre débordement en haut de l'écran
          if (menuY < 10) {
            menuY = rect.bottom + 10;
          }

          // Protection contre débordement sur les côtés
          const menuWidth = 200;
          if (menuX - menuWidth/2 < 10) {
            menuX = menuWidth/2 + 10;
          } else if (menuX + menuWidth/2 > window.innerWidth - 10) {
            menuX = window.innerWidth - menuWidth/2 - 10;
          }

          const menuData = {
            x: menuX,
            y: menuY,
            text: text,
          };

          contextMenuRef.current = menuData;
          setContextMenu(menuData);
        }
      }, adaptiveDelay);
    };

    const handleSelectionChange = () => {
      // Ne pas traiter si on est en train de faire un drag
      if (isDragging) return;

      const selection = window.getSelection();
      const text = selection?.toString().trim() || '';

      // Cacher le menu si la sélection change et qu'on a déjà un menu affiché
      if (contextMenuRef.current && text.length > 0 && text !== contextMenuRef.current.text) {
        contextMenuRef.current = null;
        setContextMenu(null);
      }
    };

    // Gestion du double-clic pour sélection rapide de mots
    const handleDoubleClick = (e: MouseEvent) => {
      // Le navigateur gère déjà la sélection de mot au double-clic
      // On laisse un petit délai puis on traite la sélection
      setTimeout(() => {
        handleTextSelection(e);
      }, 50);
    };

    // Fermer menu si clic ailleurs
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Element;
      // Ne fermer que si on clique vraiment en dehors (pas sur le menu ou ses enfants)
      if (contextMenuRef.current && !target.closest('.context-menu') && !target.closest('button')) {
        contextMenuRef.current = null;
        setContextMenu(null);
      }
    };

    // Gestion des raccourcis clavier
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+A pour sélectionner tout le texte visible
      if (e.ctrlKey && e.key === 'a' && containerRef.current) {
        e.preventDefault();

        // Sélectionner tout le texte dans le conteneur PDF
        const range = document.createRange();
        const textLayers = containerRef.current.querySelectorAll('.react-pdf__Page__textContent');

        if (textLayers.length > 0) {
          range.setStartBefore(textLayers[0]);
          range.setEndAfter(textLayers[textLayers.length - 1]);

          const selection = window.getSelection();
          selection?.removeAllRanges();
          selection?.addRange(range);

          // Traiter la sélection après un court délai
          setTimeout(() => {
            const newSelection = window.getSelection();
            const text = newSelection?.toString().trim() || '';
            if (text) {
              handleTextSelection(e as any);
            }
          }, 100);
        }
      }

      // Escape pour fermer le menu contextuel
      if (e.key === 'Escape' && contextMenuRef.current) {
        contextMenuRef.current = null;
        setContextMenu(null);
      }
    };

    // Ajouter les listeners avec nettoyage du timeout
    document.addEventListener('mouseup', handleTextSelection);
    document.addEventListener('mousedown', handleMouseDown);
    document.addEventListener('dblclick', handleDoubleClick);
    document.addEventListener('selectionchange', handleSelectionChange);
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      // Nettoyer le timeout si le composant se démonte
      if (selectionTimeout) {
        clearTimeout(selectionTimeout);
      }
      
      document.removeEventListener('mouseup', handleTextSelection);
      document.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('dblclick', handleDoubleClick);
      document.removeEventListener('selectionchange', handleSelectionChange);
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []); // Remove contextMenu dependency to prevent infinite re-renders

  // 🎯 Intercepter les liens PDF pour les ouvrir dans le browser externe
  React.useEffect(() => {
    const handleLinkClick = async (e: MouseEvent) => {
      const target = e.target as HTMLElement;

      // Vérifier si c'est un lien dans l'annotation layer
      if (target.tagName === 'A' && target.closest('.react-pdf__Page__annotations')) {
        e.preventDefault();
        e.stopPropagation();
        const href = (target as HTMLAnchorElement).href;

        console.log('🔗 Opening link in external browser:', href);

        try {
          // Utiliser l'API Tauri pour ouvrir dans le browser externe
          const { openUrl } = await import('@tauri-apps/plugin-opener');
          await openUrl(href);
          console.log('✅ Link opened successfully in external browser');
        } catch (error) {
          console.error('❌ Failed to open link with Tauri opener:', error);
          // Fallback sur window.open si Tauri échoue
          try {
            window.open(href, '_blank', 'noopener,noreferrer');
            console.log('✅ Fallback window.open succeeded');
          } catch (fallbackError) {
            console.error('❌ Fallback window.open failed:', fallbackError);
          }
        }
      }
    };

    // Écouter les clics sur tout le document
    document.addEventListener('click', handleLinkClick, true);

    return () => {
      document.removeEventListener('click', handleLinkClick, true);
    };
  }, []);

  // 🎯 Custom text renderer pour highlight au hover
  const customTextRenderer = useCallback((textItem: any) => {
    // Pour l'instant, on retourne le texte normal
    // Plus tard on pourra ajouter des highlights dynamiques
    return textItem.str;
  }, []);


  // Callbacks PDF.js
  const onDocumentLoadSuccess = ({ numPages }: { numPages: number }) => {
    setNumPages(numPages);
  };

  const onPageLoadSuccess = () => {
    // Page rendered successfully
  };

  const onGetTextSuccess = (_data: { items: any[] }) => {
    // Text layer loaded and selectable
  };

  // États de chargement
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
            // Améliorer la sélection de texte pour les grands passages
            userSelect: 'text',
            WebkitUserSelect: 'text',
            MozUserSelect: 'text',
            msUserSelect: 'text',
            // Performance pour les sélections longues
            WebkitTouchCallout: 'none',
            WebkitTapHighlightColor: 'transparent',
            // Scrolling plus fluide pendant la sélection
            scrollBehavior: 'smooth',
            // Améliore la fluidité des sélections longues
            willChange: 'scroll-position',
          }}
        >

      {/* Zoom Controls - Fixed bottom left */}
      <div style={{
        position: 'fixed',
        bottom: 20,
        left: 20,
        zIndex: 100,
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
      }}>
        <button
          onClick={() => setZoomLevel(prev => Math.min(prev + 0.25, 3.0))}
          style={{
            background: '#374151',
            border: '1px solid #4b5563',
            color: 'white',
            fontSize: '14px',
            width: '28px',
            height: '28px',
            borderRadius: '4px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
          title="Zoom in"
        >
          +
        </button>
        <button
          onClick={() => setZoomLevel(prev => Math.max(prev - 0.25, 0.5))}
          style={{
            background: '#374151',
            border: '1px solid #4b5563',
            color: 'white',
            fontSize: '14px',
            width: '28px',
            height: '28px',
            borderRadius: '4px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
          title="Zoom out"
        >
          -
        </button>
      </div>

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
                  width={pageWidth * zoomLevel}
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
              onMouseDown={(e) => {
                // Empêcher le mousedown de se propager et de déclencher handleMouseDown
                e.stopPropagation();
              }}
              onClick={(e) => {
                // Empêcher le click de se propager
                e.stopPropagation();
              }}
              style={{
                position: 'fixed',
                left: `${contextMenu.x}px`,
                top: `${contextMenu.y}px`,
                transform: 'translateX(-50%)', // Centrer le menu horizontalement
                zIndex: 1000,
                pointerEvents: 'auto', // S'assurer que les événements de souris fonctionnent
              }}
            >
              <div
                className="context-menu"
                style={{
                  background: '#1f2937',
                  border: '1px solid #374151',
                  borderRadius: '8px',
                  padding: '8px',
                  boxShadow: '0 10px 25px rgba(0, 0, 0, 0.3)',
                  display: 'flex',
                  gap: '8px',
                  position: 'relative', // Pour positionner la flèche
                  pointerEvents: 'auto', // S'assurer que les boutons sont cliquables
                }}
              >
              <button
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();

                  if (selectedText && onTextAction) {
                    onTextAction('explain', selectedText);
                  }

                  // Fermer menu
                  contextMenuRef.current = null;
                  setContextMenu(null);
                }}
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
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();

                  if (selectedText && onTextAction) {
                    onTextAction('summarize', selectedText);
                  }

                  // Fermer menu
                  contextMenuRef.current = null;
                  setContextMenu(null);
                }}
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
              
              {/* Flèche pointant vers la sélection */}
              <div
                style={{
                  position: 'absolute',
                  bottom: '-6px',
                  left: '50%',
                  transform: 'translateX(-50%)',
                  width: 0,
                  height: 0,
                  borderLeft: '6px solid transparent',
                  borderRight: '6px solid transparent',
                  borderTop: '6px solid #1f2937',
                  zIndex: 1001
                }}
              />
              </div>
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

      {/* Debug info avec indicateur d'amélioration de sélection */}
      {process.env.NODE_ENV === 'development' && (
        <div className="absolute top-4 right-4 bg-black bg-opacity-75 text-white text-xs p-3 rounded z-30 max-w-xs">
          <div>📄 PDF Pages: {numPages}</div>
          <div>🖥️ PDF Width: {pageWidth}px</div>
          <div>📍 Selected: {selectedText.length > 0 ? `"${selectedText.substring(0, 30)}..."` : 'None'}</div>
          <div className="text-green-400 mt-1">⚡ Enhanced Text Selection</div>
          <div className="text-xs opacity-70">
            • Adaptive delays for long selections
            <br />
            • Ctrl+A: Select all text
            <br />
            • Double-click: Select word
            <br />
            • Esc: Close menu
          </div>
        </div>
      )}
    </div>
  );
};

export default SimplePdfViewer;