// GRAVIS Extension - Service Worker Background Script
// Phase 0 Spike - Production-Ready Security

console.log('🚀 GRAVIS Extension Service Worker loaded');

// Keepalive pour éviter que le SW s'endorme (MV3)
chrome.runtime.onInstalled.addListener(() => {
  chrome.alarms.create('keepalive', { periodInMinutes: 1 });
  console.log('⏰ Keepalive alarm created');
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'keepalive') {
    console.log('🔄 Service Worker keepalive ping');
  }
});

// Rate limiting simple
const rateLimiter = {
  requests: new Map(),
  limit: 10, // req/min
  window: 60000, // 1 minute

  isAllowed(clientId = 'default') {
    const now = Date.now();
    const requests = this.requests.get(clientId) || [];
    
    // Nettoyer les requêtes anciennes
    const recent = requests.filter(time => now - time < this.window);
    
    if (recent.length >= this.limit) {
      console.warn('⚠️ Rate limit exceeded for', clientId);
      return false;
    }
    
    recent.push(now);
    this.requests.set(clientId, recent);
    return true;
  }
};

// Classe pour payload sécurisé avec HMAC
class SecurePayloadSender {
  constructor() {
    this.secret = null;
    this.nonce_cache = new Set();
    this.apiUrl = 'http://127.0.0.1:8766/api/extension';
  }

  async getSecret() {
    if (!this.secret) {
      try {
        const response = await fetch(`${this.apiUrl}/ping`);
        const data = await response.json();
        this.secret = data.token;
        console.log('🔑 Secret token acquired from GRAVIS');
      } catch (error) {
        console.error('❌ Failed to get secret token:', error);
        throw new Error('GRAVIS connection failed');
      }
    }
    return this.secret;
  }

  generateNonce() {
    return Array.from(crypto.getRandomValues(new Uint8Array(16)))
      .map(b => b.toString(16).padStart(2, '0')).join('');
  }

  async createSecurePayload(content) {
    const secret = await this.getSecret();
    const nonce = this.generateNonce();
    const ts = Date.now();

    const payload = {
      "v": "1",
      "nonce": nonce,
      "ts": ts,
      "title": content.title || 'Untitled',
      "url": content.url || '',
      "main_content": content.mainContent || '',
      "selected_text": content.selectedText || null,
      "extraction_method": content.extraction_method || 'extension_dom',
      "flags": {
        "is_pdf": this.isPDF(content.url, content.mainContent),
        "is_paywalled": this.isPaywalled(content.mainContent)
      }
    };

    // Canonical body pour signature - DOIT matcher le serveur Rust exactement
    // Rust serde_json utilise un ordre alphabétique pour les clés
    const canonicalPayload = {
      extraction_method: payload.extraction_method,
      flags: {
        // Ordre alphabétique pour matcher Rust serde_json
        is_paywalled: payload.flags.is_paywalled,
        is_pdf: payload.flags.is_pdf
      },
      main_content: payload.main_content,
      nonce: payload.nonce,
      selected_text: payload.selected_text,
      title: payload.title,
      ts: payload.ts,
      url: payload.url,
      v: payload.v
    };
    // Pas d'espaces pour matcher serde_json de Rust
    const canonical = JSON.stringify(canonicalPayload, null, 0);
    console.log('🔍 DEBUG - Canonical payload for signature (len=' + canonical.length + '):', canonical.substring(0, 300) + '...');
    console.log('🔑 DEBUG - Using secret (len=' + secret.length + '):', secret.substring(0, 10) + '...');
    
    const sig = await this.signPayload(canonical, secret);
    console.log('📝 DEBUG - Generated signature:', sig);
    payload.sig = sig;

    return payload;
  }

  async signPayload(data, secret) {
    const encoder = new TextEncoder();
    
    // Essayons d'abord avec le secret comme string UTF-8 (pas décodé)
    // car le serveur Rust utilise peut-être secret.as_bytes() directement
    const key = await crypto.subtle.importKey(
      'raw',
      encoder.encode(secret),  // Utiliser le secret comme string UTF-8
      { name: 'HMAC', hash: 'SHA-256' },
      false,
      ['sign']
    );
    const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(data));
    return btoa(String.fromCharCode(...new Uint8Array(signature)));
  }

  isPDF(url, content) {
    return (url && url.includes('.pdf')) || 
           (content && content.includes('PDF'));
  }

  isPaywalled(content) {
    if (!content) return false;
    
    const paywallIndicators = [
      'Subscribe to continue', 'Login to read', 'Premium content',
      'Subscribe for unlimited access', 'Upgrade to read',
      'paywall', 'subscription required', 'members only'
    ];
    
    const lowerContent = content.toLowerCase();
    return paywallIndicators.some(indicator => 
      lowerContent.includes(indicator.toLowerCase())
    );
  }

  async sendToGRAVIS(payload) {
    try {
      const response = await fetch(`${this.apiUrl}/content`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Gravis-Ext': 'v1'
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      console.log('✅ Content sent to GRAVIS successfully:', result);
      return result;
    } catch (error) {
      console.error('❌ Failed to send content to GRAVIS:', error);
      throw error;
    }
  }
}

// Instance globale
const payloadSender = new SecurePayloadSender();

// Gestionnaire des messages depuis content scripts et popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'GRAVIS_EXTRACT') {
    handleExtraction(message.payload, sender, sendResponse);
    return true; // Indique une réponse asynchrone
  }
  
  if (message.type === 'PING_GRAVIS') {
    handlePing(sendResponse);
    return true;
  }
});

async function handleExtraction(content, sender, sendResponse) {
  try {
    // Rate limiting
    const tabId = sender.tab?.id?.toString() || 'unknown';
    if (!rateLimiter.isAllowed(tabId)) {
      sendResponse({ 
        ok: false, 
        error: 'Rate limit exceeded. Please wait before extracting again.' 
      });
      return;
    }

    console.log('📄 Processing extraction request from:', content.url);
    
    // Créer payload sécurisé
    const securePayload = await payloadSender.createSecurePayload(content);
    
    // Envoyer à GRAVIS
    const result = await payloadSender.sendToGRAVIS(securePayload);
    
    sendResponse({ 
      ok: true, 
      message: 'Content extracted and sent to GRAVIS successfully',
      flags: securePayload.flags
    });
    
  } catch (error) {
    console.error('❌ Extraction failed:', error);
    sendResponse({ 
      ok: false, 
      error: error.message || 'Unknown extraction error'
    });
  }
}

async function handlePing(sendResponse) {
  try {
    const response = await fetch('http://127.0.0.1:8766/api/extension/ping');
    const data = await response.json();
    
    sendResponse({ 
      ok: true, 
      connected: true,
      gravis_status: data.message || 'Connected'
    });
  } catch (error) {
    console.warn('⚠️ GRAVIS ping failed:', error);
    sendResponse({ 
      ok: false, 
      connected: false,
      error: 'GRAVIS not detected'
    });
  }
}

// Log de démarrage
console.log('✅ GRAVIS Extension background script ready');