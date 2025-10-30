import React, { useState, useEffect } from 'react';
import { modelConfigStore, LiteLLMClient } from '../../lib/litellm';

interface Connection {
  id: string;
  name: string;
  baseUrl: string;
  apiKey: string;
  type: string;
  status?: 'active' | 'inactive' | 'error';
  lastPing?: number;
}

export const ConnectionsTab: React.FC = () => {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const [testMessage, setTestMessage] = useState('');

  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = () => {
    const savedConnections = modelConfigStore.activeConnections.map((conn: any) => ({
      ...conn,
      status: 'inactive' as const
    }));
    setConnections(savedConnections);
  };


  const saveConnections = (newConnections: Connection[]) => {
    const connectionsToSave = newConnections.map(({ status, lastPing, ...conn }) => conn);
    modelConfigStore.setActiveConnections(connectionsToSave);
    setConnections(newConnections);
  };

  const addConnection = (connection: Omit<Connection, 'id' | 'status'>) => {
    const newConnection: Connection = {
      ...connection,
      id: Date.now().toString(),
      status: 'inactive'
    };
    const updatedConnections = [...connections, newConnection];
    saveConnections(updatedConnections);
    setShowAddForm(false);
  };

  const updateConnection = (updatedConnection: Connection) => {
    const updatedConnections = connections.map(conn => 
      conn.id === updatedConnection.id ? updatedConnection : conn
    );
    saveConnections(updatedConnections);
    setEditingConnection(null);
  };

  const removeConnection = (id: string) => {
    const updatedConnections = connections.filter(conn => conn.id !== id);
    saveConnections(updatedConnections);
  };

  const testConnection = async (connection: Connection) => {
    setTestStatus('testing');
    setTestMessage('Test de connexion en cours...');
    
    try {
      const startTime = Date.now();
      const client = new LiteLLMClient({
        apiKey: connection.apiKey,
        baseUrl: connection.baseUrl,
        model: 'gpt-3.5-turbo' // modèle par défaut pour le test
      });
      
      // Test simple : demander la liste des modèles
      await client.getModels();
      const ping = Date.now() - startTime;
      
      setTestStatus('success');
      setTestMessage(`✅ Connexion réussie (${ping}ms)`);
      
      const updatedConnections = connections.map(conn => 
        conn.id === connection.id 
          ? { ...conn, status: 'active' as const, lastPing: ping }
          : conn
      );
      setConnections(updatedConnections);
    } catch (error) {
      setTestStatus('error');
      setTestMessage(`❌ Échec de connexion: ${error}`);
      
      const updatedConnections = connections.map(conn => 
        conn.id === connection.id 
          ? { ...conn, status: 'error' as const }
          : conn
      );
      setConnections(updatedConnections);
    }
  };

  return (
    <div style={{ 
      maxWidth: '1200px',
      margin: '0 auto'
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
            🔗 Connexions LiteLLM
          </h2>
          <p style={{ 
            color: '#9ca3af',
            margin: 0,
            fontSize: '14px'
          }}>
            Gérez vos connexions aux fournisseurs d'IA
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          style={{
            padding: '12px 24px',
            backgroundColor: '#3b82f6',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: '500'
          }}
        >
          {showAddForm ? '✕ Annuler' : '+ Ajouter'}
        </button>
      </div>

      {showAddForm && (
        <ConnectionForm 
          onSave={addConnection}
          onCancel={() => setShowAddForm(false)}
        />
      )}

      {editingConnection && (
        <ConnectionForm 
          connection={editingConnection}
          onSave={updateConnection}
          onCancel={() => setEditingConnection(null)}
        />
      )}

      <div style={{ display: 'grid', gap: '16px' }}>
        {connections.map((connection) => (
          <div key={connection.id} style={{
            background: 'rgba(255, 255, 255, 0.05)',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '12px',
            padding: '20px'
          }}>
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'space-between',
              marginBottom: '12px'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                <div style={{
                  width: '12px',
                  height: '12px',
                  borderRadius: '50%',
                  backgroundColor: 
                    connection.status === 'active' ? '#16a34a' :
                    connection.status === 'error' ? '#dc2626' : '#6b7280'
                }}></div>
                <h3 style={{ 
                  fontSize: '18px', 
                  margin: 0,
                  color: '#ffffff'
                }}>
                  {connection.name}
                  {modelConfigStore.selectedConnectionId === connection.id && (
                    <span style={{
                      marginLeft: '8px',
                      fontSize: '12px',
                      color: '#10b981',
                      fontWeight: 'bold'
                    }}>
                      ● ACTIVE
                    </span>
                  )}
                </h3>
                <span style={{
                  fontSize: '12px',
                  padding: '2px 8px',
                  backgroundColor: 'rgba(59, 130, 246, 0.2)',
                  color: '#60a5fa',
                  borderRadius: '4px'
                }}>
                  {connection.type}
                </span>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => testConnection(connection)}
                  style={{
                    padding: '8px 16px',
                    backgroundColor: '#3b82f6',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '12px'
                  }}
                >
                  🔍 Tester
                </button>
                <button
                  onClick={() => setEditingConnection(connection)}
                  style={{
                    padding: '8px 16px',
                    backgroundColor: '#10b981',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '12px'
                  }}
                >
                  ✏️ Modifier
                </button>
                <button
                  onClick={() => removeConnection(connection.id)}
                  style={{
                    padding: '8px 16px',
                    backgroundColor: '#dc2626',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '12px'
                  }}
                >
                  🗑️ Supprimer
                </button>
              </div>
            </div>
            <div style={{
              fontSize: '14px',
              color: '#9ca3af',
              marginBottom: '8px'
            }}>
              {connection.baseUrl}
            </div>
            {connection.lastPing && (
              <div style={{
                fontSize: '12px',
                color: '#6b7280'
              }}>
                Dernière réponse: {connection.lastPing}ms
              </div>
            )}
          </div>
        ))}
      </div>

      {testStatus !== 'idle' && (
        <div style={{
          marginTop: '16px',
          padding: '12px 16px',
          borderRadius: '8px',
          fontSize: '14px',
          backgroundColor: testStatus === 'success' ? 'rgba(22, 163, 74, 0.1)' :
                          testStatus === 'error' ? 'rgba(220, 38, 38, 0.1)' :
                          'rgba(59, 130, 246, 0.1)',
          border: `1px solid ${testStatus === 'success' ? '#16a34a' :
                              testStatus === 'error' ? '#dc2626' :
                              '#3b82f6'}`
        }}>
          {testMessage}
        </div>
      )}
    </div>
  );
};

interface ConnectionFormProps {
  connection?: Connection;
  onSave: (connection: any) => void;
  onCancel: () => void;
}

const ConnectionForm: React.FC<ConnectionFormProps> = ({ connection, onSave, onCancel }) => {
  const [name, setName] = useState(connection?.name || '');
  const [baseUrl, setBaseUrl] = useState(connection?.baseUrl || '');
  const [apiKey, setApiKey] = useState(connection?.apiKey || '');
  const [type, setType] = useState(connection?.type || 'LiteLLM');

  const handleSubmit = () => {
    if (name && baseUrl) {
      onSave(connection ? 
        { ...connection, name, baseUrl, apiKey, type } :
        { name, baseUrl, apiKey, type }
      );
    }
  };

  return (
    <div style={{
      background: 'rgba(59, 130, 246, 0.1)',
      border: '1px solid rgba(59, 130, 246, 0.3)',
      borderRadius: '12px',
      padding: '24px',
      marginBottom: '24px'
    }}>
      <h3 style={{ 
        fontSize: '18px', 
        marginBottom: '16px',
        color: '#ffffff'
      }}>
        {connection ? 'Modifier la connexion' : 'Nouvelle connexion'}
      </h3>
      <div style={{ display: 'grid', gap: '16px', gridTemplateColumns: '1fr 2fr 1fr', marginBottom: '16px' }}>
        <input
          type="text"
          placeholder="Nom de la connexion"
          value={name}
          onChange={(e) => setName(e.target.value)}
          style={{
            padding: '12px',
            borderRadius: '8px',
            border: '1px solid #374151',
            background: '#1f2937',
            color: '#ffffff',
            fontSize: '14px'
          }}
        />
        <input
          type="url"
          placeholder="URL de base (ex: http://localhost:4000)"
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          style={{
            padding: '12px',
            borderRadius: '8px',
            border: '1px solid #374151',
            background: '#1f2937',
            color: '#ffffff',
            fontSize: '14px'
          }}
        />
        <select
          value={type}
          onChange={(e) => setType(e.target.value)}
          style={{
            padding: '12px',
            borderRadius: '8px',
            border: '1px solid #374151',
            background: '#1f2937',
            color: '#ffffff',
            fontSize: '14px'
          }}
        >
          <option value="LiteLLM">LiteLLM</option>
          <option value="OpenAI">OpenAI Direct</option>
          <option value="Anthropic">Anthropic Direct</option>
          <option value="Custom">Custom API</option>
        </select>
      </div>
      <div style={{ marginBottom: '16px' }}>
        <input
          type="password"
          placeholder="Clé API (optionnel pour certains services)"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          style={{
            width: '100%',
            padding: '12px',
            borderRadius: '8px',
            border: '1px solid #374151',
            background: '#1f2937',
            color: '#ffffff',
            fontSize: '14px'
          }}
        />
      </div>
      <div style={{ display: 'flex', gap: '12px' }}>
        <button
          onClick={handleSubmit}
          disabled={!name || !baseUrl}
          style={{
            padding: '12px 24px',
            backgroundColor: name && baseUrl ? '#16a34a' : '#6b7280',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: name && baseUrl ? 'pointer' : 'not-allowed',
            fontSize: '14px',
            fontWeight: '500'
          }}
        >
          ✅ {connection ? 'Modifier' : 'Ajouter'}
        </button>
        <button
          onClick={onCancel}
          style={{
            padding: '12px 24px',
            backgroundColor: '#6b7280',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: '500'
          }}
        >
          ✕ Annuler
        </button>
      </div>
    </div>
  );
};