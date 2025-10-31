// GRAVIS AWCS - Section d'activation pour ConnectionTab
// Interface d'activation AWCS intégrée

import React, { useState } from 'react';
import { Eye, CheckCircle, AlertCircle, XCircle, Loader2, TestTube, Shield, Info, Camera } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAWCS } from '../hooks/useAWCS';
import { AWCSActivationState, AWCSPermissions, AWCSUtils } from '../types/awcs';

export const AWCSSection: React.FC = () => {
  const {
    state,
    permissions,
    isLoading,
    error,
    activateAWCS,
    deactivateAWCS,
    testCurrentWindow,
    requestPermissions,
    isActive,
    hasRequiredPermissions,
    clearError,
  } = useAWCS();

  const [showPermissionsHelp, setShowPermissionsHelp] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  const handleTest = async () => {
    setTestResult('⏳ Changez de fenêtre maintenant ! Test dans 2 secondes...');
    
    const context = await testCurrentWindow();
    
    if (context) {
      const confidence = AWCSUtils.calculateOverallConfidence(context);
      setTestResult(
        `✅ Test réussi: ${context.source.app} (${Math.round(confidence * 100)}% fiable, méthode: ${context.confidence.extractionMethod})`
      );
    } else {
      setTestResult('❌ Test échoué - voir les détails d\'erreur ci-dessus');
    }
    
    // Effacer le résultat après 5 secondes
    setTimeout(() => setTestResult(null), 5000);
  };

  const handleTestOCR = async () => {
    setTestResult('⏳ Mode OCR Direct - Changez de fenêtre ! Test dans 2 secondes...');
    
    try {
      const context = await invoke('awcs_get_context_ocr_direct') as any;
      const confidence = AWCSUtils.calculateOverallConfidence(context);
      const extractedText = context.content.fulltext || '';
      const textPreview = extractedText.length > 500 
        ? extractedText.substring(0, 500) + '...' 
        : extractedText;
      
      setTestResult(
        `✅ OCR Direct: ${context.source.app} (${Math.round(confidence * 100)}% fiable, méthode: ${context.confidence.extractionMethod})
        
📄 Contenu extrait (${extractedText.length} caractères):
"${textPreview}"`
      );
    } catch (error) {
      setTestResult(`❌ Erreur OCR: ${error}`);
    }
    
    // Effacer le résultat après 15 secondes (plus de temps pour lire)
    setTimeout(() => setTestResult(null), 15000);
  };

  const handleAction = () => {
    if (isActive) {
      deactivateAWCS();
    } else {
      activateAWCS();
    }
  };

  return (
    <div style={{
      borderTop: '1px solid rgba(255, 255, 255, 0.1)',
      paddingTop: '24px',
      marginTop: '32px'
    }}>
      {/* Header Section */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        marginBottom: '16px'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <Eye size={20} style={{ color: '#3b82f6' }} />
          <h3 style={{
            fontSize: '18px',
            fontWeight: '600',
            margin: 0,
            color: '#ffffff'
          }}>
            Active Window Context Service
          </h3>
          <span style={{
            fontSize: '11px',
            fontWeight: '500',
            color: '#60a5fa',
            backgroundColor: 'rgba(59, 130, 246, 0.2)',
            padding: '2px 6px',
            borderRadius: '4px'
          }}>
            BETA
          </span>
        </div>
        
        <AWCSActivationButton
          state={state}
          isLoading={isLoading}
          onAction={handleAction}
          isActive={isActive}
        />
      </div>

      {/* Description */}
      <p style={{
        color: '#9ca3af',
        margin: '0 0 16px 0',
        fontSize: '14px',
        lineHeight: '1.4'
      }}>
        Analysez le contenu de votre fenêtre active avec{' '}
        <kbd style={{
          backgroundColor: 'rgba(255, 255, 255, 0.1)',
          padding: '2px 4px',
          borderRadius: '3px',
          fontSize: '12px',
          fontFamily: 'monospace'
        }}>
          {navigator.platform.includes('Mac') ? '⌘⇧G' : 'Ctrl+Shift+G'}
        </kbd>
      </p>

      {/* Status Cards */}
      <StatusCard
        state={state}
        permissions={permissions}
        hasRequiredPermissions={hasRequiredPermissions}
        onShowPermissionsHelp={() => setShowPermissionsHelp(true)}
      />

      {/* Action Buttons Row */}
      {isActive && (
        <div style={{
          display: 'flex',
          gap: '8px',
          marginTop: '12px'
        }}>
          <button
            onClick={handleTest}
            style={{
              padding: '8px 16px',
              backgroundColor: 'rgba(16, 185, 129, 0.1)',
              color: '#10b981',
              border: '1px solid rgba(16, 185, 129, 0.3)',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '12px',
              fontWeight: '500',
              display: 'flex',
              alignItems: 'center',
              gap: '6px'
            }}
          >
            <TestTube size={12} />
            Test Standard
          </button>

          <button
            onClick={handleTestOCR}
            style={{
              padding: '8px 16px',
              backgroundColor: 'rgba(147, 51, 234, 0.1)',
              color: '#9333ea',
              border: '1px solid rgba(147, 51, 234, 0.3)',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '12px',
              fontWeight: '500',
              display: 'flex',
              alignItems: 'center',
              gap: '6px'
            }}
          >
            <Camera size={12} />
            Test OCR Direct
          </button>
        </div>
      )}

      {/* Test Result */}
      {testResult && (
        <div style={{
          marginTop: '12px',
          padding: '12px',
          backgroundColor: testResult.includes('✅') 
            ? 'rgba(16, 185, 129, 0.1)' 
            : 'rgba(239, 68, 68, 0.1)',
          border: `1px solid ${testResult.includes('✅') 
            ? 'rgba(16, 185, 129, 0.3)' 
            : 'rgba(239, 68, 68, 0.3)'}`,
          borderRadius: '8px',
          fontSize: '12px',
          color: testResult.includes('✅') ? '#10b981' : '#ef4444',
          fontFamily: 'SF Mono, Monaco, monospace',
          whiteSpace: 'pre-wrap',
          maxHeight: '500px',
          overflowY: 'auto'
        }}>
          {testResult}
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div style={{
          marginTop: '12px',
          padding: '12px 16px',
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid rgba(239, 68, 68, 0.3)',
          borderRadius: '8px',
          display: 'flex',
          alignItems: 'flex-start',
          gap: '8px'
        }}>
          <XCircle size={16} style={{ color: '#ef4444', marginTop: '2px', flexShrink: 0 }} />
          <div style={{ flex: 1 }}>
            <p style={{
              margin: '0 0 8px 0',
              fontSize: '13px',
              color: '#ef4444',
              fontWeight: '500'
            }}>
              Erreur AWCS
            </p>
            <p style={{
              margin: 0,
              fontSize: '12px',
              color: '#fca5a5',
              lineHeight: '1.4'
            }}>
              {error}
            </p>
            <button
              onClick={clearError}
              style={{
                marginTop: '8px',
                padding: '4px 8px',
                backgroundColor: 'transparent',
                color: '#fca5a5',
                border: '1px solid rgba(239, 68, 68, 0.3)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px'
              }}
            >
              Fermer
            </button>
          </div>
        </div>
      )}

      {/* Permissions Help Modal */}
      {showPermissionsHelp && (
        <PermissionsHelpModal
          permissions={permissions}
          onClose={() => setShowPermissionsHelp(false)}
          onRequestPermissions={requestPermissions}
        />
      )}
    </div>
  );
};

// === Composants auxiliaires ===

interface AWCSActivationButtonProps {
  state: AWCSActivationState;
  isLoading: boolean;
  onAction: () => void;
  isActive: boolean;
}

const AWCSActivationButton: React.FC<AWCSActivationButtonProps> = ({
  state,
  isLoading,
  onAction,
  isActive: _isActive
}) => {
  const getButtonConfig = () => {
    switch (state) {
      case AWCSActivationState.Disabled:
        return {
          text: 'Activer Context Service',
          icon: <Eye size={16} />,
          backgroundColor: '#3b82f6',
          hoverColor: '#2563eb'
        };
      case AWCSActivationState.PermissionsPending:
        return {
          text: 'Configuration...',
          icon: <Loader2 size={16} className="animate-spin" />,
          backgroundColor: '#6b7280',
          hoverColor: '#6b7280'
        };
      case AWCSActivationState.Active:
        return {
          text: '⌘⇧G Actif',
          icon: <CheckCircle size={16} />,
          backgroundColor: '#16a34a',
          hoverColor: '#15803d'
        };
      case AWCSActivationState.Error:
        return {
          text: 'Réessayer',
          icon: <AlertCircle size={16} />,
          backgroundColor: '#dc2626',
          hoverColor: '#b91c1c'
        };
      default:
        return {
          text: 'Activer',
          icon: <Eye size={16} />,
          backgroundColor: '#3b82f6',
          hoverColor: '#2563eb'
        };
    }
  };

  const config = getButtonConfig();

  return (
    <button
      onClick={onAction}
      disabled={isLoading}
      style={{
        minWidth: '140px',
        padding: '10px 20px',
        backgroundColor: config.backgroundColor,
        color: 'white',
        border: 'none',
        borderRadius: '8px',
        cursor: isLoading ? 'not-allowed' : 'pointer',
        fontSize: '14px',
        fontWeight: '500',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '8px',
        transition: 'background-color 0.2s',
        opacity: isLoading ? 0.7 : 1
      }}
      onMouseEnter={(e) => {
        if (!isLoading) {
          e.currentTarget.style.backgroundColor = config.hoverColor;
        }
      }}
      onMouseLeave={(e) => {
        if (!isLoading) {
          e.currentTarget.style.backgroundColor = config.backgroundColor;
        }
      }}
    >
      {config.icon}
      {config.text}
    </button>
  );
};

interface StatusCardProps {
  state: AWCSActivationState;
  permissions: AWCSPermissions | null;
  hasRequiredPermissions: boolean;
  onShowPermissionsHelp: () => void;
}

const StatusCard: React.FC<StatusCardProps> = ({
  state,
  permissions: _permissions,
  hasRequiredPermissions: _hasRequiredPermissions,
  onShowPermissionsHelp
}) => {
  const getStatusConfig = () => {
    switch (state) {
      case AWCSActivationState.Active:
        return {
          backgroundColor: 'rgba(16, 185, 129, 0.1)',
          borderColor: 'rgba(16, 185, 129, 0.3)',
          icon: <CheckCircle size={16} style={{ color: '#10b981' }} />,
          title: 'AWCS Actif',
          description: 'Extraction intelligente - Privacy-first - Données locales en priorité',
          titleColor: '#10b981',
          descColor: '#6ee7b7'
        };
      case AWCSActivationState.PermissionsPending:
        return {
          backgroundColor: 'rgba(245, 158, 11, 0.1)',
          borderColor: 'rgba(245, 158, 11, 0.3)',
          icon: <AlertCircle size={16} style={{ color: '#f59e0b' }} />,
          title: 'Configuration des permissions',
          description: 'Autorisations système en cours...',
          titleColor: '#f59e0b',
          descColor: '#fbbf24'
        };
      case AWCSActivationState.Error:
        return {
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          borderColor: 'rgba(239, 68, 68, 0.3)',
          icon: <XCircle size={16} style={{ color: '#ef4444' }} />,
          title: 'Échec d\'activation',
          description: 'Vérifiez les permissions dans Préférences Système',
          titleColor: '#ef4444',
          descColor: '#fca5a5'
        };
      default:
        return null;
    }
  };

  const statusConfig = getStatusConfig();

  if (!statusConfig) return null;

  return (
    <div style={{
      backgroundColor: statusConfig.backgroundColor,
      border: `1px solid ${statusConfig.borderColor}`,
      borderRadius: '8px',
      padding: '12px 16px'
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        marginBottom: '6px'
      }}>
        {statusConfig.icon}
        <span style={{
          fontSize: '14px',
          fontWeight: '500',
          color: statusConfig.titleColor
        }}>
          {statusConfig.title}
        </span>
        {state === AWCSActivationState.Active && (
          <kbd style={{
            backgroundColor: 'rgba(255, 255, 255, 0.9)',
            color: '#1f2937',
            padding: '2px 6px',
            borderRadius: '4px',
            fontSize: '11px',
            fontFamily: 'monospace',
            fontWeight: '500'
          }}>
            {navigator.platform.includes('Mac') ? '⌘⇧G' : 'Ctrl+Shift+G'}
          </kbd>
        )}
      </div>
      <div style={{
        fontSize: '12px',
        color: statusConfig.descColor,
        lineHeight: '1.4'
      }}>
        {statusConfig.description}
        {state === AWCSActivationState.PermissionsPending && (
          <>
            {' '}
            <button
              onClick={onShowPermissionsHelp}
              style={{
                background: 'none',
                border: 'none',
                color: statusConfig.titleColor,
                textDecoration: 'underline',
                cursor: 'pointer',
                fontSize: '12px',
                padding: 0
              }}
            >
              Aide avec les permissions
            </button>
          </>
        )}
      </div>
    </div>
  );
};

interface PermissionsHelpModalProps {
  permissions: AWCSPermissions | null;
  onClose: () => void;
  onRequestPermissions: () => void;
}

const PermissionsHelpModal: React.FC<PermissionsHelpModalProps> = ({
  permissions,
  onClose,
  onRequestPermissions
}) => {
  const steps = [
    {
      id: 'accessibility',
      title: 'Accessibilité',
      description: 'Permet à GRAVIS de lire le contenu des applications',
      required: true,
      granted: permissions?.accessibility || false,
      instructions: 'Préférences Système > Sécurité et confidentialité > Accessibilité'
    },
    {
      id: 'automation',
      title: 'Automation',
      description: 'Permet l\'extraction via AppleScript/COM',
      required: true,
      granted: permissions?.automation || false,
      instructions: 'Préférences Système > Sécurité et confidentialité > Automation'
    },
    {
      id: 'screenRecording',
      title: 'Enregistrement d\'écran',
      description: 'Pour le fallback OCR uniquement (optionnel)',
      required: false,
      granted: permissions?.screenRecording || false,
      instructions: 'Préférences Système > Sécurité et confidentialité > Enregistrement d\'écran'
    }
  ];

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000
    }}>
      <div style={{
        backgroundColor: '#1f2937',
        borderRadius: '12px',
        padding: '24px',
        maxWidth: '480px',
        width: '90%',
        maxHeight: '80vh',
        overflow: 'auto'
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          marginBottom: '16px'
        }}>
          <Shield size={20} style={{ color: '#3b82f6' }} />
          <h3 style={{
            fontSize: '18px',
            fontWeight: '600',
            margin: 0,
            color: '#ffffff'
          }}>
            Permissions AWCS
          </h3>
        </div>
        
        <p style={{
          color: '#9ca3af',
          fontSize: '14px',
          marginBottom: '20px',
          lineHeight: '1.4'
        }}>
          GRAVIS a besoin de certaines permissions pour analyser vos fenêtres actives
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '20px' }}>
          {steps.map((step) => (
            <div key={step.id} style={{
              display: 'flex',
              alignItems: 'flex-start',
              gap: '12px',
              padding: '12px',
              backgroundColor: 'rgba(255, 255, 255, 0.05)',
              borderRadius: '8px',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            }}>
              <div style={{ marginTop: '2px' }}>
                {step.granted ? (
                  <CheckCircle size={16} style={{ color: '#10b981' }} />
                ) : step.required ? (
                  <AlertCircle size={16} style={{ color: '#f59e0b' }} />
                ) : (
                  <Info size={16} style={{ color: '#6b7280' }} />
                )}
              </div>
              
              <div style={{ flex: 1 }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  marginBottom: '4px'
                }}>
                  <h4 style={{
                    fontSize: '14px',
                    fontWeight: '500',
                    margin: 0,
                    color: '#ffffff'
                  }}>
                    {step.title}
                  </h4>
                  {step.required && (
                    <span style={{
                      fontSize: '10px',
                      fontWeight: '500',
                      color: '#f59e0b',
                      backgroundColor: 'rgba(245, 158, 11, 0.2)',
                      padding: '2px 6px',
                      borderRadius: '4px'
                    }}>
                      Requis
                    </span>
                  )}
                </div>
                <p style={{
                  fontSize: '12px',
                  color: '#9ca3af',
                  margin: '0 0 8px 0',
                  lineHeight: '1.3'
                }}>
                  {step.description}
                </p>
                {!step.granted && (
                  <p style={{
                    fontSize: '11px',
                    color: '#60a5fa',
                    fontFamily: 'monospace',
                    margin: 0,
                    lineHeight: '1.3'
                  }}>
                    {step.instructions}
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>

        <div style={{
          display: 'flex',
          gap: '12px',
          justifyContent: 'flex-end'
        }}>
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              backgroundColor: 'rgba(255, 255, 255, 0.1)',
              color: '#d1d5db',
              border: '1px solid rgba(255, 255, 255, 0.2)',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px'
            }}
          >
            Fermer
          </button>
          <button
            onClick={() => {
              onRequestPermissions();
              onClose();
            }}
            style={{
              padding: '8px 16px',
              backgroundColor: '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500'
            }}
          >
            Ouvrir Préférences
          </button>
        </div>
      </div>
    </div>
  );
};