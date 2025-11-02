// GRAVIS Extension - Consent Manager
// Phase 1: Gestion du consentement per-site pour extraction

console.log('🔒 Consent Manager loading...');

class ConsentManager {
  constructor() {
    this.whitelist = new Set();
    this.blacklist = new Set();
    this.consentCache = new Map();
    this.isInitialized = false;
    
    this.init();
  }

  async init() {
    await this.loadSettings();
    this.setupDomainDetection();
    this.isInitialized = true;
    console.log('✅ Consent Manager initialized');
  }

  async loadSettings() {
    try {
      const result = await chrome.storage.local.get([
        'gravis_whitelist',
        'gravis_blacklist',
        'gravis_consent_mode'
      ]);
      
      this.whitelist = new Set(result.gravis_whitelist || []);
      this.blacklist = new Set(result.gravis_blacklist || []);
      this.consentMode = result.gravis_consent_mode || 'ask'; // 'ask', 'allow_all', 'deny_all'
      
      console.log('⚙️ Consent settings loaded:', {
        whitelisted: this.whitelist.size,
        blacklisted: this.blacklist.size,
        mode: this.consentMode
      });
      
    } catch (error) {
      console.error('❌ Failed to load consent settings:', error);
    }
  }

  async saveSettings() {
    try {
      await chrome.storage.local.set({
        'gravis_whitelist': Array.from(this.whitelist),
        'gravis_blacklist': Array.from(this.blacklist),
        'gravis_consent_mode': this.consentMode
      });
      
      console.log('💾 Consent settings saved');
    } catch (error) {
      console.error('❌ Failed to save consent settings:', error);
    }
  }

  setupDomainDetection() {
    // Détecter les domaines sensibles automatiquement
    const currentDomain = this.getCurrentDomain();
    
    if (this.isSensitiveDomain(currentDomain)) {
      console.log('⚠️ Sensitive domain detected:', currentDomain);
      this.handleSensitiveDomain(currentDomain);
    }
  }

  getCurrentDomain() {
    try {
      return new URL(window.location.href).hostname.toLowerCase();
    } catch {
      return null;
    }
  }

  getBaseDomain(domain) {
    if (!domain) return null;
    
    // Extraire le domaine de base (sans sous-domaines)
    const parts = domain.split('.');
    if (parts.length >= 2) {
      return parts.slice(-2).join('.');
    }
    return domain;
  }

  isSensitiveDomain(domain) {
    if (!domain) return false;
    
    const sensitiveDomains = [
      // Banques
      'bank', 'banking', 'paypal', 'stripe', 'visa', 'mastercard',
      // Gouvernement
      'gov', 'gouv', 'government',
      // Santé
      'health', 'medical', 'hospital',
      // Pages locales
      'localhost', '127.0.0.1', 'local',
      // Intranets
      'intranet', 'internal', 'corp',
      // Navigateur
      'chrome://', 'moz-extension://', 'chrome-extension://'
    ];
    
    return sensitiveDomains.some(sensitive => domain.includes(sensitive));
  }

  async handleSensitiveDomain(domain) {
    // Automatiquement blacklister les domaines sensibles
    this.blacklist.add(domain);
    await this.saveSettings();
    
    // Afficher une notification
    this.showSensitiveDomainNotification(domain);
  }

