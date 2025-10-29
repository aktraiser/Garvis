import React, { useState, useEffect } from 'react';
import { modelConfigStore } from '@/lib/litellm';
import { unifiedModelClient } from '@/lib/unified-model-client';

interface DebugModelWindowProps {
  onClose: () => void;
}

export const DebugModelWindow: React.FC<DebugModelWindowProps> = ({ onClose }) => {
  const [debugInfo, setDebugInfo] = useState<any>({});
  const [isLoading, setIsLoading] = useState(false);

  const runDebug = async () => {
    setIsLoading(true);
    const debug: any = {};

    try {
      // 1. État du store
      debug.store = {
        currentModel: modelConfigStore.currentModel,
        activeConnections: modelConfigStore.activeConnections,
        apiKey: modelConfigStore.apiKey ? '***SET***' : 'NOT_SET',
        baseUrl: modelConfigStore.baseUrl
      };

      // 2. Connexions actives depuis localStorage
      const saved = localStorage.getItem('gravis-config');
      debug.localStorage = saved ? JSON.parse(saved) : null;

      // 3. Modèles unifiés
      try {
        const unifiedResponse = await unifiedModelClient.getAllAvailableModels();
        debug.unifiedModels = {
          modelCount: unifiedResponse.models.length,
          models: unifiedResponse.models.map(m => ({ id: m.id, name: m.name, provider: m.provider })),
          sources: unifiedResponse.sources
        };
      } catch (err) {
        debug.unifiedModelsError = err instanceof Error ? err.message : String(err);
      }

      // 4. Test Ollama direct
      try {
        const ollamaResponse = await fetch('http://localhost:11434/api/tags');
        if (ollamaResponse.ok) {
          const ollamaData = await ollamaResponse.json();
          debug.ollamaDirect = ollamaData;
        } else {
          debug.ollamaDirectError = `HTTP ${ollamaResponse.status}`;
        }
      } catch (err) {
        debug.ollamaDirectError = err instanceof Error ? err.message : String(err);
      }

    } catch (error) {
      debug.generalError = error instanceof Error ? error.message : String(error);
    }

    setDebugInfo(debug);
    setIsLoading(false);
  };

  useEffect(() => {
    runDebug();
  }, []);

  const resetToDefaultModel = () => {
    console.log('=== RESET TO DEFAULT MODEL ===');
    const defaultModel = {
      id: 'gpt-4o',
      name: 'GPT-4o',
      provider: 'OpenAI',
      description: 'Latest GPT-4 with vision and improved reasoning',
      contextWindow: 128000,
      capabilities: ['vision', 'tools', 'reasoning']
    };
    
    console.log('Before:', modelConfigStore.currentModel);
    modelConfigStore.setModel(defaultModel);
    console.log('After:', modelConfigStore.currentModel);
    
    setTimeout(() => {
      runDebug();
    }, 1000);
  };

  return (
    <div style={{ 
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'rgba(0,0,0,0.8)',
      color: '#ffffff',
      zIndex: 10000,
      overflow: 'auto',
      padding: '20px'
    }}>
      <div style={{ maxWidth: '800px', margin: '0 auto' }}>
        <h1>🔍 Debug Audit des Modèles</h1>
        
        <div style={{ marginBottom: '20px' }}>
          <button onClick={runDebug} disabled={isLoading} style={{ 
            padding: '10px 20px', 
            marginRight: '10px',
            background: '#3b82f6',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer'
          }}>
            {isLoading ? 'Chargement...' : 'Actualiser Debug'}
          </button>
          
          <button onClick={resetToDefaultModel} style={{ 
            padding: '10px 20px', 
            marginRight: '10px',
            background: '#16a34a',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer'
          }}>
            Reset to GPT-4o
          </button>
          
          <button onClick={onClose} style={{ 
            padding: '10px 20px',
            background: '#ef4444',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer'
          }}>
            Fermer
          </button>
        </div>

        <pre style={{ 
          background: '#1f2937',
          padding: '20px',
          borderRadius: '8px',
          overflow: 'auto',
          fontSize: '12px',
          lineHeight: '1.4'
        }}>
          {JSON.stringify(debugInfo, null, 2)}
        </pre>
      </div>
    </div>
  );
};