// GRAVIS Extension - SPA Navigation Handler
// Phase 1: Navigation hooks pour Single Page Applications

console.log('🔄 SPA Navigation Handler loading...');

class SPANavigationHandler {
  constructor() {
    this.lastUrl = location.href;
    this.lastTitle = document.title;
    this.navigationCount = 0;
    this.autoExtractEnabled = false;
    this.debounceTimer = null;
    this.isSetup = false;
    
    this.setupNavigationListeners();
    this.loadSettings();
  }

  async loadSettings() {
    try {
      const result = await chrome.storage.local.get(['gravis_auto_extract', 'gravis_spa_delay']);
      this.autoExtractEnabled = result.gravis_auto_extract || false;
      this.extractionDelay = result.gravis_spa_delay || 1500; // 1.5s par défaut
      
      console.log('⚙️ SPA settings loaded:', {
        autoExtract: this.autoExtractEnabled,
        delay: this.extractionDelay
      });
    } catch (error) {
      console.warn('⚠️ Failed to load SPA settings:', error);
      this.extractionDelay = 1500;
    }
  }

  async saveSettings() {
    try {
      await chrome.storage.local.set({
        'gravis_auto_extract': this.autoExtractEnabled,
        'gravis_spa_delay': this.extractionDelay
      });
    } catch (error) {
      console.warn('⚠️ Failed to save SPA settings:', error);
    }
  }

  setupNavigationListeners() {
    if (this.isSetup) return;
    
    console.log('🎣 Setting up SPA navigation hooks...');
    
    // Hook 1: History API (pushState/replaceState)
    this.hookHistoryAPI();
    
    // Hook 2: Popstate (back/forward)
    window.addEventListener('popstate', (event) => {
      console.log('⬅️ Popstate navigation detected');
      this.handleNavigation('popstate', event.state);
    });
    
    // Hook 3: Hashchange
    window.addEventListener('hashchange', (event) => {
      console.log('🔗 Hash navigation detected');
      this.handleNavigation('hashchange', { oldURL: event.oldURL, newURL: event.newURL });
    });
    
    // Hook 4: DOM mutations (pour détecter les changements de contenu)
    this.setupMutationObserver();
    
    // Hook 5: Focus/blur pour détecter les changements d'onglets
    this.setupVisibilityDetection();
    
    this.isSetup = true;
    console.log('✅ SPA navigation hooks configured');
  }

  hookHistoryAPI() {
    // Sauvegarder les méthodes originales
    const originalPushState = history.pushState;
    const originalReplaceState = history.replaceState;
    
    // Override pushState
    history.pushState = (...args) => {
      const [state, title, url] = args;
      console.log('📝 pushState navigation:', url);
      
      // Appeler la méthode originale
      originalPushState.apply(history, args);
      
      // Traiter la navigation
      this.handleNavigation('pushState', { state, title, url });
    };
    
    // Override replaceState
    history.replaceState = (...args) => {
      const [state, title, url] = args;
      console.log('🔄 replaceState navigation:', url);
      
      // Appeler la méthode originale
      originalReplaceState.apply(history, args);
      
      // Traiter la navigation
      this.handleNavigation('replaceState', { state, title, url });
    };
  }

  setupMutationObserver() {
    // Observer les changements significatifs dans le DOM
    const observer = new MutationObserver((mutations) => {
      let significantChange = false;
      
      mutations.forEach((mutation) => {
        // Changements d'attributs importants
        if (mutation.type === 'attributes') {
          const target = mutation.target;
          if (target.tagName === 'TITLE' || 
              target.getAttribute('data-page') !== mutation.oldValue) {
            significantChange = true;
          }
        }
        
        // Ajout/suppression de noeuds importants
        if (mutation.type === 'childList') {
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              const element = node;
              if (this.isSignificantElement(element)) {
                significantChange = true;
              }
            }
          });
        }
      });
      
