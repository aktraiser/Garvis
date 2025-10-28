import React, { useState } from 'react';
import { ArrowLeft, Upload, Search, Filter, FileText, Database, Download, Trash2, Eye } from 'lucide-react';

interface RagWindowProps {
  onClose: () => void;
}

export const RagWindow: React.FC<RagWindowProps> = ({ onClose }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'documents' | 'injection'>('documents');


  // Mock documents for demonstration
  const mockDocuments = [
    { 
      id: '1', 
      name: 'rapport_financier.pdf', 
      size: '4.2 MB', 
      sizeBytes: 4404019,
      type: 'PDF', 
      status: 'Ready', 
      date: '12/11/2023 21:51', 
      category: 'Business',
      pages: 35,
      extracted: true,
      extractedAt: '12/11/2023 21:52',
      confidence: 95.3
    },
    { 
      id: '2', 
      name: 'guide_technique.pdf', 
      size: '2.3 MB', 
      sizeBytes: 2411724,
      type: 'PDF', 
      status: 'Processing', 
      date: '12/11/2023 19:56', 
      category: 'Technical',
      pages: 18,
      extracted: false,
      confidence: 0
    },
    { 
      id: '3', 
      name: 'contrat_legal.pdf', 
      size: '1.8 MB', 
      sizeBytes: 1887436,
      type: 'PDF', 
      status: 'Ready', 
      date: '12/11/2023 19:18', 
      category: 'Legal',
      pages: 12,
      extracted: true,
      extractedAt: '12/11/2023 19:19',
      confidence: 98.7
    },
    { 
      id: '4', 
      name: 'research_paper.pdf', 
      size: '5.2 MB', 
      sizeBytes: 5452595,
      type: 'PDF', 
      status: 'Ready', 
      date: '12/11/2023 18:40', 
      category: 'Academic',
      pages: 42,
      extracted: false,
      confidence: 0
    },
    { 
      id: '5', 
      name: 'mixed_document.pdf', 
      size: '3.1 MB', 
      sizeBytes: 3251712,
      type: 'PDF', 
      status: 'Ready', 
      date: '11/11/2023 19:28', 
      category: 'Mixed',
      pages: 28,
      extracted: true,
      extractedAt: '11/11/2023 19:30',
      confidence: 87.2
    },
  ];

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
              {mockDocuments
                .filter(doc => 
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
                          console.log('Extract:', doc.id);
                        }}
                        style={{
                          width: '28px',
                          height: '28px',
                          background: 'rgba(34, 197, 94, 0.1)',
                          border: '1px solid rgba(34, 197, 94, 0.3)',
                          color: '#22c55e'
                        }}
                      >
                        <Download size={12} />
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
                    <button
                      className="icon-button"
                      title="Voir les détails"
                      onClick={(e) => {
                        e.stopPropagation();
                        console.log('View details:', doc.id);
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
                  {mockDocuments.length}
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
                  {mockDocuments.filter(d => d.extracted).length}
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
                  {(mockDocuments.reduce((acc, doc) => acc + doc.sizeBytes, 0) / 1024 / 1024).toFixed(1)} MB
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
                  {(mockDocuments.filter(d => d.extracted).reduce((acc, doc) => acc + doc.confidence, 0) / mockDocuments.filter(d => d.extracted).length).toFixed(1)}%
                </div>
              </div>
            </div>
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