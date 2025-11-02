// GRAVIS Extension - Popup Script
// Phase 0 Spike - UI Handlers

console.log('🎛️ GRAVIS Extension popup loaded');

class PopupController {
  constructor() {
    this.isProcessing = false;
    this.gravisConnected = false;
    this.currentTab = null;
    
    this.initializeElements();
    this.setupEventListeners();
    this.checkGRAVISConnection();
    this.getCurrentTab();
  }

  initializeElements() {
    this.elements = {
      status: document.getElementById('status'),
      statusText: document.getElementById('status-text'),
      extractPage: document.getElementById('extract-page'),
      extractSelection: document.getElementById('extract-selection'),
      settings: document.getElementById('settings'),
      help: document.getElementById('help'),
      modeIndicator: document.getElementById('mode-indicator')
    };
  }

  setupEventListeners() {
    this.elements.extractPage.addEventListener('click', () => {
      this.handleExtractPage();
    });

    this.elements.extractSelection.addEventListener('click', () => {
      this.handleExtractSelection();
    });

    this.elements.settings.addEventListener('click', () => {
      this.handleSettings();
    });

    this.elements.help.addEventListener('click', () => {
      this.handleHelp();
    });

    // Refresh connection status every 5 seconds
    setInterval(() => {
      if (!this.isProcessing) {
        this.checkGRAVISConnection();
      }
    }, 5000);
  }