      if (significantChange) {
        console.log('🔄 Significant DOM change detected');
        this.handleNavigation('mutation', { mutationsCount: mutations.length });
      }
    });
    
    // Observer le document entier
    observer.observe(document, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeOldValue: true,
      attributeFilter: ['data-page', 'data-route', 'id', 'class']
    });
    
    this.mutationObserver = observer;
  }

  setupVisibilityDetection() {
    // Détecter quand l'utilisateur revient sur l'onglet
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden) {
        console.log('👁️ Tab became visible, checking for changes...');
        this.handleNavigation('visibility', { became: 'visible' });
      }
    });
    
    // Détecter le focus sur la fenêtre
    window.addEventListener('focus', () => {
      console.log('🎯 Window focused, checking for changes...');
      this.handleNavigation('focus', { type: 'window' });
    });
  }

  isSignificantElement(element) {
    const significantTags = ['MAIN', 'ARTICLE', 'SECTION'];
    const significantClasses = ['content', 'main', 'article', 'post', 'page'];
    const significantIds = ['content', 'main', 'article', 'app'];
    
    // Vérifier le tag
    if (significantTags.includes(element.tagName)) return true;
    
    // Vérifier les classes
    const classList = element.className.toLowerCase();
    if (significantClasses.some(cls => classList.includes(cls))) return true;
    
    // Vérifier l'ID
    const id = element.id.toLowerCase();
    if (significantIds.some(idName => id.includes(idName))) return true;
    
    // Vérifier la taille (élément avec beaucoup de contenu)
    const textLength = element.textContent?.length || 0;
    if (textLength > 1000) return true;
    
    return false;
  }

  handleNavigation(type, data = {}) {
    const currentUrl = location.href;
    const currentTitle = document.title;
    
    // Vérifier si c'est vraiment une nouvelle navigation
    if (currentUrl === this.lastUrl && currentTitle === this.lastTitle) {
      return; // Pas de changement réel
    }
    
    console.log(`🧭 Navigation detected:`, {
      type,
      from: this.lastUrl,
      to: currentUrl,
      titleChanged: currentTitle !== this.lastTitle,
      data
    });
    
    // Mettre à jour l'état
    this.lastUrl = currentUrl;
    this.lastTitle = currentTitle;
    this.navigationCount++;
    
    // Debounce pour éviter les extractions multiples rapides
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    
    this.debounceTimer = setTimeout(() => {
      this.processNavigation(type, data);
    }, 300); // 300ms debounce
  }

  async processNavigation(type, data) {
    try {
      // Notifier le background script de la navigation
      await this.notifyNavigation(type, data);
      
      // Auto-extraction si activée
      if (this.autoExtractEnabled && this.shouldAutoExtract(type)) {
        console.log(`⚡ Auto-extraction triggered after ${type} navigation`);
        await this.triggerAutoExtraction();
      }
      
      // Mettre à jour les métriques
      this.updateNavigationMetrics(type);
      
    } catch (error) {
      console.error('❌ Error processing navigation:', error);
    }
  }

  async notifyNavigation(type, data) {
    try {
      await chrome.runtime.sendMessage({
        type: 'SPA_NAVIGATION',
        navigation: {
          type,
          url: location.href,
          title: document.title,
          timestamp: Date.now(),
          count: this.navigationCount,
          data
        }
      });
    } catch (error) {
      console.warn('⚠️ Failed to notify background of navigation:', error);
    }
  }

  shouldAutoExtract(navigationType) {
    // Ne pas extraire automatiquement pour certains types
    const skipTypes = ['visibility', 'focus'];
    if (skipTypes.includes(navigationType)) return false;
    
    // Ne pas extraire si on a déjà extrait récemment
    const lastExtraction = this.getLastExtractionTime();
    if (lastExtraction && (Date.now() - lastExtraction) < 5000) {
      console.log('⏱️ Skipping auto-extraction (too recent)');
      return false;
    }
    
    // Vérifier si la page semble avoir du contenu substantiel
    const bodyText = document.body.textContent || '';
    if (bodyText.length < 500) {
      console.log('📄 Skipping auto-extraction (content too short)');
      return false;
    }
    
    return true;
  }

  async triggerAutoExtraction() {
    try {
      // Attendre que le contenu se stabilise
      await this.waitForContentStability();
      
      // Déclencher l'extraction
      if (typeof extractContent === 'function') {
        await extractContent('auto');
      } else {
        // Fallback via message
        chrome.runtime.sendMessage({
          type: 'TRIGGER_EXTRACTION',
          mode: 'auto',
          source: 'spa_navigation'
        });
      }
      
      this.setLastExtractionTime(Date.now());
      
    } catch (error) {
      console.error('❌ Auto-extraction failed:', error);
    }
  }

  async waitForContentStability() {
    // Attendre que le contenu arrête de changer
    let lastContentLength = document.body.textContent?.length || 0;
    let stableCount = 0;
    
    for (let i = 0; i < 10; i++) { // Max 3 secondes
      await this.wait(this.extractionDelay / 10);
      
      const currentLength = document.body.textContent?.length || 0;
      
      if (Math.abs(currentLength - lastContentLength) < 100) {
        stableCount++;
        if (stableCount >= 2) {
          console.log('✅ Content stabilized, proceeding with extraction');
          break;
        }
      } else {
        stableCount = 0;
      }
      
      lastContentLength = currentLength;
    }
  }

  updateNavigationMetrics(type) {
    // Métriques simples en localStorage
    try {
      const metrics = JSON.parse(localStorage.getItem('gravis_nav_metrics') || '{}');
      
      metrics.totalNavigations = (metrics.totalNavigations || 0) + 1;
      metrics.byType = metrics.byType || {};
      metrics.byType[type] = (metrics.byType[type] || 0) + 1;
      metrics.lastNavigation = {
        type,
        url: location.href,
        timestamp: Date.now()
      };
      
      localStorage.setItem('gravis_nav_metrics', JSON.stringify(metrics));
    } catch (error) {
      console.warn('⚠️ Failed to update navigation metrics:', error);
    }
  }

  getLastExtractionTime() {
    try {
      return parseInt(localStorage.getItem('gravis_last_extraction') || '0');
    } catch {
      return 0;
    }
  }

  setLastExtractionTime(timestamp) {
    try {
      localStorage.setItem('gravis_last_extraction', timestamp.toString());
    } catch (error) {
      console.warn('⚠️ Failed to save extraction timestamp:', error);
    }
  }

  // API publique
  enableAutoExtract() {
    this.autoExtractEnabled = true;
    this.saveSettings();
    console.log('✅ Auto-extraction enabled');
  }

  disableAutoExtract() {
    this.autoExtractEnabled = false;
    this.saveSettings();
    console.log('❌ Auto-extraction disabled');
  }

  setExtractionDelay(ms) {
    this.extractionDelay = Math.max(500, Math.min(5000, ms)); // Entre 0.5s et 5s
    this.saveSettings();
    console.log(`⏱️ Extraction delay set to ${this.extractionDelay}ms`);
  }

  getNavigationMetrics() {
    try {
      return JSON.parse(localStorage.getItem('gravis_nav_metrics') || '{}');
    } catch {
      return {};
    }
  }

  cleanup() {
    if (this.mutationObserver) {
      this.mutationObserver.disconnect();
    }
    
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    
    console.log('🧹 SPA Navigation Handler cleaned up');
  }

  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Instance globale
window.spaNavigationHandler = new SPANavigationHandler();

// API pour les autres scripts
window.GRAVIS_SPA = {
  enableAutoExtract: () => window.spaNavigationHandler.enableAutoExtract(),
  disableAutoExtract: () => window.spaNavigationHandler.disableAutoExtract(),
  setDelay: (ms) => window.spaNavigationHandler.setExtractionDelay(ms),
  getMetrics: () => window.spaNavigationHandler.getNavigationMetrics(),
  isAutoEnabled: () => window.spaNavigationHandler.autoExtractEnabled
};

console.log('✅ SPA Navigation Handler ready');