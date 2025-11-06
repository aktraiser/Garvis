import React, { useState, useEffect } from 'react';
import { AVAILABLE_MODELS } from '@/lib/litellm';

export interface ModelParameters {
  temperature: number;
  maxTokens: number;
  topP: number;
  frequencyPenalty: number;
  presencePenalty: number;
  systemPrompt: string;
}

interface ParametersTabProps {
  selectedModel: string;
  availableModels: any[];
  modelParameters: ModelParameters;
  setModelParameters: (params: ModelParameters) => void;
  onSave: () => void;
}

export const ParametersTab: React.FC<ParametersTabProps> = ({ 
  selectedModel, 
  availableModels,
  modelParameters, 
  setModelParameters, 
  onSave 
}) => {
  // États locaux pour une réactivité immédiate
  const [localParameters, setLocalParameters] = useState(modelParameters);
  const [selectedModelName, setSelectedModelName] = useState('');
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  
  // Trouve le nom du modèle sélectionné
  const getSelectedModelName = () => {
    if (!selectedModel) {
      return "aucun modèle sélectionné";
    }
    
    // Chercher d'abord dans les modèles disponibles
    let foundModel = availableModels.find(m => m.id === selectedModel);
    
    // Si pas trouvé, chercher dans AVAILABLE_MODELS
    if (!foundModel) {
      foundModel = AVAILABLE_MODELS.find(m => m.id === selectedModel);
    }
    
    return foundModel ? (foundModel.name || foundModel.id) : selectedModel;
  };
  
  // Mettre à jour le nom du modèle quand selectedModel change
  useEffect(() => {
    setSelectedModelName(getSelectedModelName());
  }, [selectedModel, availableModels]);

  // Synchroniser avec les props quand elles changent
  useEffect(() => {
    setLocalParameters(modelParameters);
  }, [modelParameters]);

  const handleParameterChange = (key: keyof ModelParameters, value: any) => {
    const newParameters = {
      ...localParameters,
      [key]: value
    };
    setLocalParameters(newParameters);
    setModelParameters(newParameters);
  };

  const handleSaveClick = async () => {
    console.log('🔧 ParametersTab handleSaveClick called!');
    console.log('🔧 Current localParameters:', localParameters);
    console.log('🔧 Current modelParameters prop:', modelParameters);
    
    setSaveStatus('saving');
    
    try {
      await onSave();
      setSaveStatus('success');
      
      // Réinitialiser le statut après 2 secondes
      setTimeout(() => {
        setSaveStatus('idle');
      }, 2000);
    } catch (error) {
      setSaveStatus('error');
      console.error('Erreur lors de la sauvegarde:', error);
      
      // Réinitialiser le statut après 3 secondes
      setTimeout(() => {
        setSaveStatus('idle');
      }, 3000);
    }
  };

  return (
    <div style={{ 
      maxWidth: '800px',
      margin: '0 auto',
      height: '100%',
      display: 'flex',
      flexDirection: 'column'
    }}>
      <div style={{
        background: 'rgba(31, 41, 55, 0.5)',
        backdropFilter: 'blur(12px)',
        border: '1px solid #374151',
        borderRadius: '12px',
        padding: '24px',
        flex: 1,
        display: 'flex',
        flexDirection: 'column'
      }}>
        <div style={{ 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'space-between',
          marginBottom: '24px'
        }}>
          <div>
            <h2 style={{ 
              fontSize: '24px', 
              fontWeight: '600', 
              margin: 0,
              color: '#ffffff',
              marginBottom: '8px'
            }}>
              ⚙️ Paramètres du Modèle
            </h2>
            <p style={{ 
              color: '#9ca3af',
              margin: 0,
              fontSize: '14px'
            }}>
              Configuration pour: <span style={{ color: '#60a5fa', fontWeight: '500' }}>{selectedModelName}</span>
            </p>
          </div>
        </div>

        <div style={{ display: 'grid', gap: '20px', flex: 1, overflowY: 'auto' }}>
          {/* Température */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '16px', alignItems: 'center' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Température
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Contrôle la créativité (0.0-1.0). Plus élevé = plus créatif mais moins cohérent.
              </p>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <input
                type="range"
                min="0"
                max="1"
                step="0.1"
                value={localParameters.temperature}
                onChange={(e) => handleParameterChange('temperature', parseFloat(e.target.value))}
                style={{
                  flex: 1,
                  accentColor: '#3b82f6'
                }}
              />
              <input
                type="number"
                min="0"
                max="1"
                step="0.1"
                value={localParameters.temperature}
                onChange={(e) => handleParameterChange('temperature', parseFloat(e.target.value))}
                style={{
                  width: '80px',
                  padding: '8px',
                  borderRadius: '6px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '14px'
                }}
              />
            </div>
          </div>

          {/* Max Tokens */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '16px', alignItems: 'center' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Tokens Maximum
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Limite le nombre de tokens générés dans la réponse.
              </p>
            </div>
            <input
              type="number"
              min="100"
              max="8000"
              step="100"
              value={localParameters.maxTokens}
              onChange={(e) => handleParameterChange('maxTokens', parseInt(e.target.value))}
              style={{
                padding: '12px',
                borderRadius: '8px',
                border: '1px solid #374151',
                background: '#1f2937',
                color: '#ffffff',
                fontSize: '14px'
              }}
            />
          </div>

          {/* Top P */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '16px', alignItems: 'center' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Top P
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Contrôle la diversité du vocabulaire (0.0-1.0). Alternative à la température.
              </p>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={localParameters.topP}
                onChange={(e) => handleParameterChange('topP', parseFloat(e.target.value))}
                style={{
                  flex: 1,
                  accentColor: '#10b981'
                }}
              />
              <input
                type="number"
                min="0"
                max="1"
                step="0.05"
                value={localParameters.topP}
                onChange={(e) => handleParameterChange('topP', parseFloat(e.target.value))}
                style={{
                  width: '80px',
                  padding: '8px',
                  borderRadius: '6px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '14px'
                }}
              />
            </div>
          </div>

          {/* Frequency Penalty */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '16px', alignItems: 'center' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Pénalité de Fréquence
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Réduit la répétition de mots (-2.0 à 2.0).
              </p>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <input
                type="range"
                min="-2"
                max="2"
                step="0.1"
                value={localParameters.frequencyPenalty}
                onChange={(e) => handleParameterChange('frequencyPenalty', parseFloat(e.target.value))}
                style={{
                  flex: 1,
                  accentColor: '#f59e0b'
                }}
              />
              <input
                type="number"
                min="-2"
                max="2"
                step="0.1"
                value={localParameters.frequencyPenalty}
                onChange={(e) => handleParameterChange('frequencyPenalty', parseFloat(e.target.value))}
                style={{
                  width: '80px',
                  padding: '8px',
                  borderRadius: '6px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '14px'
                }}
              />
            </div>
          </div>

          {/* Presence Penalty */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '16px', alignItems: 'center' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Pénalité de Présence
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Encourage de nouveaux sujets (-2.0 à 2.0).
              </p>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <input
                type="range"
                min="-2"
                max="2"
                step="0.1"
                value={localParameters.presencePenalty}
                onChange={(e) => handleParameterChange('presencePenalty', parseFloat(e.target.value))}
                style={{
                  flex: 1,
                  accentColor: '#8b5cf6'
                }}
              />
              <input
                type="number"
                min="-2"
                max="2"
                step="0.1"
                value={localParameters.presencePenalty}
                onChange={(e) => handleParameterChange('presencePenalty', parseFloat(e.target.value))}
                style={{
                  width: '80px',
                  padding: '8px',
                  borderRadius: '6px',
                  border: '1px solid #374151',
                  background: '#1f2937',
                  color: '#ffffff',
                  fontSize: '14px'
                }}
              />
            </div>
          </div>

          {/* System Prompt */}
          <div style={{ display: 'grid', gap: '12px' }}>
            <div>
              <label style={{ 
                fontSize: '14px', 
                color: '#ffffff', 
                fontWeight: '500',
                display: 'block',
                marginBottom: '4px'
              }}>
                Prompt Système
              </label>
              <p style={{ 
                fontSize: '12px', 
                color: '#9ca3af', 
                margin: 0,
                lineHeight: 1.4
              }}>
                Instructions qui définissent le comportement et la personnalité du modèle.
              </p>
            </div>
            <textarea
              value={localParameters.systemPrompt}
              onChange={(e) => handleParameterChange('systemPrompt', e.target.value)}
              placeholder="Tu es un assistant expert en... (optionnel)"
              rows={4}
              style={{
                padding: '12px',
                borderRadius: '8px',
                border: '1px solid #374151',
                background: '#1f2937',
                color: '#ffffff',
                fontSize: '14px',
                fontFamily: 'inherit',
                resize: 'vertical',
                minHeight: '100px'
              }}
            />
          </div>
        </div>

        {/* Bouton Appliquer - Sticky au bas */}
        <div style={{ 
          marginTop: '24px', 
          display: 'flex', 
          justifyContent: 'flex-end',
          flexShrink: 0,
          paddingTop: '16px',
          borderTop: '1px solid #374151'
        }}>
          <button 
            onClick={handleSaveClick}
            disabled={saveStatus === 'saving'}
            style={{
              padding: '12px 24px',
              background: saveStatus === 'success' 
                ? 'linear-gradient(135deg, #10b981 0%, #059669 100%)'
                : saveStatus === 'error'
                ? 'linear-gradient(135deg, #dc2626 0%, #b91c1c 100%)'
                : saveStatus === 'saving'
                ? 'linear-gradient(135deg, #6b7280 0%, #4b5563 100%)'
                : 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              fontWeight: '500',
              cursor: saveStatus === 'saving' ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              transition: 'all 0.2s ease',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              position: 'relative',
              zIndex: 1000,
              opacity: saveStatus === 'saving' ? 0.7 : 1
            }}
            onMouseEnter={(e) => {
              if (saveStatus === 'idle') {
                e.currentTarget.style.background = 'linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%)';
              }
            }}
            onMouseLeave={(e) => {
              if (saveStatus === 'idle') {
                e.currentTarget.style.background = 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)';
              }
            }}
          >
            {saveStatus === 'saving' && 'Enregistrement...'}
            {saveStatus === 'success' && 'Configuration enregistrée'}
            {saveStatus === 'error' && 'Erreur lors de l\'enregistrement'}
            {saveStatus === 'idle' && 'Appliquer la Configuration'}
          </button>
        </div>
      </div>
    </div>
  );
};