  showSensitiveDomainNotification(domain) {
    // Créer une notification discrète
    const notification = this.createNotificationElement(
      '🔒 Domaine Sensible Détecté',
      `GRAVIS a automatiquement désactivé l'extraction sur ${domain} pour votre sécurité.`,
      'info'
    );
    
    document.body.appendChild(notification);
    
    // Retirer après 5 secondes
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    }, 5000);
  }

  async checkConsent(url = window.location.href) {
    const domain = this.getCurrentDomain();
    const baseDomain = this.getBaseDomain(domain);
    
    if (!domain) {
      console.warn('⚠️ Cannot determine domain for consent check');
      return false;
    }
    
    // Vérifier le cache
    if (this.consentCache.has(domain)) {
      const cached = this.consentCache.get(domain);
      const cacheAge = Date.now() - cached.timestamp;
      
      // Cache valide pendant 1 heure
      if (cacheAge < 3600000) {
        console.log('📋 Using cached consent for', domain, ':', cached.granted);
        return cached.granted;
      }
    }
    
    // Vérifier les listes
    if (this.blacklist.has(domain) || this.blacklist.has(baseDomain)) {
      console.log('❌ Domain blacklisted:', domain);
      this.cacheConsent(domain, false);
      return false;
    }
    
    if (this.whitelist.has(domain) || this.whitelist.has(baseDomain)) {
      console.log('✅ Domain whitelisted:', domain);
      this.cacheConsent(domain, true);
      return true;
    }
    
    // Appliquer le mode de consentement
    switch (this.consentMode) {
      case 'allow_all':
        console.log('🟢 Allow-all mode, granting consent for', domain);
        this.cacheConsent(domain, true);
        return true;
        
      case 'deny_all':
        console.log('🔴 Deny-all mode, denying consent for', domain);
        this.cacheConsent(domain, false);
        return false;
        
      case 'ask':
      default:
        return await this.requestConsent(domain, url);
    }
  }

  async requestConsent(domain, url) {
    console.log('❓ Requesting consent for', domain);
    
    return new Promise((resolve) => {
      // Créer une modal de consentement
      const modal = this.createConsentModal(domain, url, (granted) => {
        if (granted) {
          this.whitelist.add(domain);
          console.log('✅ Consent granted for', domain);
        } else {
          console.log('❌ Consent denied for', domain);
        }
        
        this.cacheConsent(domain, granted);
        this.saveSettings();
        resolve(granted);
      });
      
      document.body.appendChild(modal);
    });
  }

  createConsentModal(domain, url, callback) {
    const modal = document.createElement('div');
    modal.className = 'gravis-consent-modal';
    modal.innerHTML = `
      <div class="gravis-consent-overlay"></div>
      <div class="gravis-consent-dialog">
        <div class="gravis-consent-header">
          <h3>🤖 GRAVIS - Demande d'Autorisation</h3>
        </div>
        <div class="gravis-consent-body">
          <p><strong>Autoriser GRAVIS à extraire le contenu de :</strong></p>
          <p class="domain"><code>${domain}</code></p>
          <div class="url-preview">
            <small>${url}</small>
          </div>
          <div class="privacy-note">
            <small>🔒 Le contenu sera traité localement uniquement. Aucune donnée n'est envoyée vers internet.</small>
          </div>
        </div>
        <div class="gravis-consent-actions">
          <button class="gravis-btn gravis-btn-deny">❌ Refuser</button>
          <button class="gravis-btn gravis-btn-allow">✅ Autoriser</button>
          <button class="gravis-btn gravis-btn-always">🔓 Toujours Autoriser</button>
        </div>
      </div>
    `;
    
    // Styles inline pour éviter les conflits
    const style = document.createElement('style');
    style.textContent = `
      .gravis-consent-modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: 999999;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      }
      .gravis-consent-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
      }
      .gravis-consent-dialog {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: white;
        border-radius: 8px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
        max-width: 400px;
        width: 90%;
      }
      .gravis-consent-header {
        padding: 16px 20px;
        border-bottom: 1px solid #eee;
      }
      .gravis-consent-header h3 {
        margin: 0;
        font-size: 16px;
        color: #333;
      }
      .gravis-consent-body {
        padding: 20px;
      }
      .gravis-consent-body p {
        margin: 0 0 12px 0;
        color: #333;
        font-size: 14px;
      }
      .domain {
        background: #f5f5f5;
        padding: 8px 12px;
        border-radius: 4px;
        font-family: monospace;
        word-break: break-all;
      }
      .url-preview {
        margin: 8px 0;
        color: #666;
      }
      .privacy-note {
        margin-top: 16px;
        padding: 12px;
        background: #e8f5e8;
        border-radius: 4px;
        color: #2d5016;
      }
      .gravis-consent-actions {
        padding: 16px 20px;
        display: flex;
        gap: 8px;
        border-top: 1px solid #eee;
      }
      .gravis-btn {
        flex: 1;
        padding: 8px 12px;
        border: 1px solid #ddd;
        border-radius: 4px;
        background: white;
        cursor: pointer;
        font-size: 12px;
        transition: background 0.2s;
      }
      .gravis-btn:hover {
        background: #f5f5f5;
      }
      .gravis-btn-allow {
        background: #4CAF50;
        color: white;
        border-color: #4CAF50;
      }
      .gravis-btn-allow:hover {
        background: #45a049;
      }
      .gravis-btn-deny {
        background: #f44336;
        color: white;
        border-color: #f44336;
      }
      .gravis-btn-deny:hover {
        background: #da190b;
      }
      .gravis-btn-always {
        background: #2196F3;
        color: white;
        border-color: #2196F3;
      }
      .gravis-btn-always:hover {
        background: #0b7dda;
      }
    `;
    
    modal.appendChild(style);
    
    // Event handlers
    const denyBtn = modal.querySelector('.gravis-btn-deny');
    const allowBtn = modal.querySelector('.gravis-btn-allow');
    const alwaysBtn = modal.querySelector('.gravis-btn-always');
    const overlay = modal.querySelector('.gravis-consent-overlay');
    
    const closeModal = () => {
      if (modal.parentNode) {
        modal.parentNode.removeChild(modal);
      }
    };
    
    denyBtn.onclick = () => {
      closeModal();
      callback(false);
    };
    
    allowBtn.onclick = () => {
      closeModal();
      callback(true);
    };
    
    alwaysBtn.onclick = () => {
      closeModal();
      this.whitelist.add(domain);
      callback(true);
    };
    
    overlay.onclick = () => {
      closeModal();
      callback(false);
    };
    
    // Fermer avec Escape
    const escapeHandler = (e) => {
      if (e.key === 'Escape') {
        closeModal();
        callback(false);
        document.removeEventListener('keydown', escapeHandler);
      }
    };
    document.addEventListener('keydown', escapeHandler);
    
    return modal;
  }

  createNotificationElement(title, message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `gravis-notification gravis-${type}`;
    notification.innerHTML = `
      <div class="gravis-notification-content">
        <div class="gravis-notification-title">${title}</div>
        <div class="gravis-notification-message">${message}</div>
      </div>
    `;
    
    // Styles
    const style = document.createElement('style');
    style.textContent = `
      .gravis-notification {
        position: fixed;
        top: 20px;
        right: 20px;
        background: white;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        max-width: 300px;
        z-index: 999998;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        border-left: 4px solid #2196F3;
      }
      .gravis-notification-content {
        padding: 16px;
      }
      .gravis-notification-title {
        font-weight: 600;
        margin-bottom: 4px;
        color: #333;
        font-size: 14px;
      }
      .gravis-notification-message {
        color: #666;
        font-size: 12px;
        line-height: 1.4;
      }
      .gravis-info {
        border-left-color: #2196F3;
      }
      .gravis-warning {
        border-left-color: #FF9800;
      }
      .gravis-error {
        border-left-color: #f44336;
      }
    `;
    
    notification.appendChild(style);
    return notification;
  }

  cacheConsent(domain, granted) {
    this.consentCache.set(domain, {
      granted,
      timestamp: Date.now()
    });
  }

  // API publique
  async grantConsent(domain) {
    this.whitelist.add(domain);
    this.blacklist.delete(domain);
    this.cacheConsent(domain, true);
    await this.saveSettings();
    console.log('✅ Consent manually granted for', domain);
  }

  async revokeConsent(domain) {
    this.whitelist.delete(domain);
    this.blacklist.add(domain);
    this.cacheConsent(domain, false);
    await this.saveSettings();
    console.log('❌ Consent manually revoked for', domain);
  }

  async setConsentMode(mode) {
    if (['ask', 'allow_all', 'deny_all'].includes(mode)) {
      this.consentMode = mode;
      await this.saveSettings();
      console.log('⚙️ Consent mode set to:', mode);
    }
  }

  getConsentStatus(domain = this.getCurrentDomain()) {
    if (this.whitelist.has(domain)) return 'granted';
    if (this.blacklist.has(domain)) return 'denied';
    return 'unknown';
  }

  getConsentStats() {
    return {
      mode: this.consentMode,
      whitelisted: Array.from(this.whitelist),
      blacklisted: Array.from(this.blacklist),
      cached: this.consentCache.size,
      currentDomain: this.getCurrentDomain(),
      currentStatus: this.getConsentStatus()
    };
  }

  clearAllConsent() {
    this.whitelist.clear();
    this.blacklist.clear();
    this.consentCache.clear();
    this.saveSettings();
    console.log('🧹 All consent data cleared');
  }
}

// Instance globale
window.consentManager = new ConsentManager();

// API pour les autres scripts
window.GRAVIS_CONSENT = {
  check: (url) => window.consentManager.checkConsent(url),
  grant: (domain) => window.consentManager.grantConsent(domain),
  revoke: (domain) => window.consentManager.revokeConsent(domain),
  setMode: (mode) => window.consentManager.setConsentMode(mode),
  getStatus: (domain) => window.consentManager.getConsentStatus(domain),
  getStats: () => window.consentManager.getConsentStats(),
  clearAll: () => window.consentManager.clearAllConsent()
};

console.log('✅ Consent Manager ready');