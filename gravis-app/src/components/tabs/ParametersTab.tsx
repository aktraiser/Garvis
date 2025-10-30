import React, { useState, useEffect } from 'react';
import { CheckCircle } from 'lucide-react';

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
  modelParameters: ModelParameters;
  setModelParameters: (params: ModelParameters) => void;
  onSave: () => void;
}

export const ParametersTab: React.FC<ParametersTabProps> = ({ 
  selectedModel, 
  modelParameters, 
  setModelParameters, 
  onSave 
}) => {
  // États locaux pour une réactivité immédiate
  const [localParameters, setLocalParameters] = useState(modelParameters);

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

  const handleSaveClick = () => {
    console.log('🔧 ParametersTab handleSaveClick called!');
    console.log('🔧 Current localParameters:', localParameters);
    console.log('🔧 Current modelParameters prop:', modelParameters);
    onSave();
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
              Configuration pour: <span style={{ color: '#60a5fa', fontWeight: '500' }}>{selectedModel}</span>
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
            style={{
              padding: '12px 24px',
              background: 'linear-gradient(135deg, #16a34a 0%, #15803d 100%)',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              fontWeight: '500',
              cursor: 'pointer',
              fontSize: '14px',
              transition: 'all 0.2s ease',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              position: 'relative',
              zIndex: 1000
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'linear-gradient(135deg, #15803d 0%, #166534 100%)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'linear-gradient(135deg, #16a34a 0%, #15803d 100%)';
            }}
          >
            <CheckCircle size={16} />
            Appliquer la Configuration
          </button>
        </div>
      </div>
    </div>
  );
};