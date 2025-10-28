import React, { useState } from 'react';
import { Loader2, CheckCircle, XCircle, Wifi, TestTube, Play, Edit3, Trash2 } from 'lucide-react';
import { LiteLLMClient, modelConfigStore } from '@/lib/litellm';

interface SettingsWindowProps {
  onClose: () => void;
}

interface Connection {
  id: string;
  name: string;
  baseUrl: string;
  apiKey: string;
  isActive: boolean;
}

export const SettingsWindow: React.FC<SettingsWindowProps> = () => {
  const [connections, setConnections] = useState<Connection[]>([
    {
      id: '1',
      name: 'LiteLLM Local',
      baseUrl: modelConfigStore.baseUrl || 'http://localhost:4000',
      apiKey: modelConfigStore.apiKey || '',
      isActive: true
    }
  ]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newConnection, setNewConnection] = useState({ name: '', baseUrl: '', apiKey: '' });
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const [testMessage, setTestMessage] = useState('');
  const [testingConnectionId, setTestingConnectionId] = useState<string | null>(null);

  const handleAddConnection = () => {
    if (!newConnection.name || !newConnection.baseUrl) return;
    
    const connection: Connection = {
      id: Date.now().toString(),
      name: newConnection.name,
      baseUrl: newConnection.baseUrl,
      apiKey: newConnection.apiKey,
      isActive: false
    };
    
    setConnections([...connections, connection]);
    setNewConnection({ name: '', baseUrl: '', apiKey: '' });
    setShowAddForm(false);
  };

  const handleActivateConnection = (connectionId: string) => {
    setConnections(prev => prev.map(conn => ({
      ...conn,
      isActive: conn.id === connectionId
    })));
    
    const activeConnection = connections.find(c => c.id === connectionId);
    if (activeConnection) {
      modelConfigStore.setApiKey(activeConnection.apiKey);
      modelConfigStore.setBaseUrl(activeConnection.baseUrl);
    }
  };

  const handleTestConnection = async (connection: Connection) => {
    if (!connection.baseUrl.trim()) {
      setTestStatus('error');
      setTestMessage('URL requise');
      return;
    }

    setTestingConnectionId(connection.id);
    setTestStatus('testing');
    setTestMessage('Test en cours...');

    try {
      const testClient = new LiteLLMClient({
        apiKey: connection.apiKey || 'test',
        baseUrl: connection.baseUrl.trim(),
        model: 'test'
      });

      await testClient.getModels();
      
      setTestStatus('success');
      setTestMessage('Connexion réussie !');
      
      setTimeout(() => {
        setTestStatus('idle');
        setTestMessage('');
        setTestingConnectionId(null);
      }, 3000);
      
    } catch (error) {
      setTestStatus('error');
      const errorMsg = error instanceof Error ? error.message : 'Erreur de connexion';
      setTestMessage(errorMsg);
      
      setTimeout(() => {
        setTestStatus('idle');
        setTestMessage('');
        setTestingConnectionId(null);
      }, 5000);
    }
  };

  const handleDeleteConnection = (connectionId: string) => {
    setConnections(prev => prev.filter(conn => conn.id !== connectionId));
  };

  const handleEditConnection = (connection: Connection) => {
    setEditingConnection({ ...connection });
  };

  const handleSaveEdit = () => {
    if (!editingConnection || !editingConnection.name.trim() || !editingConnection.baseUrl.trim()) return;
    
    setConnections(prev => prev.map(conn => 
      conn.id === editingConnection.id ? editingConnection : conn
    ));
    
    // Si on édite la connexion active, mettre à jour le store
    if (editingConnection.isActive) {
      modelConfigStore.setApiKey(editingConnection.apiKey);
      modelConfigStore.setBaseUrl(editingConnection.baseUrl);
    }
    
    setEditingConnection(null);
  };

  const handleCancelEdit = () => {
    setEditingConnection(null);
  };

  const getTestIcon = () => {
    switch (testStatus) {
      case 'testing':
        return <Loader2 size={14} className="animate-spin" />;
      case 'success':
        return <CheckCircle size={14} />;
      case 'error':
        return <XCircle size={14} />;
      default:
        return <Wifi size={14} />;
    }
  };

  const getTestText = () => {
    if (testMessage) return testMessage;
    return 'Tester';
  };

  console.log('SettingsWindow rendering');

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

      {/* Content */}
      <div style={{ 
        flex: 1, 
        padding: '24px',
        overflow: 'auto',
        background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f1629 100%)'
      }}>
        <div style={{ 
          maxWidth: '1200px',
          margin: '0 auto'
        }}>
          
          {/* Header avec bouton d'ajout */}
          <div style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'space-between',
            marginBottom: '24px'
          }}>
            <h2 style={{ 
              fontSize: '24px', 
              fontWeight: '600', 
              color: '#ffffff', 
              margin: 0,
              display: 'flex',
              alignItems: 'center',
              gap: '12px'
            }}>
              <Wifi size={24} />
              Connexions
            </h2>
            <button 
              onClick={() => setShowAddForm(true)}
              disabled={!!editingConnection}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '8px 16px',
                background: editingConnection ? '#6b7280' : '#3b82f6',
                color: '#ffffff',
                border: 'none',
                borderRadius: '6px',
                cursor: editingConnection ? 'not-allowed' : 'pointer',
                transition: 'background-color 0.2s',
                fontSize: '14px',
                fontWeight: '500',
                opacity: editingConnection ? 0.6 : 1
              }}
              onMouseEnter={(e) => {
                if (!editingConnection) {
                  e.currentTarget.style.backgroundColor = '#2563eb';
                }
              }}
              onMouseLeave={(e) => {
                if (!editingConnection) {
                  e.currentTarget.style.backgroundColor = '#3b82f6';
                }
              }}
            >
              + Ajouter une connexion
            </button>
          </div>

          {/* Tableau des connexions */}
          <div style={{
            background: 'rgba(31, 41, 55, 0.5)',
            backdropFilter: 'blur(12px)',
            border: '1px solid #374151',
            borderRadius: '12px',
            overflow: 'hidden'
          }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ background: 'rgba(55, 65, 81, 0.5)' }}>
                  <th style={{ padding: '16px', textAlign: 'left', color: '#d1d5db', fontWeight: '600' }}>Nom</th>
                  <th style={{ padding: '16px', textAlign: 'left', color: '#d1d5db', fontWeight: '600' }}>URL</th>
                  <th style={{ padding: '16px', textAlign: 'left', color: '#d1d5db', fontWeight: '600' }}>Status</th>
                  <th style={{ padding: '16px', textAlign: 'center', color: '#d1d5db', fontWeight: '600' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {connections.map((connection) => (
                  <tr key={connection.id} style={{ borderTop: '1px solid #374151' }}>
                    <td style={{ padding: '16px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span style={{ color: '#ffffff', fontWeight: '500' }}>{connection.name}</span>
                        {connection.isActive && (
                          <span style={{
                            padding: '2px 6px',
                            background: '#16a34a',
                            color: '#ffffff',
                            fontSize: '10px',
                            borderRadius: '4px',
                            fontWeight: '500'
                          }}>
                            actif
                          </span>
                        )}
                      </div>
                    </td>
                    <td style={{ padding: '16px', color: '#9ca3af', fontSize: '14px' }}>
                      {connection.baseUrl}
                    </td>
                    <td style={{ padding: '16px' }}>
                      {testingConnectionId === connection.id ? (
                        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                          {getTestIcon()}
                          <span style={{ color: '#93c5fd', fontSize: '14px' }}>{getTestText()}</span>
                        </div>
                      ) : (
                        <span style={{ 
                          color: connection.isActive ? '#86efac' : '#6b7280',
                          fontSize: '14px'
                        }}>
                          {connection.isActive ? 'Connecté' : 'Inactif'}
                        </span>
                      )}
                    </td>
                    <td style={{ padding: '16px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px' }}>
                        <button 
                          onClick={() => handleTestConnection(connection)}
                          disabled={testingConnectionId === connection.id}
                          title="Tester la connexion"
                          style={{
                            padding: '6px',
                            background: '#4b5563',
                            color: '#ffffff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: testingConnectionId === connection.id ? 'not-allowed' : 'pointer',
                            transition: 'background-color 0.2s',
                            display: 'flex',
                            alignItems: 'center',
                            opacity: testingConnectionId === connection.id ? 0.6 : 1
                          }}
                          onMouseEnter={(e) => {
                            if (testingConnectionId !== connection.id) {
                              e.currentTarget.style.backgroundColor = '#6b7280';
                            }
                          }}
                          onMouseLeave={(e) => {
                            e.currentTarget.style.backgroundColor = '#4b5563';
                          }}
                        >
                          <TestTube size={14} />
                        </button>
                        {!connection.isActive && (
                          <button 
                            onClick={() => handleActivateConnection(connection.id)}
                            title="Activer cette connexion"
                            style={{
                              padding: '6px',
                              background: '#16a34a',
                              color: '#ffffff',
                              border: 'none',
                              borderRadius: '4px',
                              cursor: 'pointer',
                              transition: 'background-color 0.2s',
                              display: 'flex',
                              alignItems: 'center'
                            }}
                            onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#15803d'}
                            onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#16a34a'}
                          >
                            <Play size={14} />
                          </button>
                        )}
                        <button 
                          onClick={() => handleEditConnection(connection)}
                          title="Modifier cette connexion"
                          style={{
                            padding: '6px',
                            background: '#3b82f6',
                            color: '#ffffff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            transition: 'background-color 0.2s',
                            display: 'flex',
                            alignItems: 'center'
                          }}
                          onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#2563eb'}
                          onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#3b82f6'}
                        >
                          <Edit3 size={14} />
                        </button>
                        <button 
                          onClick={() => handleDeleteConnection(connection.id)}
                          title="Supprimer cette connexion"
                          style={{
                            padding: '6px',
                            background: '#dc2626',
                            color: '#ffffff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            transition: 'background-color 0.2s',
                            display: 'flex',
                            alignItems: 'center'
                          }}
                          onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#b91c1c'}
                          onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#dc2626'}
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Formulaire d'édition */}
          {editingConnection && (
            <div style={{
              marginTop: '24px',
              background: 'rgba(31, 41, 55, 0.5)',
              backdropFilter: 'blur(12px)',
              border: '1px solid #374151',
              borderRadius: '12px',
              padding: '24px'
            }}>
              <h3 style={{ 
                fontSize: '18px', 
                fontWeight: '600', 
                color: '#ffffff', 
                margin: '0 0 16px 0' 
              }}>
                Modifier la connexion
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr 1fr', gap: '16px', marginBottom: '16px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    Nom <span style={{ color: '#ef4444' }}>*</span>
                  </label>
                  <input
                    type="text"
                    value={editingConnection.name}
                    onChange={(e) => setEditingConnection(prev => prev ? { ...prev, name: e.target.value } : null)}
                    placeholder="Nom de la connexion..."
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: `1px solid ${!editingConnection.name.trim() ? '#ef4444' : '#4b5563'}`,
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                    onFocus={(e) => e.target.style.borderColor = '#3b82f6'}
                    onBlur={(e) => e.target.style.borderColor = !editingConnection.name.trim() ? '#ef4444' : '#4b5563'}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    Base URL <span style={{ color: '#ef4444' }}>*</span>
                  </label>
                  <input
                    type="text"
                    value={editingConnection.baseUrl}
                    onChange={(e) => setEditingConnection(prev => prev ? { ...prev, baseUrl: e.target.value } : null)}
                    placeholder="http://localhost:4000"
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: `1px solid ${!editingConnection.baseUrl.trim() ? '#ef4444' : '#4b5563'}`,
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                    onFocus={(e) => e.target.style.borderColor = '#3b82f6'}
                    onBlur={(e) => e.target.style.borderColor = !editingConnection.baseUrl.trim() ? '#ef4444' : '#4b5563'}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    API Key
                  </label>
                  <input
                    type="password"
                    value={editingConnection.apiKey}
                    onChange={(e) => setEditingConnection(prev => prev ? { ...prev, apiKey: e.target.value } : null)}
                    placeholder="sk-..."
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: '1px solid #4b5563',
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                  />
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button 
                  onClick={handleSaveEdit}
                  disabled={!editingConnection.name.trim() || !editingConnection.baseUrl.trim()}
                  style={{
                    padding: '8px 16px',
                    background: (!editingConnection.name.trim() || !editingConnection.baseUrl.trim()) ? '#6b7280' : '#16a34a',
                    color: '#ffffff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: (!editingConnection.name.trim() || !editingConnection.baseUrl.trim()) ? 'not-allowed' : 'pointer',
                    fontSize: '14px',
                    fontWeight: '500',
                    opacity: (!editingConnection.name.trim() || !editingConnection.baseUrl.trim()) ? 0.6 : 1,
                    transition: 'all 0.2s',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px'
                  }}
                >
                  <CheckCircle size={14} />
                  Sauvegarder
                </button>
                <button 
                  onClick={handleCancelEdit}
                  style={{
                    padding: '8px 16px',
                    background: '#6b7280',
                    color: '#ffffff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: '500',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px'
                  }}
                >
                  <XCircle size={14} />
                  Annuler
                </button>
              </div>
            </div>
          )}

          {/* Formulaire d'ajout */}
          {showAddForm && !editingConnection && (
            <div style={{
              marginTop: '24px',
              background: 'rgba(31, 41, 55, 0.5)',
              backdropFilter: 'blur(12px)',
              border: '1px solid #374151',
              borderRadius: '12px',
              padding: '24px'
            }}>
              <h3 style={{ 
                fontSize: '18px', 
                fontWeight: '600', 
                color: '#ffffff', 
                margin: '0 0 16px 0' 
              }}>
                Nouvelle connexion
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr 1fr', gap: '16px', marginBottom: '16px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    Nom <span style={{ color: '#ef4444' }}>*</span>
                  </label>
                  <input
                    type="text"
                    value={newConnection.name}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="Nom de la connexion..."
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: `1px solid ${!newConnection.name.trim() ? '#ef4444' : '#4b5563'}`,
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                    onFocus={(e) => e.target.style.borderColor = '#3b82f6'}
                    onBlur={(e) => e.target.style.borderColor = !newConnection.name.trim() ? '#ef4444' : '#4b5563'}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    Base URL <span style={{ color: '#ef4444' }}>*</span>
                  </label>
                  <input
                    type="text"
                    value={newConnection.baseUrl}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, baseUrl: e.target.value }))}
                    placeholder="http://localhost:4000"
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: `1px solid ${!newConnection.baseUrl.trim() ? '#ef4444' : '#4b5563'}`,
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                    onFocus={(e) => e.target.style.borderColor = '#3b82f6'}
                    onBlur={(e) => e.target.style.borderColor = !newConnection.baseUrl.trim() ? '#ef4444' : '#4b5563'}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '14px', color: '#d1d5db', marginBottom: '4px' }}>
                    API Key
                  </label>
                  <input
                    type="password"
                    value={newConnection.apiKey}
                    onChange={(e) => setNewConnection(prev => ({ ...prev, apiKey: e.target.value }))}
                    placeholder="sk-..."
                    style={{
                      width: '100%',
                      background: '#374151',
                      border: '1px solid #4b5563',
                      borderRadius: '6px',
                      padding: '8px',
                      color: '#ffffff',
                      fontSize: '14px',
                      outline: 'none'
                    }}
                  />
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button 
                  onClick={handleAddConnection}
                  disabled={!newConnection.name.trim() || !newConnection.baseUrl.trim()}
                  style={{
                    padding: '8px 16px',
                    background: (!newConnection.name.trim() || !newConnection.baseUrl.trim()) ? '#6b7280' : '#16a34a',
                    color: '#ffffff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: (!newConnection.name.trim() || !newConnection.baseUrl.trim()) ? 'not-allowed' : 'pointer',
                    fontSize: '14px',
                    fontWeight: '500',
                    opacity: (!newConnection.name.trim() || !newConnection.baseUrl.trim()) ? 0.6 : 1,
                    transition: 'all 0.2s'
                  }}
                  onMouseEnter={(e) => {
                    if (newConnection.name.trim() && newConnection.baseUrl.trim()) {
                      e.currentTarget.style.backgroundColor = '#15803d';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (newConnection.name.trim() && newConnection.baseUrl.trim()) {
                      e.currentTarget.style.backgroundColor = '#16a34a';
                    }
                  }}
                >
                  Ajouter
                </button>
                <button 
                  onClick={() => setShowAddForm(false)}
                  style={{
                    padding: '8px 16px',
                    background: '#6b7280',
                    color: '#ffffff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: '500'
                  }}
                >
                  Annuler
                </button>
              </div>
            </div>
          )}

          {/* Message de test global */}
          {testMessage && testingConnectionId && (
            <div style={{
              marginTop: '24px',
              padding: '12px',
              borderRadius: '6px',
              fontSize: '14px',
              background: testStatus === 'success' ? 'rgba(22, 163, 74, 0.2)' :
                         testStatus === 'error' ? 'rgba(220, 38, 38, 0.2)' :
                         'rgba(59, 130, 246, 0.2)',
              color: testStatus === 'success' ? '#86efac' :
                     testStatus === 'error' ? '#fca5a5' :
                     '#93c5fd',
              border: `1px solid ${testStatus === 'success' ? '#16a34a' :
                                  testStatus === 'error' ? '#dc2626' :
                                  '#3b82f6'}`
            }}>
              {testMessage}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};