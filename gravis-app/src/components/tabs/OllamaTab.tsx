import React, { useState, useEffect } from 'react';
import { ollamaManager, type OllamaModel, type AvailableOllamaModel } from '../../lib/ollama-manager';

export const OllamaTab: React.FC = () => {
  const [isOllamaAvailable, setIsOllamaAvailable] = useState(false);
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [availableModels, setAvailableModels] = useState<AvailableOllamaModel[]>([]);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<{ completed?: number; total?: number; status: string }>({ completed: 0, total: 0, status: '' });

  useEffect(() => {
    checkOllamaStatus();
    loadModels();
    loadAvailableModels();
  }, []);

  const loadAvailableModels = () => {
    const popularModels = ollamaManager.getPopularModels();
    setAvailableModels(popularModels);
  };

  const checkOllamaStatus = async () => {
    const available = await ollamaManager.isAvailable();
    setIsOllamaAvailable(available);
  };

  const loadModels = async () => {
    const modelList = await ollamaManager.listModels();
    setModels(modelList);
  };

  const downloadModel = async (modelName: string) => {
    setDownloadingModel(modelName);
    try {
      await ollamaManager.downloadModel(modelName, (progress) => {
        setDownloadProgress(progress);
      });
      await loadModels();
    } catch (error) {
      console.error('Error downloading model:', error);
    } finally {
      setDownloadingModel(null);
      setDownloadProgress({ completed: 0, total: 0, status: '' });
    }
  };

  const deleteModel = async (modelName: string) => {
    try {
      await ollamaManager.deleteModel(modelName);
      await loadModels();
    } catch (error) {
      console.error('Error deleting model:', error);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div style={{ 
      maxWidth: '1200px',
      margin: '0 auto',
      pointerEvents: 'auto',
      position: 'relative',
      zIndex: 1
    }}>
      
      {!isOllamaAvailable ? (
        <div style={{ textAlign: 'center', padding: '60px 24px' }}>
          <div style={{ fontSize: '48px', marginBottom: '24px' }}>🦙</div>
          <h2 style={{ color: '#ffffff', marginBottom: '16px' }}>Ollama non disponible</h2>
          <p style={{ color: '#9ca3af', marginBottom: '24px' }}>
            Veuillez installer et démarrer Ollama pour gérer les modèles locaux.
          </p>
          <a 
            href="https://ollama.ai"
            target="_blank"
            rel="noopener noreferrer"
            style={{
              display: 'inline-block',
              padding: '12px 24px',
              backgroundColor: '#3b82f6',
              color: 'white',
              textDecoration: 'none',
              borderRadius: '8px',
              fontWeight: '500'
            }}
          >
            Télécharger Ollama
          </a>
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
                color: '#ffffff'
              }}>
                Gestion des modèles Ollama
              </h2>
              <p style={{ 
                color: '#9ca3af', 
                margin: '8px 0 0 0',
                fontSize: '14px'
              }}>
                Téléchargez et gérez vos modèles d'IA locaux
              </p>
            </div>
          </div>

          {/* Modèles installés */}
          {models.length > 0 && (
            <div style={{ marginBottom: '40px' }}>
              <h3 style={{ 
                color: '#ffffff', 
                marginBottom: '16px',
                fontSize: '18px',
                fontWeight: '500'
              }}>
                Modèles installés ({models.length})
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
                        Taille
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Format
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Famille
                      </th>
                      <th style={{ 
                        padding: '12px 16px', 
                        textAlign: 'center', 
                        color: '#ffffff', 
                        fontWeight: '600',
                        fontSize: '14px'
                      }}>
                        Modifié
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
                    {models.map((model) => (
                      <tr key={model.name} style={{ 
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
                          textAlign: 'center'
                        }}>
                          <span style={{
                            fontSize: '12px',
                            color: '#6b7280',
                            backgroundColor: 'rgba(255, 255, 255, 0.1)',
                            padding: '4px 8px',
                            borderRadius: '4px'
                          }}>
                            {formatBytes(model.size)}
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
                            {model.details?.format || 'N/A'}
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
                            {model.details?.family || 'N/A'}
                          </span>
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center',
                          color: '#9ca3af',
                          fontSize: '12px'
                        }}>
                          {new Date(model.modified_at).toLocaleDateString('fr-FR')}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <button
                            onClick={() => deleteModel(model.name)}
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
                            Supprimer
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Modèles disponibles */}
          <div>
            <h3 style={{ 
              color: '#ffffff', 
              marginBottom: '16px',
              fontSize: '18px',
              fontWeight: '500'
            }}>
              Modèles disponibles au téléchargement
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
                      Statut
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
                  {availableModels.filter((model) => {
                    // Masquer les modèles déjà installés
                    return !models.some(m => m.name.includes(model.name));
                  }).map((model) => {
                    const isInstalled = models.some(m => m.name.includes(model.name));
                    const isDownloading = downloadingModel === model.name;
                    
                    return (
                      <tr key={model.name} style={{ 
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
                          maxWidth: '300px'
                        }}>
                          {model.description}
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
                          {isInstalled ? (
                            <span style={{
                              fontSize: '12px',
                              color: '#10b981',
                              fontWeight: '500'
                            }}>
                              ✅ Installé
                            </span>
                          ) : isDownloading ? (
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
                                  width: `${downloadProgress.total ? (downloadProgress.completed || 0) / downloadProgress.total * 100 : 0}%`,
                                  transition: 'width 0.3s'
                                }} />
                              </div>
                              <span style={{ fontSize: '10px', color: '#9ca3af' }}>
                                {downloadProgress.total && downloadProgress.completed 
                                  ? `${Math.round(downloadProgress.completed / downloadProgress.total * 100)}%`
                                  : '0%'
                                }
                              </span>
                            </div>
                          ) : (
                            <span style={{
                              fontSize: '12px',
                              color: '#9ca3af'
                            }}>
                              Non installé
                            </span>
                          )}
                        </td>
                        <td style={{ 
                          padding: '16px',
                          textAlign: 'center'
                        }}>
                          <button
                            onClick={() => downloadModel(model.name)}
                            disabled={isInstalled || isDownloading}
                            style={{
                              padding: '6px 12px',
                              backgroundColor: isInstalled ? '#374151' : '#3b82f6',
                              color: isInstalled ? '#9ca3af' : 'white',
                              border: 'none',
                              borderRadius: '6px',
                              fontSize: '12px',
                              fontWeight: '500',
                              cursor: isInstalled ? 'not-allowed' : 'pointer',
                              opacity: isInstalled ? 0.6 : 1
                            }}
                          >
                            {isInstalled ? 'Installé' : isDownloading ? 'En cours...' : 'Télécharger'}
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
};