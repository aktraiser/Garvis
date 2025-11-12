import React, { useState, useEffect } from 'react';
import { modelConfigStore, AVAILABLE_MODELS } from '@/lib/litellm';
import { unifiedModelClient } from '@/lib/unified-model-client';
import { tauriModelStore } from '@/lib/tauri-model-store';
import { ModelsTab } from './tabs/ModelsTab';
import { ParametersTab, ModelParameters } from './tabs/ParametersTab';

interface ModelSelectorWindowProps {
  onClose: () => void;
}

type TabType = 'models' | 'parameters';

export const ModelSelectorWindow: React.FC<ModelSelectorWindowProps> = ({ onClose }) => {
  // États pour les modèles
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [modelSources, setModelSources] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedModel, setSelectedModel] = useState('');
  
  // États pour l'interface
  const [activeTab, setActiveTab] = useState<TabType>('models');
  
  // États pour les paramètres
  const [modelParameters, setModelParameters] = useState<ModelParameters>(modelConfigStore.modelParameters);

  useEffect(() => {
    const initializeParams = async () => {
      // S'assurer que le store est initialisé
      await modelConfigStore.init();
      console.log('🔧 Store initialized, parameters:', modelConfigStore.modelParameters);
      setModelParameters(modelConfigStore.modelParameters);
    };
    
    loadModels();
    initializeParams();
  }, []);

  // Synchronisation initiale uniquement (pas de polling)
  useEffect(() => {
    // Synchroniser une seule fois au montage sans intervalle
    setModelParameters(modelConfigStore.modelParameters);
  }, []);

  const loadModels = async () => {
    setIsLoading(true);
    setError('');

    try {
      const unifiedResponse = await unifiedModelClient.getAllAvailableModels();
      
      setAvailableModels(unifiedResponse.models);
      setModelSources(unifiedResponse.sources);
      
      if (unifiedResponse.models.length === 0) {
        setError('Aucun modèle disponible. Vérifiez vos connexions dans les paramètres.');
      }
    } catch (err) {
      console.error('Error loading models:', err);
      setError(err instanceof Error ? err.message : 'Erreur de connexion');

      // Ne jamais afficher de modèles par défaut si les connexions échouent
      // Cela force l'utilisateur à vérifier ses connexions
      setAvailableModels([]);
      setModelSources([]);

      if (modelConfigStore.activeConnections.length === 0 && !modelConfigStore.selectedConnectionId) {
        setError('Aucune connexion configurée. Veuillez ajouter une connexion LiteLLM, Ollama ou Modal dans les paramètres.');
      } else {
        const connectionNames = modelConfigStore.activeConnections.map((c: any) => c.name).join(', ');
        setError(`Impossible de se connecter à ${connectionNames}. Vérifiez que les serveurs sont accessibles et démarrés.`);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleModelSelect = (modelId: string) => {
    setSelectedModel(modelId);
  };

  const handleSave = async () => {
    if (!selectedModel) {
      return;
    }
    
    // Chercher le modèle sélectionné
    let foundModel = availableModels.find(m => m.id === selectedModel);
    
    if (!foundModel) {
      foundModel = AVAILABLE_MODELS.find(m => m.id === selectedModel);
    }
    
    if (!foundModel) {
      foundModel = modelConfigStore.currentModel;
    }
    
    // Assurer que le modèle a un nom
    if (foundModel && !foundModel.name) {
      foundModel = {
        ...foundModel,
        name: foundModel.id
      };
    }
    
    try {
      // Utiliser le système d'événements Tauri au lieu de localStorage
      await tauriModelStore.emitModelChanged(foundModel);
      
      // Optionnel : broadcaster spécifiquement à la fenêtre principale
      try {
        await tauriModelStore.emitToWindow('main', foundModel);
      } catch (error) {
        // Ignore les erreurs si la fenêtre principale n'existe pas
      }
      
    } catch (error) {
      // Fallback vers localStorage en cas d'échec
      try {
        modelConfigStore.setModel(foundModel);
        
        const storageEvent = new StorageEvent('storage', {
          key: 'gravis-config',
          newValue: localStorage.getItem('gravis-config'),
          oldValue: null,
          storageArea: localStorage,
          url: window.location.href
        });
        
        window.dispatchEvent(storageEvent);
      } catch (fallbackError) {
        return;
      }
    }
    
    // Fermer la fenêtre après une courte pause
    setTimeout(() => {
      onClose();
    }, 300);
  };

  const handleParametersSave = async () => {
    console.log('🔧 handleParametersSave called with:', modelParameters);
    
    try {
      // Utiliser le système Tauri pour émettre les changements de paramètres
      await tauriModelStore.emitParametersChanged(modelParameters);
      
      // Fermer la fenêtre après succès
      setTimeout(() => {
        onClose();
      }, 300);
    } catch (error) {
      console.error('Failed to save parameters via Tauri:', error);
      // Fallback vers localStorage
      modelConfigStore.setModelParameters(modelParameters);
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
      flexDirection: 'column',
      overflow: 'hidden'
    }}>
        
        {/* Header avec onglets - Sticky */}
        <div style={{ 
          background: 'linear-gradient(90deg, #1e293b 0%, #334155 100%)',
          borderBottom: '1px solid #475569',
          padding: '16px 24px 0 24px',
          display: 'flex',
          alignItems: 'end',
          justifyContent: 'space-between',
          position: 'sticky',
          top: 0,
          zIndex: 10,
          flexShrink: 0
        }}>
          <div style={{ display: 'flex', gap: '2px', marginBottom: '-1px' }}>
            <button
              onClick={() => setActiveTab('models')}
              style={{
                padding: '12px 24px 16px 24px',
                background: activeTab === 'models' 
                  ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                  : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
                color: activeTab === 'models' ? '#ffffff' : '#d1d5db',
                border: '1px solid #475569',
                borderBottom: activeTab === 'models' ? '1px solid #0f172a' : '1px solid #475569',
                borderTopLeftRadius: '12px',
                borderTopRightRadius: '12px',
                borderBottomLeftRadius: '0',
                borderBottomRightRadius: '0',
                cursor: 'pointer',
                fontSize: '14px',
                fontWeight: '500',
                transition: 'all 0.2s ease',
                position: 'relative',
                zIndex: activeTab === 'models' ? 2 : 1,
                boxShadow: activeTab === 'models' 
                  ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                  : '0 2px 4px rgba(0, 0, 0, 0.1)'
              }}
            >
              Modèles
            </button>
            <button
              onClick={() => setActiveTab('parameters')}
              disabled={!selectedModel}
              style={{
                padding: '12px 24px 16px 24px',
                background: activeTab === 'parameters' 
                  ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                  : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
                color: activeTab === 'parameters' ? '#ffffff' : selectedModel ? '#d1d5db' : '#6b7280',
                border: '1px solid #475569',
                borderBottom: activeTab === 'parameters' ? '1px solid #0f172a' : '1px solid #475569',
                borderTopLeftRadius: '12px',
                borderTopRightRadius: '12px',
                borderBottomLeftRadius: '0',
                borderBottomRightRadius: '0',
                cursor: selectedModel ? 'pointer' : 'not-allowed',
                fontSize: '14px',
                fontWeight: '500',
                transition: 'all 0.2s ease',
                position: 'relative',
                zIndex: activeTab === 'parameters' ? 2 : 1,
                boxShadow: activeTab === 'parameters' 
                  ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                  : '0 2px 4px rgba(0, 0, 0, 0.1)',
                opacity: selectedModel ? 1 : 0.5
              }}
            >
              Paramètres
            </button>
          </div>
        </div>

        {/* Contenu des onglets - Scrollable */}
        <div style={{ 
          flex: 1, 
          overflow: 'auto',
          background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f1629 100%)'
        }}>
          <div style={{ padding: '24px' }}>
          {activeTab === 'models' ? (
            <ModelsTab
              availableModels={availableModels}
              modelSources={modelSources}
              isLoading={isLoading}
              error={error}
              selectedModel={selectedModel}
              onModelSelect={handleModelSelect}
              onLoadModels={loadModels}
              onSave={handleSave}
            />
          ) : (
            <ParametersTab
              selectedModel={selectedModel}
              availableModels={availableModels}
              modelParameters={modelParameters}
              setModelParameters={setModelParameters}
              onSave={handleParametersSave}
            />
          )}
          </div>
        </div>
    </div>
  );
};