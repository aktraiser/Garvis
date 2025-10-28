import React, { useState, useEffect } from 'react';
import { Loader2, CheckCircle, XCircle, Bot, RefreshCw } from 'lucide-react';
import { LiteLLMClient, modelConfigStore, AVAILABLE_MODELS } from '@/lib/litellm';

interface ModelSelectorWindowProps {
  onClose: () => void;
}

export const ModelSelectorWindow: React.FC<ModelSelectorWindowProps> = ({ onClose }) => {
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedModel, setSelectedModel] = useState(modelConfigStore.currentModel.id);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    setIsLoading(true);
    setError('');

    try {
      const config = modelConfigStore.getConfig();
      
      if (!config.apiKey && !config.baseUrl) {
        setError('Configuration manquante : veuillez d\'abord configurer votre API');
        setAvailableModels([]);
        return;
      }

      const client = new LiteLLMClient(config);
      const result = await client.getModels();
      
      if (result.data && Array.isArray(result.data)) {
        setAvailableModels(result.data);
      } else {
        // Fallback vers les modèles par défaut
        setAvailableModels([]);
        setError('Impossible de récupérer les modèles du serveur');
      }
    } catch (err) {
      console.error('Error loading models:', err);
      setError(err instanceof Error ? err.message : 'Erreur de connexion');
      setAvailableModels([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleModelSelect = (modelId: string) => {
    setSelectedModel(modelId);
  };

  const handleSave = () => {
    // Chercher le modèle d'abord dans les modèles du serveur, puis dans notre liste locale
    let foundModel = availableModels.find(m => m.id === selectedModel) || 
                     AVAILABLE_MODELS.find(m => m.id === selectedModel) ||
                     modelConfigStore.currentModel;
    
    // Si le modèle du serveur n'a pas de 'name', utiliser l'id
    if (foundModel && !foundModel.name) {
      foundModel = {
        ...foundModel,
        name: foundModel.id
      };
    }
    
    console.log('Saving model:', foundModel);
    modelConfigStore.setModel(foundModel);
    
    // Close window after save
    setTimeout(() => onClose(), 500);
  };

  const currentModels = availableModels.length > 0 ? availableModels : AVAILABLE_MODELS;

  console.log('ModelSelectorWindow rendering');

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
      <div style={{ 
        flex: 1, 
        display: 'flex', 
        flexDirection: 'column' 
      }}>
        

        {/* Content */}
        <div style={{ 
          flex: 1, 
          padding: '24px',
          overflow: 'auto',
          background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f1629 100%)'
        }}>
          {isLoading ? (
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center', 
              padding: '64px 0' 
            }}>
              <div style={{ textAlign: 'center' }}>
                <Loader2 size={32} style={{ 
                  animation: 'spin 1s linear infinite', 
                  margin: '0 auto 16px', 
                  color: '#3b82f6' 
                }} />
                <p style={{ fontSize: '18px', color: '#d1d5db', marginBottom: '8px' }}>
                  Chargement des modèles...
                </p>
                <p style={{ fontSize: '14px', color: '#6b7280' }}>
                  Connexion au serveur LiteLLM
                </p>
              </div>
            </div>
          ) : error ? (
            <div style={{ textAlign: 'center', padding: '64px 0' }}>
              <XCircle size={48} style={{ 
                margin: '0 auto 16px', 
                color: '#ef4444' 
              }} />
              <p style={{ fontSize: '18px', color: '#ef4444', marginBottom: '8px' }}>
                Erreur de connexion
              </p>
              <p style={{ fontSize: '14px', color: '#9ca3af', marginBottom: '24px' }}>
                {error}
              </p>
              <button 
                onClick={loadModels} 
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  margin: '0 auto',
                  padding: '8px 16px',
                  background: '#3b82f6',
                  color: '#ffffff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  transition: 'background-color 0.2s'
                }}
                onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#2563eb'}
                onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#3b82f6'}
              >
                <RefreshCw size={16} />
                Réessayer
              </button>
            </div>
          ) : (
            <div style={{ 
              display: 'grid', 
              gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))',
              gap: '32px',
              maxWidth: '1200px',
              margin: '0 auto',
              alignItems: 'start'
            }}>
              
              {/* Left Panel - Models List */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
                <div style={{
                  background: 'rgba(31, 41, 55, 0.5)',
                  backdropFilter: 'blur(12px)',
                  border: '1px solid #374151',
                  borderRadius: '12px',
                  padding: '24px'
                }}>
                  <div style={{ 
                    display: 'flex', 
                    alignItems: 'center', 
                    justifyContent: 'space-between',
                    marginBottom: '16px'
                  }}>
                    <h2 style={{ 
                      fontSize: '18px', 
                      fontWeight: '600', 
                      color: '#ffffff', 
                      margin: 0,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px'
                    }}>
                      <Bot size={18} />
                      Modèles Disponibles
                      <span style={{
                        marginLeft: '8px',
                        padding: '4px 8px',
                        background: '#3b82f6',
                        color: '#ffffff',
                        fontSize: '12px',
                        borderRadius: '999px'
                      }}>
                        {currentModels.length}
                      </span>
                    </h2>
                    <button 
                      onClick={loadModels}
                      disabled={isLoading}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        padding: '6px 10px',
                        background: '#4b5563',
                        color: '#ffffff',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: isLoading ? 'not-allowed' : 'pointer',
                        transition: 'background-color 0.2s',
                        opacity: isLoading ? 0.5 : 1,
                        fontSize: '12px'
                      }}
                      onMouseEnter={(e) => {
                        if (!isLoading) e.currentTarget.style.backgroundColor = '#6b7280';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.backgroundColor = '#4b5563';
                      }}
                    >
                      <RefreshCw size={12} style={{ animation: isLoading ? 'spin 1s linear infinite' : 'none' }} />
                      <span>Actualiser</span>
                    </button>
                  </div>
                  
                  <div style={{ 
                    display: 'flex', 
                    flexDirection: 'column', 
                    gap: '8px', 
                    maxHeight: '250px', 
                    overflowY: 'auto' 
                  }}>
                    {currentModels.map((model) => (
                      <div
                        key={model.id}
                        style={{
                          padding: '12px',
                          borderRadius: '8px',
                          border: `1px solid ${selectedModel === model.id ? '#8b5cf6' : '#4b5563'}`,
                          background: selectedModel === model.id ? 'rgba(139, 92, 246, 0.1)' : 'rgba(55, 65, 81, 0.5)',
                          transition: 'all 0.2s',
                          cursor: 'pointer'
                        }}
                        onClick={() => handleModelSelect(model.id)}
                        onMouseEnter={(e) => {
                          if (selectedModel !== model.id) {
                            e.currentTarget.style.borderColor = '#6b7280';
                          }
                        }}
                        onMouseLeave={(e) => {
                          if (selectedModel !== model.id) {
                            e.currentTarget.style.borderColor = '#4b5563';
                          }
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                          <div style={{ flex: 1 }}>
                            <div style={{ 
                              display: 'flex', 
                              alignItems: 'center', 
                              gap: '8px',
                              fontWeight: '500', 
                              color: '#ffffff', 
                              fontSize: '14px' 
                            }}>
                              {model.id}
                              {model.id === modelConfigStore.currentModel.id && (
                                <span style={{
                                  padding: '2px 6px',
                                  background: '#16a34a',
                                  color: '#ffffff',
                                  fontSize: '10px',
                                  borderRadius: '4px',
                                  fontWeight: '500'
                                }}>
                                  utilisé
                                </span>
                              )}
                            </div>
                            {model.object && (
                              <div style={{ fontSize: '12px', color: '#9ca3af' }}>
                                {model.object}
                              </div>
                            )}
                          </div>
                          {selectedModel === model.id && (
                            <CheckCircle size={16} style={{ color: '#a855f7', marginLeft: '8px' }} />
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Right Panel - Selection Info & Actions */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>


                {/* Action Button */}
                <button 
                  onClick={handleSave} 
                  disabled={!selectedModel || selectedModel === modelConfigStore.currentModel.id}
                  style={{
                    width: '100%',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '8px',
                    padding: '12px 16px',
                    background: '#8b5cf6',
                    color: '#ffffff',
                    border: 'none',
                    borderRadius: '8px',
                    fontWeight: '500',
                    cursor: (!selectedModel || selectedModel === modelConfigStore.currentModel.id) ? 'not-allowed' : 'pointer',
                    transition: 'background-color 0.2s',
                    opacity: (!selectedModel || selectedModel === modelConfigStore.currentModel.id) ? 0.5 : 1
                  }}
                  onMouseEnter={(e) => {
                    if (selectedModel && selectedModel !== modelConfigStore.currentModel.id) {
                      e.currentTarget.style.backgroundColor = '#7c3aed';
                    }
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = '#8b5cf6';
                  }}
                >
                  <CheckCircle size={16} />
                  <span>
                    {selectedModel === modelConfigStore.currentModel.id ? 'Modèle Actuel' : 'Appliquer la Sélection'}
                  </span>
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};