  async getCurrentTab() {
    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      this.currentTab = tab;
      this.updateUIForCurrentTab();
    } catch (error) {
      console.error('❌ Failed to get current tab:', error);
    }
  }

  updateUIForCurrentTab() {
    if (!this.currentTab) return;

    const url = this.currentTab.url;
    console.log('🔍 Checking current tab URL:', url);
    
    const isPDF = url.includes('.pdf');
    const isSpecialPage = url.startsWith('chrome://') || 
                         url.startsWith('chrome-extension://') ||
                         url.startsWith('moz-extension://') ||
                         url.startsWith('edge://') ||
                         url.startsWith('about:');

    console.log('📋 Page analysis:', { isPDF, isSpecialPage, gravisConnected: this.gravisConnected });

    // Update mode indicator
    if (isPDF) {
      this.elements.modeIndicator.textContent = 'PDF';
    } else if (isSpecialPage) {
      this.elements.modeIndicator.textContent = 'Limited';
    } else {
      this.elements.modeIndicator.textContent = 'Smart';
    }

    // Disable buttons for special pages
    if (isSpecialPage) {
      console.log('🚫 Disabling buttons for special page');
      this.setButtonsEnabled(false);
      this.updateStatus('disconnected', '❌ Cannot extract from this page type');
    } else if (this.gravisConnected) {
      console.log('✅ Enabling buttons for normal page');
      this.setButtonsEnabled(true);
      this.updateStatus('connected', '✅ GRAVIS Connected');
    } else {
      console.log('📡 Page is normal but GRAVIS not connected');
      this.setButtonsEnabled(false);
    }
  }

  async checkGRAVISConnection() {
    this.updateStatus('checking', '🔄 Checking GRAVIS...');

    try {
      const response = await chrome.runtime.sendMessage({ type: 'PING_GRAVIS' });
      
      if (response?.connected) {
        this.gravisConnected = true;
        console.log('✅ GRAVIS connected, updating UI');
      } else {
        this.gravisConnected = false;
        console.log('❌ GRAVIS not connected');
      }
    } catch (error) {
      console.error('❌ Connection check failed:', error);
      this.gravisConnected = false;
    }
    
    // Always update UI based on current tab and connection status
    this.updateUIForCurrentTab();
  }

  updateStatus(type, message) {
    this.elements.status.className = `status ${type}`;
    this.elements.statusText.textContent = message;
  }

  setButtonsEnabled(enabled) {
    console.log('🎛️ Setting buttons enabled:', enabled);
    const buttons = [this.elements.extractPage, this.elements.extractSelection];
    buttons.forEach((button, index) => {
      if (button) {
        button.disabled = !enabled;
        console.log(`Button ${index} (${button.textContent}) disabled: ${!enabled}`);
      }
    });
  }

  async handleExtractPage() {
    console.log('🚀 handleExtractPage called');
    if (this.isProcessing || !this.gravisConnected) {
      console.log('❌ Cannot extract: processing=', this.isProcessing, 'connected=', this.gravisConnected);
      return;
    }

    this.startProcessing('📄 Extracting page content...');

    try {
      console.log('🔄 Starting page extraction...');
      const result = await this.executeExtraction('page');
      console.log('✅ Page extraction completed:', result);
      this.showSuccess('✅ Page content sent to GRAVIS!');
    } catch (error) {
      console.error('❌ Page extraction failed:', error);
      console.error('Error details:', error.stack);
      this.showError('❌ ' + error.message);
    } finally {
      this.stopProcessing();
    }
  }

  async handleExtractSelection() {
    console.log('🚀 handleExtractSelection called');
    if (this.isProcessing || !this.gravisConnected) {
      console.log('❌ Cannot extract: processing=', this.isProcessing, 'connected=', this.gravisConnected);
      return;
    }

    this.startProcessing('✂️ Extracting selection...');

    try {
      console.log('🔄 Starting selection extraction...');
      const result = await this.executeExtraction('selection');
      console.log('✅ Selection extraction completed:', result);
      this.showSuccess('✅ Selection sent to GRAVIS!');
    } catch (error) {
      console.error('❌ Selection extraction failed:', error);
      console.error('Error details:', error.stack);
      this.showError('❌ ' + error.message);
    } finally {
      this.stopProcessing();
    }
  }

  async executeExtraction(mode) {
    console.log('🔧 executeExtraction called with mode:', mode);
    if (!this.currentTab) {
      throw new Error('No active tab found');
    }
    console.log('📋 Current tab:', this.currentTab.url);

    // Check if current page allows content script injection
    const url = this.currentTab.url;
    const isSpecialPage = url.startsWith('chrome://') || 
                         url.startsWith('chrome-extension://') ||
                         url.startsWith('moz-extension://') ||
                         url.startsWith('edge://') ||
                         url.startsWith('about:');
    
    if (isSpecialPage) {
      throw new Error('Cannot extract from this page type. Please navigate to a regular website (http:// or https://)');
    }

    console.log('🚀 Using inline extraction method to avoid script injection issues');

    // Execute extraction in content script using code injection
    const results = await chrome.scripting.executeScript({
      target: { tabId: this.currentTab.id },
      function: function(mode) {
        console.log('🔍 Running inline extraction, mode:', mode);
        
        try {
          // Simple content extraction logic
          const selection = window.getSelection()?.toString()?.trim();
          let content, method;
          
          if (selection && selection.length > 50) {
            console.log('✂️ Using user selection');
            content = selection;
            method = 'user_selection';
          } else {
            // Try to get main content
            const candidates = [
              document.querySelector('main'),
              document.querySelector('article'),
              document.querySelector('[role="main"]'),
              document.querySelector('.content'),
              document.querySelector('#content'),
              document.querySelector('.post'),
              document.querySelector('.article')
            ];
            
            const bestElement = candidates.find(el => el && el.textContent && el.textContent.length > 200);
            
            if (bestElement) {
              content = bestElement.textContent.trim();
              method = 'heuristic';
            } else {
              content = document.body.textContent.trim().slice(0, 10000);
              method = 'body_fallback';
            }
          }
          
          console.log('📄 Extracted content:', { method, length: content.length });
          
          // Send to background script
          return chrome.runtime.sendMessage({
            type: 'GRAVIS_EXTRACT',
            payload: {
              url: window.location.href,
              title: document.title,
              mainContent: content,
              selectedText: selection || null,
              extraction_method: 'extension_' + method,
              metadata: {
                method: method,
                contentLength: content.length,
                timestamp: Date.now()
              },
              timestamp: Date.now()
            }
          }).then(response => {
            if (response?.ok) {
              console.log('✅ Content sent to GRAVIS successfully');
              return { ok: true, method: method, contentLength: content.length };
            } else {
              throw new Error(response?.error || 'Failed to send content');
            }
          }).catch(error => {
            console.error('❌ Send error:', error);
            return { error: error.message };
          });
          
        } catch (error) {
          console.error('❌ Inline extraction failed:', error);
          throw new Error('Content extraction failed: ' + error.message);
        }
      },
      args: [mode]
    });

    if (!results || !results[0]) {
      throw new Error('Extraction script failed to execute');
    }

    return results[0].result;
  }

  // Get the extraction function reference  
  getExtractionFunction() {
    return extractContentFunction;
  }

  handleSettings() {
    // Open options page (to be implemented)
    chrome.tabs.create({ 
      url: chrome.runtime.getURL('options.html') 
    });
  }

  handleHelp() {
    // Open help page
    chrome.tabs.create({ 
      url: 'https://docs.gravis.ai/extension-help' 
    });
  }

  startProcessing(message) {
    this.isProcessing = true;
    document.body.classList.add('processing');
    this.updateStatus('checking', message);
    this.setButtonsEnabled(false);
  }

  stopProcessing() {
    this.isProcessing = false;
    document.body.classList.remove('processing');
    this.setButtonsEnabled(this.gravisConnected);
    
    // Restore original status
    setTimeout(() => {
      if (this.gravisConnected) {
        this.updateStatus('connected', '✅ GRAVIS Connected');
      } else {
        this.updateStatus('disconnected', '❌ GRAVIS Not Detected');
      }
    }, 2000);
  }

  showSuccess(message) {
    this.updateStatus('connected', message);
  }

  showError(message) {
    this.updateStatus('disconnected', message);
  }
}

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new PopupController();
});

// Handle popup closing
window.addEventListener('beforeunload', () => {
  console.log('🎛️ GRAVIS Extension popup closing');
});

// Extension ready
console.log('✅ GRAVIS Extension popup script loaded');