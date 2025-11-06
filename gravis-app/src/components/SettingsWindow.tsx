import React, { useState } from 'react';
import { ConnectionsTab } from './tabs/ConnectionsTab';
import { OllamaTab } from './tabs/OllamaTab';
import { HuggingFaceTab } from './tabs/HuggingFaceTab';

export interface SettingsWindowProps {
  onClose: () => void;
}

export const SettingsWindow: React.FC<SettingsWindowProps> = () => {
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
        padding: '16px 24px 0 24px',
        display: 'flex',
        alignItems: 'end',
        justifyContent: 'space-between'
      }}>
        <div style={{ display: 'flex', gap: '2px', marginBottom: '-1px' }}>
          <button
            onClick={() => setActiveTab('connections')}
            style={{
              padding: '12px 24px 16px 24px',
              background: activeTab === 'connections' 
                ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
              color: activeTab === 'connections' ? '#ffffff' : '#d1d5db',
              border: '1px solid #475569',
              borderBottom: activeTab === 'connections' ? '1px solid #0f172a' : '1px solid #475569',
              borderTopLeftRadius: '12px',
              borderTopRightRadius: '12px',
              borderBottomLeftRadius: '0',
              borderBottomRightRadius: '0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease',
              position: 'relative',
              zIndex: activeTab === 'connections' ? 2 : 1,
              boxShadow: activeTab === 'connections' 
                ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                : '0 2px 4px rgba(0, 0, 0, 0.1)'
            }}
          >
            Connexions
          </button>
          <button
            onClick={() => setActiveTab('ollama')}
            style={{
              padding: '12px 24px 16px 24px',
              background: activeTab === 'ollama' 
                ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
              color: activeTab === 'ollama' ? '#ffffff' : '#d1d5db',
              border: '1px solid #475569',
              borderBottom: activeTab === 'ollama' ? '1px solid #0f172a' : '1px solid #475569',
              borderTopLeftRadius: '12px',
              borderTopRightRadius: '12px',
              borderBottomLeftRadius: '0',
              borderBottomRightRadius: '0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease',
              position: 'relative',
              zIndex: activeTab === 'ollama' ? 2 : 1,
              boxShadow: activeTab === 'ollama' 
                ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                : '0 2px 4px rgba(0, 0, 0, 0.1)'
            }}
          >
            Ollama
          </button>
          <button
            onClick={() => setActiveTab('huggingface')}
            style={{
              padding: '12px 24px 16px 24px',
              background: activeTab === 'huggingface' 
                ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' 
                : 'linear-gradient(135deg, #374151 0%, #4b5563 100%)',
              color: activeTab === 'huggingface' ? '#ffffff' : '#d1d5db',
              border: '1px solid #475569',
              borderBottom: activeTab === 'huggingface' ? '1px solid #0f172a' : '1px solid #475569',
              borderTopLeftRadius: '12px',
              borderTopRightRadius: '12px',
              borderBottomLeftRadius: '0',
              borderBottomRightRadius: '0',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
              transition: 'all 0.2s ease',
              position: 'relative',
              zIndex: activeTab === 'huggingface' ? 2 : 1,
              boxShadow: activeTab === 'huggingface' 
                ? '0 -2px 8px rgba(0, 0, 0, 0.3)' 
                : '0 2px 4px rgba(0, 0, 0, 0.1)'
            }}
          >
            Hugging Face
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
        {activeTab === 'connections' && <ConnectionsTab />}
        {activeTab === 'ollama' && <OllamaTab />}
        {activeTab === 'huggingface' && <HuggingFaceTab />}
        </div>
      </div>
    </div>
  );
};

export default SettingsWindow;