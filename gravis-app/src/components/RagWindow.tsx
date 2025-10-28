import React, { useState, useEffect } from 'react';
import { ArrowLeft, Upload, Search, Settings, Filter, FileText, File, Database } from 'lucide-react';
import { RagStore, DocumentGroup } from '@/lib/rag';

interface RagWindowProps {
  onClose: () => void;
}

export const RagWindow: React.FC<RagWindowProps> = ({ onClose }) => {
  const [selectedGroup, setSelectedGroup] = useState<string>('');
  const [chunkSize, setChunkSize] = useState(512);
  const [overlap, setOverlap] = useState(64);
  const [strategy, setStrategy] = useState('AST-First');
  const [tags, setTags] = useState('');
  const [priority, setPriority] = useState('Normal');
  const [language, setLanguage] = useState('Auto-detect');
  const [groups, setGroups] = useState<DocumentGroup[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedDocument, setSelectedDocument] = useState<any>(null);

  // États OCR
  const [ocrLanguage, setOcrLanguage] = useState('fra+eng');
  const [psm, setPsm] = useState(6);
  const [oem, setOem] = useState(3);
  const [dpi, setDpi] = useState(300);
  const [forceOcr, setForceOcr] = useState(false);
  const [enhanceContrast, setEnhanceContrast] = useState(true);
  const [denoise, setDenoise] = useState(true);
  const [deskew, setDeskew] = useState(true);
  const [scaleFactor, setScaleFactor] = useState(2.0);

  // États classification
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [minOcrConfidence, setMinOcrConfidence] = useState(0.7);
  const [sourceTypeFilter, setSourceTypeFilter] = useState('all');

  // États recherche
  const [searchQuery, setSearchQuery] = useState('');
  const [searchGroup, setSearchGroup] = useState('');
  const [activeTab, setActiveTab] = useState<'documents' | 'groups'>('documents');

  // Load groups on mount
  useEffect(() => {
    const loadGroups = async () => {
      setIsLoading(true);
      try {
        await RagStore.loadGroups();
      } catch (error) {
        console.error('Error loading groups:', error);
      }
      setIsLoading(false);
    };

    loadGroups();

    // Subscribe to groups changes
    const unsubscribe = RagStore.subscribe((updatedGroups) => {
      setGroups(updatedGroups);
    });

    return unsubscribe;
  }, []);

  const createNewGroup = async () => {
    const name = prompt('Nom du nouveau groupe:');
    if (name) {
      try {
        setIsLoading(true);
        await RagStore.createGroup(name, {
          chunk_size: chunkSize,
          overlap: overlap,
          strategy: strategy as any
        });
      } catch (error) {
        console.error('Error creating group:', error);
        alert('Erreur lors de la création du groupe');
      }
      setIsLoading(false);
    }
  };

  const toggleGroup = async (groupId: string) => {
    try {
      setIsLoading(true);
      await RagStore.toggleGroup(groupId);
    } catch (error) {
      console.error('Error toggling group:', error);
      alert('Erreur lors de la modification du groupe');
    }
    setIsLoading(false);
  };

  const deleteGroup = async (groupId: string) => {
    if (confirm('Êtes-vous sûr de vouloir supprimer ce groupe ?')) {
      try {
        setIsLoading(true);
        await RagStore.deleteGroup(groupId);
      } catch (error) {
        console.error('Error deleting group:', error);
        alert('Erreur lors de la suppression du groupe');
      }
      setIsLoading(false);
    }
  };

  console.log('RagWindow rendering');

  // Mock documents for demonstration
  const mockDocuments = [
    { id: '1', name: 'rapport_financier.pdf', size: '4 MB', type: 'PDF', status: 'Ready', date: '12/11/2023 21:51', category: 'Business' },
    { id: '2', name: 'guide_technique.pdf', size: '2.3 MB', type: 'PDF', status: 'Processing', date: '12/11/2023 19:56', category: 'Technical' },
    { id: '3', name: 'contrat_legal.pdf', size: '1.8 MB', type: 'PDF', status: 'Ready', date: '12/11/2023 19:18', category: 'Legal' },
    { id: '4', name: 'research_paper.pdf', size: '5.2 MB', type: 'PDF', status: 'Ready', date: '12/11/2023 18:40', category: 'Academic' },
    { id: '5', name: 'mixed_document.pdf', size: '3.1 MB', type: 'PDF', status: 'Ready', date: '11/11/2023 19:28', category: 'Mixed' },
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
              className={`icon-button ${activeTab === 'groups' ? 'active' : ''}`}
              onClick={() => setActiveTab('groups')}
              style={{ 
                background: activeTab === 'groups' ? 'rgba(255,255,255,0.1)' : 'transparent',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '8px 12px',
                borderRadius: '8px'
              }}
            >
              <Database size={16} />
              Groupes
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
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Left Panel - Document List */}
        <div style={{ 
          width: '400px', 
          borderRight: '1px solid #333', 
          background: '#1a1a1a', 
          overflowY: 'auto' 
        }}>
          {activeTab === 'documents' ? (
            <div>
              {mockDocuments.map((doc) => (
                <div 
                  key={doc.id} 
                  className="conversation-item"
                  onClick={() => setSelectedDocument(doc)}
                  style={{ 
                    background: selectedDocument?.id === doc.id ? 'rgba(255,255,255,0.1)' : 'transparent',
                    borderBottom: '1px solid #333'
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: '500', marginBottom: '4px' }}>{doc.name}</div>
                    <div style={{ fontSize: '12px', color: '#999' }}>{doc.size} - {doc.category}</div>
                  </div>
                  <div style={{ fontSize: '12px', color: '#999' }}>{doc.date}</div>
                </div>
              ))}
            </div>
          ) : (
            <div>
              {groups.map((group) => (
                <div 
                  key={group.id} 
                  className="conversation-item"
                  onClick={() => setSelectedDocument(group)}
                  style={{ 
                    background: selectedDocument?.id === group.id ? 'rgba(255,255,255,0.1)' : 'transparent',
                    borderBottom: '1px solid #333'
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: '500', marginBottom: '4px' }}>{group.name}</div>
                    <div style={{ fontSize: '12px', color: '#999' }}>
                      {group.documents?.length || 0} documents - {group.active ? 'Actif' : 'Inactif'}
                    </div>
                  </div>
                  <div>
                    <button 
                      className="icon-button"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleGroup(group.id);
                      }}
                      style={{
                        padding: '4px 8px',
                        fontSize: '11px',
                        background: group.active ? '#22c55e' : 'none',
                        color: group.active ? 'white' : '#999',
                        border: `1px solid ${group.active ? '#22c55e' : '#333'}`
                      }}
                    >
                      {group.active ? 'ON' : 'OFF'}
                    </button>
                  </div>
                </div>
              ))}
              <button 
                className="conversation-item"
                onClick={createNewGroup}
                style={{ 
                  width: '100%',
                  color: '#0066cc',
                  justifyContent: 'flex-start',
                  borderBottom: '1px solid #333'
                }}
              >
                + Nouveau Groupe
              </button>
            </div>
          )}
        </div>

        {/* Right Panel - Details */}
        <div style={{ flex: 1, padding: '24px', overflowY: 'auto' }}>
          {selectedDocument ? (
            <div style={{ maxWidth: '600px' }}>
              <div style={{ marginBottom: '24px' }}>
                <span style={{ 
                  fontSize: '12px', 
                  color: '#999', 
                  textTransform: 'uppercase', 
                  letterSpacing: '0.5px' 
                }}>
                  FILE
                </span>
                <h2 style={{ 
                  fontSize: '24px', 
                  fontWeight: '600', 
                  margin: '8px 0 0 0' 
                }}>
                  {selectedDocument.name}
                </h2>
              </div>

              <div>
                <div className="conversation-item" style={{ 
                  background: 'rgba(255,255,255,0.05)',
                  marginBottom: '1px',
                  alignItems: 'center'
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <div style={{ 
                      width: '8px', 
                      height: '8px', 
                      borderRadius: '50%',
                      background: selectedDocument.status === 'Ready' ? '#22c55e' : '#f59e0b'
                    }}></div>
                    <span style={{ fontSize: '12px', color: '#999' }}>Status</span>
                  </div>
                  <span style={{ 
                    color: selectedDocument.status === 'Ready' ? '#22c55e' : '#f59e0b'
                  }}>
                    {selectedDocument.status === 'Ready' ? 'Ready' : 'Processing'}
                  </span>
                </div>

                <div className="conversation-item" style={{ 
                  background: 'rgba(255,255,255,0.05)',
                  marginBottom: '1px',
                  alignItems: 'center'
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <File size={16} color="#999" />
                    <span style={{ fontSize: '12px', color: '#999' }}>File ID</span>
                  </div>
                  <span>file-{selectedDocument.id}A45k8BzPTCuCciDlJJT51MP</span>
                </div>

                <div className="conversation-item" style={{ 
                  background: 'rgba(255,255,255,0.05)',
                  marginBottom: '1px',
                  alignItems: 'center'
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <Settings size={16} color="#999" />
                    <span style={{ fontSize: '12px', color: '#999' }}>Purpose</span>
                  </div>
                  <span>{selectedDocument.category || 'assistants'}</span>
                </div>

                <div className="conversation-item" style={{ 
                  background: 'rgba(255,255,255,0.05)',
                  marginBottom: '1px',
                  alignItems: 'center'
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <Database size={16} color="#999" />
                    <span style={{ fontSize: '12px', color: '#999' }}>Size</span>
                  </div>
                  <span>{selectedDocument.size}</span>
                </div>

                <div className="conversation-item" style={{ 
                  background: 'rgba(255,255,255,0.05)',
                  marginBottom: '1px',
                  alignItems: 'center'
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <FileText size={16} color="#999" />
                    <span style={{ fontSize: '12px', color: '#999' }}>Created at</span>
                  </div>
                  <span>{selectedDocument.date}</span>
                </div>

                {/* OCR Configuration */}
                <div style={{ 
                  margin: '24px 0', 
                  padding: '16px', 
                  background: 'rgba(255,255,255,0.05)', 
                  borderRadius: '8px' 
                }}>
                  <h3 style={{ margin: '0 0 16px 0', fontSize: '14px' }}>🔍 Configuration OCR</h3>
                  <div style={{ display: 'grid', gap: '12px' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                      <label style={{ fontSize: '12px', color: '#999' }}>Langue: {ocrLanguage}</label>
                      <select 
                        value={ocrLanguage} 
                        onChange={(e) => setOcrLanguage(e.target.value)}
                        className="search-input"
                        style={{ padding: '6px', fontSize: '14px' }}
                      >
                        <option value="fra+eng">Français + Anglais</option>
                        <option value="eng">Anglais</option>
                        <option value="fra">Français</option>
                      </select>
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                      <label style={{ fontSize: '12px', color: '#999' }}>DPI: {dpi}</label>
                      <input 
                        type="range" 
                        min="150" 
                        max="600" 
                        value={dpi} 
                        onChange={(e) => setDpi(Number(e.target.value))}
                        style={{ width: '100%' }}
                      />
                    </div>
                  </div>
                </div>

                {/* Classification */}
                <div style={{ 
                  margin: '24px 0', 
                  padding: '16px', 
                  background: 'rgba(255,255,255,0.05)', 
                  borderRadius: '8px' 
                }}>
                  <h3 style={{ margin: '0 0 16px 0', fontSize: '14px' }}>🏷️ Classification</h3>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                    {['Business', 'Academic', 'Legal', 'Technical', 'Mixed'].map(category => (
                      <button
                        key={category}
                        className="icon-button"
                        onClick={() => {
                          setSelectedCategories(prev => 
                            prev.includes(category)
                              ? prev.filter(c => c !== category)
                              : [...prev, category]
                          );
                        }}
                        style={{
                          padding: '6px 12px',
                          background: selectedCategories.includes(category) ? '#0066cc' : 'rgba(255,255,255,0.05)',
                          color: selectedCategories.includes(category) ? 'white' : '#ffffff',
                          fontSize: '12px',
                          border: selectedCategories.includes(category) ? '1px solid #0066cc' : '1px solid #444'
                        }}
                      >
                        {category}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              <div style={{ 
                marginTop: '24px', 
                paddingTop: '24px', 
                borderTop: '1px solid #333' 
              }}>
                <button 
                  className="icon-button"
                  style={{
                    width: '40px',
                    height: '40px',
                    background: '#dc2626',
                    borderRadius: '6px',
                    color: 'white',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center'
                  }}
                >
                  <span>🗑</span>
                </button>
              </div>
            </div>
          ) : (
            <div style={{ 
              display: 'flex', 
              flexDirection: 'column', 
              alignItems: 'center', 
              justifyContent: 'center', 
              height: '100%', 
              color: '#999', 
              textAlign: 'center' 
            }}>
              <FileText size={64} style={{ marginBottom: '16px', opacity: 0.5 }} />
              <h3 style={{ margin: '0 0 8px 0', color: '#ffffff' }}>Sélectionner un document</h3>
              <p style={{ margin: 0, fontSize: '14px' }}>Choisissez un document ou un groupe pour voir les détails</p>
            </div>
          )}
        </div>
      </div>

    </div>
  );
};