import React from 'react';
import { Loader2, CheckCircle, XCircle, Bot, RefreshCw } from 'lucide-react';
import { modelConfigStore, AVAILABLE_MODELS } from '@/lib/litellm';

interface ModelSource {
  name: string;
  type: string;
  baseUrl: string;
  modelCount: number;
  isAvailable: boolean;
}

interface ModelsTabProps {
  availableModels: any[];
  modelSources: ModelSource[];
  isLoading: boolean;
  error: string;
  selectedModel: string;
  onModelSelect: (modelId: string) => void;
  onLoadModels: () => void;
  onSave: () => void;
}

export const ModelsTab: React.FC<ModelsTabProps> = ({
  availableModels,
  modelSources,
  isLoading,
  error,
  selectedModel,
  onModelSelect,
  onLoadModels,
  onSave
}) => {
  const getModelCapabilities = (modelId: string): string[] => {
    const capabilities: string[] = [];
    
    // Fallback vers notre mapping statique
    const localModel = AVAILABLE_MODELS.find(m => m.id === modelId);
    if (localModel && localModel.capabilities) {
      capabilities.push(...localModel.capabilities);
    }
    
    return capabilities;
  };

  const getCapabilityColor = (capability: string): string => {
    switch (capability.toLowerCase()) {
      case 'vision': return '#f59e0b'; // orange
      case 'tools': return '#3b82f6'; // blue
      case 'parallel-tools': return '#2563eb'; // blue foncé
      case 'thinking': return '#8b5cf6'; // purple
      case 'reasoning': return '#10b981'; // green
      case 'code': return '#ef4444'; // red
      case 'multimodal': return '#f97316'; // orange
      default: return '#6b7280'; // gray
    }
  };

  if (isLoading) {
    const activeConnectionsCount = modelConfigStore.activeConnections.length;
    const connectionNames = modelConfigStore.activeConnections.map((c: any) => c.name).join(', ');

    return (
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
            {activeConnectionsCount === 0
              ? 'Détection automatique des modèles locaux...'
              : `Connexion à ${connectionNames}...`
            }
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
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
          onClick={onLoadModels} 
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
    );
  }

  return (
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
                {availableModels.length}
              </span>
            </h2>
            <button 
              onClick={onLoadModels}
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
            {availableModels.map((model) => (
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
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  onModelSelect(model.id);
                }}
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
                      fontSize: '14px',
                      marginBottom: '4px'
                    }}>
                      {model.id}
                      {selectedModel === model.id && (
                        <span style={{
                          padding: '2px 6px',
                          background: '#8b5cf6',
                          color: '#ffffff',
                          fontSize: '10px',
                          borderRadius: '4px',
                          fontWeight: '500'
                        }}>
                          sélectionné
                        </span>
                      )}
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
                      <div style={{ fontSize: '12px', color: '#9ca3af', marginBottom: '4px' }}>
                        {model.object}
                      </div>
                    )}
                    {/* Affichage des capacités */}
                    {(() => {
                      const capabilities = getModelCapabilities(model.id);
                      return capabilities.length > 0 && (
                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
                          {capabilities.map((capability: string) => (
                            <span 
                              key={capability}
                              style={{
                                padding: '1px 4px',
                                background: getCapabilityColor(capability),
                                color: '#ffffff',
                                fontSize: '9px',
                                borderRadius: '3px',
                                fontWeight: '500',
                                textTransform: 'lowercase'
                              }}
                            >
                              {capability}
                            </span>
                          ))}
                        </div>
                      );
                    })()}
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

        {/* Sources Info */}
        {modelSources.length > 0 && (
          <div style={{
            background: 'rgba(31, 41, 55, 0.5)',
            backdropFilter: 'blur(12px)',
            border: '1px solid #374151',
            borderRadius: '12px',
            padding: '16px'
          }}>
            <h3 style={{ 
              fontSize: '16px', 
              fontWeight: '600', 
              color: '#ffffff', 
              margin: '0 0 12px 0' 
            }}>
              Sources Actives
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {modelSources.map((source, index) => (
                <div key={index} style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '8px 12px',
                  background: 'rgba(75, 85, 99, 0.3)',
                  borderRadius: '6px',
                  border: `1px solid ${source.isAvailable ? '#16a34a' : '#ef4444'}`
                }}>
                  <div>
                    <div style={{ 
                      fontSize: '12px', 
                      fontWeight: '500', 
                      color: '#ffffff',
                      marginBottom: '2px'
                    }}>
                      {source.name}
                    </div>
                    <div style={{ fontSize: '10px', color: '#9ca3af' }}>
                      {source.type} • {source.modelCount} modèles
                    </div>
                  </div>
                  <div style={{
                    padding: '2px 6px',
                    background: source.isAvailable ? '#16a34a' : '#ef4444',
                    color: '#ffffff',
                    fontSize: '9px',
                    borderRadius: '4px'
                  }}>
                    {source.isAvailable ? 'OK' : 'OFF'}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Action Button */}
        <button 
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onSave();
          }} 
          disabled={!selectedModel}
          style={{
            width: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            padding: '12px 16px',
            background: !selectedModel ? '#6b7280' : (selectedModel === modelConfigStore.currentModel.id ? '#16a34a' : '#8b5cf6'),
            color: '#ffffff',
            border: 'none',
            borderRadius: '8px',
            fontWeight: '500',
            cursor: !selectedModel ? 'not-allowed' : 'pointer',
            transition: 'background-color 0.2s',
            opacity: !selectedModel ? 0.5 : 1
          }}
          onMouseEnter={(e) => {
            if (selectedModel) {
              if (selectedModel === modelConfigStore.currentModel.id) {
                e.currentTarget.style.backgroundColor = '#15803d';
              } else {
                e.currentTarget.style.backgroundColor = '#7c3aed';
              }
            }
          }}
          onMouseLeave={(e) => {
            if (selectedModel) {
              if (selectedModel === modelConfigStore.currentModel.id) {
                e.currentTarget.style.backgroundColor = '#16a34a';
              } else {
                e.currentTarget.style.backgroundColor = '#8b5cf6';
              }
            } else {
              e.currentTarget.style.backgroundColor = '#6b7280';
            }
          }}
        >
          <CheckCircle size={16} />
          <span>
            {!selectedModel ? 'Aucun modèle sélectionné' : 
             selectedModel === modelConfigStore.currentModel.id ? 'Modèle Actuel - Confirmer' : 
             'Appliquer la Sélection'}
          </span>
        </button>
      </div>
    </div>
  );
};