import React, { useState } from 'react';
import { ConnectionsTab } from './tabs/ConnectionsTab';
import { OllamaTab } from './tabs/OllamaTab';
import { HuggingFaceTab } from './tabs/HuggingFaceTab';

export interface SettingsWindowProps {
  onClose: () => void;
}

export const SettingsWindow: React.FC<SettingsWindowProps> = ({ onClose }) => {
  const [activeTab, setActiveTab] = useState<'connections' | 'ollama' | 'huggingface'>('connections');

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
      {/* Header avec onglets */}
      <div style={{ 
        background: 'linear-gradient(90deg, #1e293b 0%, #334155 100%)',
        borderBottom: '1px solid #475569',
        padding: '16px 24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between'
      }}>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button
            onClick={() => setActiveTab('connections')}
            style={{
              padding: '12px 24px',
              background: activeTab === 'connections' 
                ? 'linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%)' 
                : 'transparent',
              color: activeTab === 'connections' ? '#ffffff' : '#94a3b8',
              border: activeTab === 'connections' 
                ? '1px solid #3b82f6' 
                : '1px solid transparent',
              borderRadius: '8px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease'
            }}
          >
            🔗 Connexions
          </button>
          <button
            onClick={() => setActiveTab('ollama')}
            style={{
              padding: '12px 24px',
              background: activeTab === 'ollama' 
                ? 'linear-gradient(135deg, #10b981 0%, #047857 100%)' 
                : 'transparent',
              color: activeTab === 'ollama' ? '#ffffff' : '#94a3b8',
              border: activeTab === 'ollama' 
                ? '1px solid #10b981' 
                : '1px solid transparent',
              borderRadius: '8px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease'
            }}
          >
            🦙 Ollama
          </button>
          <button
            onClick={() => setActiveTab('huggingface')}
            style={{
              padding: '12px 24px',
              background: activeTab === 'huggingface' 
                ? 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)' 
                : 'transparent',
              color: activeTab === 'huggingface' ? '#ffffff' : '#94a3b8',
              border: activeTab === 'huggingface' 
                ? '1px solid #f59e0b' 
                : '1px solid transparent',
              borderRadius: '8px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease'
            }}
          >
            🤗 Hugging Face
          </button>
        </div>
        
        <button
          onClick={onClose}
          style={{
            padding: '8px 16px',
            background: 'transparent',
            color: '#94a3b8',
            border: '1px solid #475569',
            borderRadius: '6px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: '500',
            transition: 'all 0.2s ease'
          }}
          onMouseOver={(e) => {
            e.currentTarget.style.background = '#dc2626';
            e.currentTarget.style.color = '#ffffff';
            e.currentTarget.style.borderColor = '#dc2626';
          }}
          onMouseOut={(e) => {
            e.currentTarget.style.background = 'transparent';
            e.currentTarget.style.color = '#94a3b8';
            e.currentTarget.style.borderColor = '#475569';
          }}
        >
          ✕ Fermer
        </button>
      </div>

      {/* Content */}
      <div style={{ 
        flex: 1, 
        padding: '24px',
        overflow: 'auto',
        background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f1629 100%)'
      }}>
        {activeTab === 'connections' && <ConnectionsTab />}
        {activeTab === 'ollama' && <OllamaTab />}
        {activeTab === 'huggingface' && <HuggingFaceTab />}
      </div>
    </div>
  );
};

export default SettingsWindow;