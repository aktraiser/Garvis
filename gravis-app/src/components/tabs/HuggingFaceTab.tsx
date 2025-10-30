import React, { useState, useEffect } from 'react';
import { huggingFaceManager, type HuggingFaceModel, type PopularHFModel } from '../../lib/huggingface-manager';

export const HuggingFaceTab: React.FC = () => {
  const [isHFAvailable, setIsHFAvailable] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<HuggingFaceModel[]>([]);
  const [popularModels, setPopularModels] = useState<PopularHFModel[]>([]);
  const [localModels, setLocalModels] = useState<HuggingFaceModel[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [isSearching, setIsSearching] = useState(false);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState({ completed: 0, total: 0, status: '' });

  useEffect(() => {
    checkHFStatus();
    loadPopularModels();
    loadLocalModels();
  }, []);

  const checkHFStatus = async () => {
    const available = await huggingFaceManager.isAvailable();
    setIsHFAvailable(available);
  };

  const loadPopularModels = () => {
    const models = huggingFaceManager.getPopularModels();
    setPopularModels(models);
  };

  const loadLocalModels = async () => {
    const models = await huggingFaceManager.listLocalModels();
    setLocalModels(models);
  };

  const searchModels = async () => {
    if (!searchQuery.trim()) return;
    
    setIsSearching(true);
    try {
      const results = await huggingFaceManager.searchModels(searchQuery, 20);
      setSearchResults(results);
    } catch (error) {
      console.error('Error searching models:', error);
    } finally {
      setIsSearching(false);
    }
  };

  const downloadModel = async (modelId: string) => {
    setDownloadingModel(modelId);
    try {
      await huggingFaceManager.downloadModel(modelId, (progress) => {
        setDownloadProgress(progress);
      });
      await loadLocalModels();
    } catch (error) {
      console.error('Error downloading model:', error);
    } finally {
      setDownloadingModel(null);
      setDownloadProgress({ completed: 0, total: 0, status: '' });
    }
  };

  const deleteModel = async (modelId: string) => {
    try {
      await huggingFaceManager.deleteModel(modelId);
      await loadLocalModels();
    } catch (error) {
      console.error('Error deleting model:', error);
    }
  };

  const categories = huggingFaceManager.getCategories();
  const filteredPopularModels = selectedCategory === 'all' 
    ? popularModels 
    : popularModels.filter(model => model.category === selectedCategory);

  return (
    <div style={{ 
      maxWidth: '1200px',
      margin: '0 auto',
      pointerEvents: 'auto',
      position: 'relative',
      zIndex: 1
    }}>
      
      {!isHFAvailable ? (
        <div style={{ textAlign: 'center', padding: '60px 24px' }}>
          <div style={{ fontSize: '48px', marginBottom: '24px' }}>🤗</div>
          <h2 style={{ color: '#ffffff', marginBottom: '16px' }}>Hugging Face non disponible</h2>
          <p style={{ color: '#9ca3af', marginBottom: '24px' }}>
            Impossible de se connecter à l'API Hugging Face. Vérifiez votre connexion internet.
          </p>
          <button
            onClick={checkHFStatus}
            style={{
              padding: '12px 24px',
              backgroundColor: '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              fontWeight: '500'
            }}
          >
            🔄 Réessayer
          </button>
        </div>
      ) : (
        <>
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'space-between',
            marginBottom: '32px'
          }}>
            <div>
              <h2 style={{ 
                fontSize: '24px', 
                fontWeight: '600', 
                margin: 0,
                color: '#ffffff',
                marginBottom: '8px'
              }}>
                🤗 Modèles Hugging Face
              </h2>
              <p style={{ 
                color: '#9ca3af',
                margin: 0,
                fontSize: '14px'
              }}>
                Découvrez et téléchargez des modèles de la communauté
              </p>
            </div>
          </div>

          {/* Section de recherche */}
          <div style={{ marginBottom: '32px' }}>
            <div style={{ 
              display: 'flex', 
              gap: '12px',
              marginBottom: '24px'
            }}>
              <input
                type="text"
                placeholder="Rechercher un modèle..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && searchModels()}
                style={{
                  flex: 1,
                  padding: '12px 16px',
                  borderRadius: '8px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '14px'
                }}
              />
              <button
                onClick={searchModels}
                disabled={isSearching || !searchQuery.trim()}
                style={{
                  padding: '12px 24px',
                  backgroundColor: isSearching ? '#6b7280' : '#3b82f6',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  cursor: isSearching ? 'not-allowed' : 'pointer',
                  fontSize: '14px',
                  fontWeight: '500'
                }}
              >
                {isSearching ? '🔍 Recherche...' : '🔍 Rechercher'}
              </button>
            </div>

            {/* Résultats de recherche - Tableau */}
            {searchResults.length > 0 && (
              <div style={{ marginBottom: '32px' }}>
                <h3 style={{ 
                  color: '#ffffff',
                  fontSize: '18px',
                  marginBottom: '16px',
                  fontWeight: '500'
                }}>
                  📋 Résultats de recherche ({searchResults.length})
                </h3>
                <div style={{
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '12px',
                  overflow: 'hidden'
                }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                    <thead>
                      <tr style={{ backgroundColor: 'rgba(255, 255, 255, 0.1)' }}>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'left', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Modèle
                        </th>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'center', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Auteur
                        </th>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'center', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Type
                        </th>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'center', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Téléchargements
                        </th>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'center', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Likes
                        </th>
                        <th style={{ 
                          padding: '12px 16px', 
                          textAlign: 'center', 
                          color: '#ffffff', 
                          fontWeight: '600',
                          fontSize: '14px'
                        }}>
                          Action
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      {searchResults.map((model, index) => (
                        <tr key={index} style={{ 
                          borderTop: '1px solid rgba(255, 255, 255, 0.1)'
                        }}>
                          <td style={{ 
                            padding: '16px',
                            color: '#ffffff',
                            fontWeight: '500',
                            fontSize: '14px'
                          }}>
                            {model.name}
                          </td>
                          <td style={{ 
                            padding: '16px',
                            textAlign: 'center',
                            color: '#9ca3af',
                            fontSize: '14px'
                          }}>
                            {model.author}
                          </td>
                          <td style={{ 
                            padding: '16px',
                            textAlign: 'center'
                          }}>
                            <span style={{
                              fontSize: '12px',
                              color: '#6b7280',
                              backgroundColor: 'rgba(255, 255, 255, 0.1)',
                              padding: '4px 8px',
                              borderRadius: '4px'
                            }}>
                              {model.modelType}
                            </span>
                          </td>
                          <td style={{ 
                            padding: '16px',
                            textAlign: 'center',
                            color: '#9ca3af',
                            fontSize: '14px'
                          }}>
                            {model.downloads.toLocaleString()}
                          </td>
                          <td style={{ 
                            padding: '16px',
                            textAlign: 'center',
                            color: '#9ca3af',
                            fontSize: '14px'
                          }}>
                            {model.likes}
                          </td>
                          <td style={{ 
                            padding: '16px',
                            textAlign: 'center'
                          }}>
                            <button
                              onClick={() => downloadModel(model.id)}
                              disabled={downloadingModel === model.id}
                              style={{
                                padding: '6px 12px',
                                backgroundColor: downloadingModel === model.id ? '#6b7280' : '#3b82f6',
                                color: 'white',
                                border: 'none',
                                borderRadius: '6px',
                                fontSize: '12px',
                                fontWeight: '500',
                                cursor: downloadingModel === model.id ? 'not-allowed' : 'pointer'
                              }}
                            >
                              {downloadingModel === model.id ? '⏳ En cours...' : '📥 Télécharger'}
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>

          {/* Modèles populaires - Tableau */}
          <div style={{ marginBottom: '32px' }}>
            <div style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: '16px'
            }}>
              <h3 style={{ 
                color: '#ffffff',
                fontSize: '18px',
                margin: 0,
                fontWeight: '500'
              }}>
                ⭐ Modèles populaires
              </h3>
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                style={{
                  padding: '8px 12px',
                  borderRadius: '6px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '12px'
                }}
              >
                <option value="all">Toutes les catégories</option>
                {categories.map(category => (
                  <option key={category} value={category}>
                    {category}
                  </option>
                ))}
              </select>
            </div>
            
            <div style={{
              background: 'rgba(255, 255, 255, 0.05)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '12px',
              overflow: 'hidden'
            }}>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ backgroundColor: 'rgba(255, 255, 255, 0.1)' }}>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'left', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Modèle
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'left', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Description
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'center', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Auteur
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'center', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Taille
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'center', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Catégorie
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'center', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Tags
                    </th>
                    <th style={{ 
                      padding: '12px 16px', 
                      textAlign: 'center', 
                      color: '#ffffff', 
                      fontWeight: '600',
                      fontSize: '14px'
                    }}>
                      Action
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {filteredPopularModels.filter((model) => {
                    // Masquer les modèles déjà installés
                    return !localModels.some(m => m.id === model.id);
                  }).map((model, index) => {
                    const isDownloading = downloadingModel === model.id;
                    
                    return (
                      <tr key={index} style={{ 
                        borderTop: '1px solid rgba(255, 255, 255, 0.1)'
                      }}>
                        <td style={{ 
                          padding: '16px',
                          color: '#ffffff',
                          fontWeight: '500',
                          fontSize: '14px'
                        }}>
                          {model.name}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          color: '#9ca3af',
                          fontSize: '14px',
                          maxWidth: '250px'
                        }}>
                          {model.description}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center',
                          color: '#9ca3af',
                          fontSize: '14px'
                        }}>
                          {model.author}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <span style={{
                            fontSize: '12px',
                            color: '#6b7280',
                            backgroundColor: 'rgba(255, 255, 255, 0.1)',
                            padding: '4px 8px',
                            borderRadius: '4px'
                          }}>
                            {model.size}
                          </span>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <span style={{
                            fontSize: '12px',
                            color: '#6b7280',
                            backgroundColor: 'rgba(255, 255, 255, 0.1)',
                            padding: '4px 8px',
                            borderRadius: '4px'
                          }}>
                            {model.category}
                          </span>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <div style={{ 
                            display: 'flex',
                            gap: '4px',
                            flexWrap: 'wrap',
                            justifyContent: 'center'
                          }}>
                            {model.tags.slice(0, 2).map(tag => (
                              <span key={tag} style={{
                                fontSize: '10px',
                                padding: '2px 4px',
                                background: 'rgba(59, 130, 246, 0.2)',
                                color: '#60a5fa',
                                borderRadius: '4px'
                              }}>
                                {tag}
                              </span>
                            ))}
                            {model.tags.length > 2 && (
                              <span style={{
                                fontSize: '10px',
                                color: '#9ca3af'
                              }}>
                                +{model.tags.length - 2}
                              </span>
                            )}
                          </div>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          {isDownloading ? (
                            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '4px' }}>
                              <div style={{
                                background: 'rgba(59, 130, 246, 0.2)',
                                borderRadius: '4px',
                                height: '6px',
                                width: '60px',
                                overflow: 'hidden'
                              }}>
                                <div style={{
                                  background: '#3b82f6',
                                  height: '100%',
                                  width: downloadProgress.total > 0 
                                    ? `${(downloadProgress.completed / downloadProgress.total) * 100}%`
                                    : '0%',
                                  transition: 'width 0.3s'
                                }} />
                              </div>
                              <span style={{ fontSize: '10px', color: '#9ca3af' }}>
                                {downloadProgress.total > 0 
                                  ? `${Math.round((downloadProgress.completed / downloadProgress.total) * 100)}%`
                                  : '0%'
                                }
                              </span>
                            </div>
                          ) : (
                            <button
                              onClick={() => downloadModel(model.id)}
                              style={{
                                padding: '6px 12px',
                                backgroundColor: '#3b82f6',
                                color: 'white',
                                border: 'none',
                                borderRadius: '6px',
                                fontSize: '12px',
                                fontWeight: '500',
                                cursor: 'pointer'
                              }}
                            >
                              📥 Télécharger
                            </button>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>

          {/* Modèles installés - Tableau */}
          {localModels.length > 0 && (
            <div>
              <h3 style={{ 
                color: '#ffffff',
                fontSize: '18px',
                marginBottom: '16px',
                fontWeight: '500'
              }}>
                💾 Modèles installés ({localModels.length})
              </h3>
              <div style={{
                background: 'rgba(34, 197, 94, 0.1)',
                border: '1px solid rgba(34, 197, 94, 0.3)',
                borderRadius: '12px',
                overflow: 'hidden'
              }}>
                <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                  <thead>
                    <tr style={{ backgroundColor: 'rgba(34, 197, 94, 0.2)' }}>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'left', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Modèle
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Auteur
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Type
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Taille
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Action
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {localModels.map((model, index) => (
                      <tr key={index} style={{ 
                        borderTop: '1px solid rgba(34, 197, 94, 0.2)'
                      }}>
                        <td style={{ 
                          padding: '16px',
                          color: '#ffffff',
                          fontWeight: '500',
                          fontSize: '14px'
                        }}>
                          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                            <span style={{ color: '#10b981' }}>✅</span>
                            {model.name}
                          </div>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center',
                          color: '#9ca3af',
                          fontSize: '14px'
                        }}>
                          {model.author}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <span style={{
                            fontSize: '12px',
                            color: '#6b7280',
                            backgroundColor: 'rgba(255, 255, 255, 0.1)',
                            padding: '4px 8px',
                            borderRadius: '4px'
                          }}>
                            {model.modelType}
                          </span>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          {model.size && (
                            <span style={{
                              fontSize: '12px',
                              color: '#6b7280',
                              backgroundColor: 'rgba(255, 255, 255, 0.1)',
                              padding: '4px 8px',
                              borderRadius: '4px'
                            }}>
                              {model.size}
                            </span>
                          )}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <button
                            onClick={() => deleteModel(model.id)}
                            style={{
                              padding: '6px 12px',
                              backgroundColor: '#dc2626',
                              color: 'white',
                              border: 'none',
                              borderRadius: '6px',
                              fontSize: '12px',
                              fontWeight: '500',
                              cursor: 'pointer'
                            }}
                          >
                            🗑️ Supprimer
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};