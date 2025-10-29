import { useState, useEffect } from "react";
import { ChevronDown, Check, Settings } from "lucide-react";
import { AVAILABLE_MODELS, LLMModel, modelConfigStore } from "@/lib/litellm";

interface ModelSelectorProps {
  onModelChange?: (model: LLMModel) => void;
  compact?: boolean;
}

export function ModelSelector({ onModelChange, compact = false }: ModelSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedModel, setSelectedModel] = useState<LLMModel>(modelConfigStore.currentModel);
  const [showSettings, setShowSettings] = useState(false);

  // Écouter les changements du modèle dans le store
  useEffect(() => {
    const checkModelUpdate = () => {
      if (modelConfigStore.currentModel.id !== selectedModel.id) {
        console.log('ModelSelector: Model changed externally', {
          old: selectedModel.id,
          new: modelConfigStore.currentModel.id
        });
        setSelectedModel(modelConfigStore.currentModel);
        onModelChange?.(modelConfigStore.currentModel);
      }
    };

    // Vérifier périodiquement si le modèle a changé
    const interval = setInterval(checkModelUpdate, 1000);
    
    return () => clearInterval(interval);
  }, [selectedModel.id, onModelChange]);

  const handleModelSelect = (model: LLMModel) => {
    setSelectedModel(model);
    modelConfigStore.setModel(model);
    onModelChange?.(model);
    setIsOpen(false);
  };

  const toggleSettings = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowSettings(!showSettings);
  };

  if (compact) {
    return (
      <div className="relative">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="icon-button"
          title={`Model: ${selectedModel.name}`}
        >
          <span style={{ fontSize: "10px", fontWeight: "600" }}>
            {selectedModel.name.split(" ")[0]}
          </span>
        </button>

        {isOpen && (
          <>
            <div 
              className="fixed inset-0 z-10" 
              onClick={() => setIsOpen(false)}
            />
            <div className="model-dropdown compact">
              {AVAILABLE_MODELS.map((model) => (
                <div
                  key={model.id}
                  className={`model-option ${selectedModel.id === model.id ? "selected" : ""}`}
                  onClick={() => handleModelSelect(model)}
                >
                  <div className="model-info">
                    <div className="model-name">{model.name}</div>
                    <div className="model-provider">{model.provider}</div>
                  </div>
                  {selectedModel.id === model.id && (
                    <Check size={12} className="check-icon" />
                  )}
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <div className="model-selector">
      <div className="model-selector-header">
        <button
          className="model-selector-button"
          onClick={() => setIsOpen(!isOpen)}
        >
          <div className="selected-model">
            <div className="model-name">{selectedModel.name}</div>
            <div className="model-provider">{selectedModel.provider}</div>
          </div>
          <ChevronDown size={16} className={`chevron ${isOpen ? "open" : ""}`} />
        </button>
        
        <button className="settings-button" onClick={toggleSettings}>
          <Settings size={14} />
        </button>
      </div>

      {isOpen && (
        <>
          <div className="dropdown-overlay" onClick={() => setIsOpen(false)} />
          <div className="model-dropdown">
            {AVAILABLE_MODELS.map((model) => (
              <div
                key={model.id}
                className={`model-option ${selectedModel.id === model.id ? "selected" : ""}`}
                onClick={() => handleModelSelect(model)}
              >
                <div className="model-info">
                  <div className="model-name">{model.name}</div>
                  <div className="model-description">{model.description}</div>
                  <div className="model-meta">
                    <span className="provider">{model.provider}</span>
                    <span className="context">{model.contextWindow.toLocaleString()} ctx</span>
                    {model.pricing && (
                      <span className="pricing">
                        ${model.pricing.input.toFixed(4)}/1K in
                      </span>
                    )}
                  </div>
                </div>
                {selectedModel.id === model.id && (
                  <Check size={16} className="check-icon" />
                )}
              </div>
            ))}
          </div>
        </>
      )}

      {showSettings && (
        <ModelSettings onClose={() => setShowSettings(false)} />
      )}
    </div>
  );
}

function ModelSettings({ onClose }: { onClose: () => void }) {
  const [apiKey, setApiKey] = useState(modelConfigStore.apiKey);
  const [baseUrl, setBaseUrl] = useState(modelConfigStore.baseUrl);

  const handleSave = () => {
    modelConfigStore.setApiKey(apiKey);
    modelConfigStore.setBaseUrl(baseUrl);
    onClose();
  };

  return (
    <>
      <div className="dropdown-overlay" onClick={onClose} />
      <div className="settings-modal">
        <div className="settings-header">
          <h3>Configuration LiteLLM</h3>
          <button onClick={onClose} className="close-button">×</button>
        </div>
        
        <div className="settings-content">
          <div className="setting-group">
            <label>Base URL</label>
            <input
              type="text"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="http://localhost:4000"
              className="setting-input"
            />
          </div>
          
          <div className="setting-group">
            <label>API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              className="setting-input"
            />
          </div>
        </div>
        
        <div className="settings-footer">
          <button onClick={onClose} className="cancel-button">
            Annuler
          </button>
          <button onClick={handleSave} className="save-button">
            Sauvegarder
          </button>
        </div>
      </div>
    </>
  );
}