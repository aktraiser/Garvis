import React, { useState, useEffect } from 'react';
import { ArrowLeft, Upload, Search, Filter, FileText, Database, Download, Trash2, Eye, PlayCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface RagWindowProps {
  onClose: () => void;
}

// Types pour les résultats des commandes Tauri
interface DocumentIngestionResponse {
  document_id: string;
  document_category: string;
  chunks_created: number;
  extraction_method: string;
  source_type: string;
  processing_time_ms: number;
  confidence_score?: number;
  business_metadata?: any;
  cache_stats?: any;
}

interface OcrProcessResponse {
  text: string;
  confidence: number;
  language: string;
  processing_time_ms: number;
}

interface DocumentInfo {
  id: string;
  name: string;
  size: string;
  sizeBytes: number;
  type: string;
  status: string;
  date: string;
  category: string;
  pages: number;
  extracted: boolean;
  extractedAt: string;
  confidence: number;
}

export const RagWindow: React.FC<RagWindowProps> = ({ onClose }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'documents' | 'injection'>('documents');
  const [isExtracting, setIsExtracting] = useState<Record<string, boolean>>({});
  const [isInjecting, setIsInjecting] = useState<Record<string, boolean>>({});
  const [extractionResults, setExtractionResults] = useState<Record<string, any>>({});
  const [documents, setDocuments] = useState<DocumentInfo[]>([]);
  const [isLoadingDocuments, setIsLoadingDocuments] = useState(true);
  // Variables inutilisées mais gardées pour éviter les erreurs
  const [_isClearing, _setIsClearing] = useState(false);
  const [_showUploadDialog, _setShowUploadDialog] = useState(false);
  const [_selectedFile, _setSelectedFile] = useState<File | null>(null);
  const [_uploadParams, _setUploadParams] = useState({
    category: 'Mixed',
    forceOcr: false,
    language: 'fra+eng'
  });

  // Charger les documents dynamiquement depuis le dossier exemple
  const loadDocuments = async () => {
    setIsLoadingDocuments(true);
    try {
      // Créer une commande Tauri pour lister les fichiers du dossier exemple
      // const documentsPath = '/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple';
      
      // Pour l'instant, on peut utiliser une approche manuelle
      // TODO: Créer une commande Tauri list_documents() pour lister automatiquement
      const exampleFiles = [
        'unilever-annual-report-and-accounts-2024.pdf',
        '2510.18234v1.pdf', 
        'contrôle technique.pdf',
        'PV_AGE_XME_20octobre2025.pdf',
        'IMG_20251007_0001.pdf',
        '7fd558c8d29c99e999e2b6708de21b6b65cbc79de443f9bdd976eb38d8a611f9.png'
      ];
      
      const documentList: DocumentInfo[] = exampleFiles.map((filename, index) => {
        const isImage = filename.toLowerCase().endsWith('.png') || filename.toLowerCase().endsWith('.jpg');
        const isPdf = filename.toLowerCase().endsWith('.pdf');
        
        // Catégorisation automatique basée sur le nom du fichier
        let category = 'Mixed';
        if (filename.includes('unilever') || filename.includes('PV_AGE')) category = 'Business';
        else if (filename.includes('2510') || filename.includes('research')) category = 'Academic';
        else if (filename.includes('contrôle') || filename.includes('technique')) category = 'Legal';
        else if (isImage) category = 'Technical';
        
        return {
          id: (index + 1).toString(),
          name: filename,
          size: '? MB', // TODO: Récupérer la vraie taille
          sizeBytes: 1000000, // Taille par défaut
          type: isImage ? 'Image' : isPdf ? 'PDF' : 'Unknown',
          status: 'Ready',
          date: new Date().toLocaleDateString('fr-FR'),
          category,
          pages: isImage ? 1 : 10, // Estimation
          extracted: false,
          extractedAt: '',
          confidence: 0
        };
      });
      
      setDocuments(documentList);
    } catch (error) {
      console.error('Failed to load documents:', error);
    } finally {
      setIsLoadingDocuments(false);
    }
  };

  // Charger les documents au montage du composant
  useEffect(() => {
    loadDocuments();
  }, []);

  // Documents affichés (dynamiques)
  const displayedDocuments = documents;

  // Créer un groupe par défaut si nécessaire
  const ensureDefaultGroup = async () => {
    try {
      // Tenter de créer le groupe par défaut
      await invoke('rag_create_group', { name: 'Documents Exemple' });
      console.log('Default group created');
    } catch (error) {
      // Le groupe existe déjà, c'est OK
      console.log('Default group already exists or creation failed:', error);
    }
  };

  // Fonction d'extraction OCR pour un document
  const handleExtractOCR = async (docId: string, docName: string) => {
    setIsExtracting(prev => ({ ...prev, [docId]: true }));
    
    try {
      // S'assurer que le groupe par défaut existe
      await ensureDefaultGroup();
      
      // Construire le chemin du fichier depuis le dossier exemple
      const filePath = `/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/${docName}`;
      
      console.log(`Extracting OCR for: ${filePath}`);
      
      // Appeler la commande OCR selon le type de fichier
      let result: DocumentIngestionResponse | OcrProcessResponse;
      if (docName.toLowerCase().endsWith('.pdf')) {
        // Pour PDF, utiliser le pipeline intelligent
        result = await invoke<DocumentIngestionResponse>('add_document_intelligent', {
          filePath,
          groupId: 'default_group', // Groupe par défaut
          forceOcr: true
        });
        
        alert(`Extraction OCR terminée!\nChunks créés: ${result.chunks_created}\nCatégorie: ${result.document_category}\nConfiance: ${result.confidence_score?.toFixed(2) || 'N/A'}\nTemps: ${result.processing_time_ms}ms`);
      } else {
        // Pour images, utiliser OCR direct
        result = await invoke<OcrProcessResponse>('ocr_process_image', {
          imagePath: filePath,
          language: 'fra+eng'
        });
        
        alert(`Extraction OCR terminée!\nTexte extrait: ${result.text?.length || 0} caractères\nConfiance: ${result.confidence?.toFixed(2) || 'N/A'}`);
      }
      
      console.log('OCR extraction result:', result);
      
      // Stocker le résultat
      setExtractionResults(prev => ({
        ...prev,
        [docId]: result
      }));
      
      // Marquer comme extrait dans les données dynamiques
      setDocuments(prevDocs => 
        prevDocs.map(doc => 
          doc.id === docId 
            ? {
                ...doc,
                extracted: true,
                extractedAt: new Date().toLocaleString(),
                confidence: (() => {
                  // Calculer la confiance selon le type de résultat
                  if ('confidence_score' in result && result.confidence_score) {
                    return result.confidence_score * 100;
                  } else if ('confidence' in result) {
                    return result.confidence * 100;
                  } else {
                    return 95.0;
                  }
                })()
              }
            : doc
        )
      );
      
    } catch (error) {
      console.error('OCR extraction failed:', error);
      alert(`Erreur d'extraction OCR: ${error}`);
    } finally {
      setIsExtracting(prev => ({ ...prev, [docId]: false }));
    }
  };

  // Fonction d'injection RAG pour un document
  const handleInjectRAG = async (docId: string, docName: string) => {
    setIsInjecting(prev => ({ ...prev, [docId]: true }));
    
    try {
      // S'assurer que le groupe par défaut existe
      await ensureDefaultGroup();
      
      const filePath = `/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/exemple/${docName}`;
      
      console.log(`Injecting document to RAG: ${filePath}`);
      
      // Utiliser la commande d'ingestion intelligente (Phase 3)
      const result = await invoke<DocumentIngestionResponse>('add_document_intelligent', {
        filePath,
        groupId: 'default_group',
        forceOcr: false // Laisser le système décider
      });
      
      console.log('RAG injection result:', result);
      
      // Stocker le résultat
      setExtractionResults(prev => ({
        ...prev,
        [`rag_${docId}`]: result
      }));
      
      alert(`Document injecté avec succès!\nChunks créés: ${result.chunks_created}\nCatégorie: ${result.document_category}\nMéthode extraction: ${result.extraction_method}\nSource: ${result.source_type}\nConfiance: ${result.confidence_score?.toFixed(2) || 'N/A'}\nTemps: ${result.processing_time_ms}ms`);
      
    } catch (error) {
      console.error('RAG injection failed:', error);
      alert(`Erreur d'injection RAG: ${error}`);
    } finally {
      setIsInjecting(prev => ({ ...prev, [docId]: false }));
    }
  };

  return (
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
      {/* Header */}
      <div className="search-container" style={{ borderBottom: '1px solid #333', paddingBottom: '16px' }}>
        <div className="search-input-wrapper" style={{ justifyContent: 'space-between', padding: '12px 16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <button onClick={onClose} className="icon-button">
              <ArrowLeft size={20} />
            </button>
            <h1 style={{ fontSize: '18px', fontWeight: '600', margin: 0 }}>Storage RAG</h1>
          </div>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button 
              className={`icon-button ${activeTab === 'documents' ? 'active' : ''}`}
              onClick={() => setActiveTab('documents')}
              style={{ 
                background: activeTab === 'documents' ? 'rgba(255,255,255,0.1)' : 'transparent',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '8px 12px',
                borderRadius: '8px'
              }}
            >
              <FileText size={16} />
              Documents
            </button>
            <button 
              className={`icon-button ${activeTab === 'injection' ? 'active' : ''}`}
              onClick={() => setActiveTab('injection')}
              style={{ 
                background: activeTab === 'injection' ? 'rgba(255,255,255,0.1)' : 'transparent',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '8px 12px',
                borderRadius: '8px'
              }}
            >
              <Database size={16} />
              Injection
            </button>
          </div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <button className="icon-button" style={{ padding: '8px 12px' }}>Learn more</button>
            <button className="icon-button" style={{ 
              background: '#0066cc', 
              padding: '8px 12px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}>
              <Upload size={16} />
              Upload
            </button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: 'hidden', padding: '24px' }}>
        {activeTab === 'documents' ? (
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
            
            {/* Main content - only show when not loading */}
            {!isLoadingDocuments && (
              <>
                {/* Search bar */}
                <div style={{ 
              marginBottom: '20px', 
              display: 'flex', 
              gap: '12px', 
              alignItems: 'center' 
            }}>
              <div style={{ 
                flex: 1, 
                position: 'relative',
                background: 'rgba(255, 255, 255, 0.06)',
                border: '1px solid rgba(255, 255, 255, 0.12)',
                borderRadius: '8px',
                padding: '8px 12px',
                display: 'flex',
                alignItems: 'center',
                gap: '8px'
              }}>
                <Search size={16} color="#999" />
                <input
                  type="text"
                  placeholder="Rechercher des documents..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    outline: 'none',
                    color: '#ffffff',
                    flex: 1,
                    fontSize: '14px'
                  }}
                />
              </div>
              <button className="icon-button">
                <Filter size={16} />
              </button>
            </div>

            {/* Documents table */}
            <div style={{ 
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '12px',
              overflow: 'hidden'
            }}>
              {/* Table header */}
              <div style={{
                display: 'grid',
                gridTemplateColumns: '3fr 1fr 1fr 1fr 1fr 1fr 120px',
                gap: '16px',
                padding: '16px 20px',
                background: 'rgba(255, 255, 255, 0.05)',
                borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
                fontSize: '12px',
                fontWeight: '600',
                color: '#999',
                textTransform: 'uppercase',
                letterSpacing: '0.5px'
              }}>
                <div>Document</div>
                <div>Type</div>
                <div>Taille</div>
                <div>Pages</div>
                <div>Statut</div>
                <div>Date</div>
                <div>Actions</div>
              </div>

              {/* Table rows */}
              {displayedDocuments
                .filter((doc: DocumentInfo) => 
                  searchQuery === '' || 
                  doc.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                  doc.category.toLowerCase().includes(searchQuery.toLowerCase())
                )
                .map((doc) => (
                <div 
                  key={doc.id}
                  style={{
                    display: 'grid',
                    gridTemplateColumns: '3fr 1fr 1fr 1fr 1fr 1fr 120px',
                    gap: '16px',
                    padding: '16px 20px',
                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                    transition: 'background-color 0.2s ease',
                    cursor: 'pointer'
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = 'rgba(255, 255, 255, 0.03)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'transparent';
                  }}
                >
                  {/* Document name and info */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <div style={{
                      width: '32px',
                      height: '32px',
                      background: 'rgba(239, 68, 68, 0.1)',
                      border: '1px solid rgba(239, 68, 68, 0.3)',
                      borderRadius: '6px',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      color: '#ef4444'
                    }}>
                      <FileText size={16} />
                    </div>
                    <div>
                      <div style={{ fontWeight: '500', color: '#ffffff', marginBottom: '2px' }}>
                        {doc.name}
                      </div>
                      <div style={{ fontSize: '12px', color: '#999' }}>
                        {doc.category}
                      </div>
                    </div>
                  </div>

                  {/* Type */}
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center',
                    color: '#e0e0e0',
                    fontSize: '13px'
                  }}>
                    {doc.type}
                  </div>

                  {/* Size */}
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center',
                    color: '#e0e0e0',
                    fontSize: '13px'
                  }}>
                    {doc.size}
                  </div>

                  {/* Pages */}
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center',
                    color: '#e0e0e0',
                    fontSize: '13px'
                  }}>
                    {doc.pages}
                  </div>

                  {/* Status */}
                  <div style={{ display: 'flex', alignItems: 'center' }}>
                    <div style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '4px 8px',
                      borderRadius: '12px',
                      fontSize: '11px',
                      fontWeight: '500',
                      background: doc.status === 'Ready' 
                        ? 'rgba(34, 197, 94, 0.1)' 
                        : 'rgba(245, 158, 11, 0.1)',
                      color: doc.status === 'Ready' ? '#22c55e' : '#f59e0b',
                      border: `1px solid ${doc.status === 'Ready' ? 'rgba(34, 197, 94, 0.3)' : 'rgba(245, 158, 11, 0.3)'}`
                    }}>
                      <div style={{
                        width: '6px',
                        height: '6px',
                        borderRadius: '50%',
                        background: doc.status === 'Ready' ? '#22c55e' : '#f59e0b'
                      }}></div>
                      {doc.status}
                    </div>
                  </div>

                  {/* Date */}
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center',
                    color: '#999',
                    fontSize: '12px'
                  }}>
                    {doc.date.split(' ')[0]}
                  </div>

                  {/* Actions */}
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center', 
                    gap: '8px' 
                  }}>
                    {doc.status === 'Ready' && !doc.extracted && (
                      <button
                        className="icon-button"
                        title="Extraire le contenu"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleExtractOCR(doc.id, doc.name);
                        }}
                        disabled={isExtracting[doc.id]}
                        style={{
                          width: '28px',
                          height: '28px',
                          background: 'rgba(34, 197, 94, 0.1)',
                          border: '1px solid rgba(34, 197, 94, 0.3)',
                          color: isExtracting[doc.id] ? '#999' : '#22c55e',
                          cursor: isExtracting[doc.id] ? 'not-allowed' : 'pointer',
                          opacity: isExtracting[doc.id] ? 0.5 : 1
                        }}
                      >
                        {isExtracting[doc.id] ? '...' : <Download size={12} />}
                      </button>
                    )}
                    {doc.extracted && (
                      <div style={{
                        padding: '4px 6px',
                        background: 'rgba(34, 197, 94, 0.1)',
                        border: '1px solid rgba(34, 197, 94, 0.3)',
                        borderRadius: '4px',
                        fontSize: '10px',
                        color: '#22c55e',
                        fontWeight: '500'
                      }}>
                        ✓ Extrait
                      </div>
                    )}
                    
                    {/* Bouton Injection RAG */}
                    <button
                      className="icon-button"
                      title="Injecter dans RAG"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleInjectRAG(doc.id, doc.name);
                      }}
                      disabled={isInjecting[doc.id]}
                      style={{
                        width: '28px',
                        height: '28px',
                        background: 'rgba(168, 85, 247, 0.1)',
                        border: '1px solid rgba(168, 85, 247, 0.3)',
                        color: isInjecting[doc.id] ? '#999' : '#a855f7',
                        cursor: isInjecting[doc.id] ? 'not-allowed' : 'pointer',
                        opacity: isInjecting[doc.id] ? 0.5 : 1
                      }}
                    >
                      {isInjecting[doc.id] ? '...' : <PlayCircle size={12} />}
                    </button>
                    
                    <button
                      className="icon-button"
                      title="Voir les détails"
                      onClick={(e) => {
                        e.stopPropagation();
                        const result = extractionResults[doc.id] || extractionResults[`rag_${doc.id}`];
                        if (result) {
                          alert(`Résultats:\n${JSON.stringify(result, null, 2)}`);
                        } else {
                          alert('Aucun résultat disponible. Effectuez d\'abord une extraction ou injection.');
                        }
                      }}
                      style={{
                        width: '28px',
                        height: '28px',
                        background: 'rgba(59, 130, 246, 0.1)',
                        border: '1px solid rgba(59, 130, 246, 0.3)',
                        color: '#3b82f6'
                      }}
                    >
                      <Eye size={12} />
                    </button>
                    <button
                      className="icon-button"
                      title="Supprimer"
                      onClick={(e) => {
                        e.stopPropagation();
                        console.log('Delete:', doc.id);
                      }}
                      style={{
                        width: '28px',
                        height: '28px',
                        background: 'rgba(239, 68, 68, 0.1)',
                        border: '1px solid rgba(239, 68, 68, 0.3)',
                        color: '#ef4444'
                      }}
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>
                </div>
              ))}
            </div>

            {/* Statistics */}
            <div style={{ 
              marginTop: '20px',
              display: 'grid',
              gridTemplateColumns: 'repeat(4, 1fr)',
              gap: '16px'
            }}>
              <div style={{
                padding: '16px',
                background: 'rgba(255, 255, 255, 0.03)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '8px'
              }}>
                <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>
                  Total Documents
                </div>
                <div style={{ fontSize: '24px', fontWeight: '600', color: '#ffffff' }}>
                  {displayedDocuments.length}
                </div>
              </div>
              <div style={{
                padding: '16px',
                background: 'rgba(255, 255, 255, 0.03)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '8px'
              }}>
                <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>
                  Extraits
                </div>
                <div style={{ fontSize: '24px', fontWeight: '600', color: '#22c55e' }}>
                  {displayedDocuments.filter(d => d.extracted).length}
                </div>
              </div>
              <div style={{
                padding: '16px',
                background: 'rgba(255, 255, 255, 0.03)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '8px'
              }}>
                <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>
                  Taille Totale
                </div>
                <div style={{ fontSize: '24px', fontWeight: '600', color: '#ffffff' }}>
                  {(displayedDocuments.reduce((acc, doc) => acc + doc.sizeBytes, 0) / 1024 / 1024).toFixed(1)} MB
                </div>
              </div>
              <div style={{
                padding: '16px',
                background: 'rgba(255, 255, 255, 0.03)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '8px'
              }}>
                <div style={{ fontSize: '12px', color: '#999', marginBottom: '4px' }}>
                  Confiance Moyenne
                </div>
                <div style={{ fontSize: '24px', fontWeight: '600', color: '#3b82f6' }}>
                  {displayedDocuments.filter(d => d.extracted).length > 0 
                    ? (displayedDocuments.filter(d => d.extracted).reduce((acc, doc) => acc + doc.confidence, 0) / displayedDocuments.filter(d => d.extracted).length).toFixed(1)
                    : '0.0'
                  }%
                </div>
              </div>
            </div>
              </>
            )}
          </div>
        ) : (
          <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            textAlign: 'center'
          }}>
            <Database size={64} style={{ marginBottom: '16px', opacity: 0.3, color: '#999' }} />
            <h3 style={{ margin: '0 0 8px 0', color: '#ffffff', fontSize: '18px' }}>
              Injection de Documents
            </h3>
            <p style={{ margin: '0 0 24px 0', fontSize: '14px', color: '#999', maxWidth: '400px' }}>
              Cette fonctionnalité permet d'injecter le contenu des documents extraits dans votre base de connaissances pour les requêtes RAG.
            </p>
            <div style={{
              padding: '20px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '12px',
              maxWidth: '500px'
            }}>
              <h4 style={{ margin: '0 0 12px 0', color: '#ffffff', fontSize: '16px' }}>
                Fonctionnalité à venir
              </h4>
              <p style={{ margin: 0, fontSize: '13px', color: '#999', lineHeight: '1.5' }}>
                L'onglet Injection permettra de configurer et gérer l'injection des documents extraits dans votre système RAG. 
                Cette fonctionnalité sera disponible prochainement.
              </p>
            </div>
          </div>
        )}
      </div>

    </div>
  );
};