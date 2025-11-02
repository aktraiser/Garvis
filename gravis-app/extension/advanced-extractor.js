// GRAVIS Extension - Advanced Content Extractor
// Phase 1: Shadow DOM, Iframes, Complex DOM Structures

if (typeof window.AdvancedContentExtractor === 'undefined') {
  console.log('🔧 Advanced Content Extractor loaded');

class AdvancedContentExtractor {
  constructor() {
    this.isProcessing = false;
    this.frameContentCache = new Map();
  }

  async extractWithAdvancedFeatures() {
    if (this.isProcessing) {
      console.warn('⚠️ Advanced extraction already in progress');
      return null;
    }

    this.isProcessing = true;
    
    try {
      console.log('🔍 Starting advanced extraction...');
      
      // 1. Contenu principal du document
      const mainContent = this.extractMainContent();
      
      // 2. Extraction des iframes accessibles
      const frameContent = await this.extractFromFrames();
      
      // 3. Extraction du Shadow DOM
      const shadowContent = this.extractFromShadowDOM();
      
      // 4. Contenu dynamique (lazy-loaded)
      const dynamicContent = await this.extractDynamicContent();
      
      // 5. Combiner tout le contenu
      const combinedContent = this.combineContent({
        main: mainContent,
        frames: frameContent,
        shadow: shadowContent,
        dynamic: dynamicContent
      });
      
      console.log('✅ Advanced extraction completed:', {
        mainLength: mainContent.length,
        framesCount: frameContent.length,
        shadowElements: shadowContent.length,
        dynamicElements: dynamicContent.length,
        totalLength: combinedContent.length
      });
      
      return {
        method: 'advanced_extraction',
        content: combinedContent,
        confidence: 0.95,
        metadata: {
          hasFrames: frameContent.length > 0,
          hasShadowDOM: shadowContent.length > 0,
          hasDynamicContent: dynamicContent.length > 0,
          extractionComplexity: this.assessComplexity()
        }
      };
      
    } catch (error) {
      console.error('❌ Advanced extraction error:', error);
      throw error;
    } finally {
      this.isProcessing = false;
    }
  }

  extractMainContent() {
    // Utiliser l'extracteur existant comme base
    if (typeof window.extractor !== 'undefined' && window.extractor.extract) {
      const result = window.extractor.extract();
      return result.content || '';
    }
    
    // Fallback si extracteur principal non disponible
    return this.fallbackMainExtraction();
  }

  async extractFromFrames() {
    const frameContents = [];
    const frames = document.querySelectorAll('iframe');
    
    console.log(`🖼️ Found ${frames.length} iframe(s) to analyze`);
    
    for (const frame of frames) {
      try {
        // Essayer d'accéder au contenu de l'iframe (same-origin)
        const frameDoc = frame.contentDocument || frame.contentWindow?.document;
        
        if (frameDoc) {
          console.log('✅ Same-origin iframe accessible:', frame.src);
          const content = this.extractFromDocument(frameDoc);
          if (content.trim().length > 50) {
            frameContents.push({
              src: frame.src || 'inline',
              content: content,
              method: 'direct_access'
            });
          }
        } else {
          // Cross-origin iframe - essayer postMessage
          console.log('🔒 Cross-origin iframe detected:', frame.src);
          const content = await this.requestFrameContentViaPostMessage(frame);
          if (content) {
            frameContents.push({
              src: frame.src,
              content: content,
              method: 'post_message'
            });
          }
        }
      } catch (e) {
        console.warn('⚠️ Cannot access iframe:', frame.src, e.message);
      }
    }
    
    return frameContents;
  }

  extractFromDocument(doc) {
    // Extraction du contenu d'un document (principal ou iframe)
    const textContent = doc.body?.textContent || '';
    return this.cleanText(textContent);
  }

  async requestFrameContentViaPostMessage(iframe) {
    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        resolve(null);
      }, 2000); // 2s timeout
      
      const messageHandler = (event) => {
        if (event.source === iframe.contentWindow) {
          if (event.data.type === 'GRAVIS_EXTRACT_RESPONSE') {
            clearTimeout(timeout);
            window.removeEventListener('message', messageHandler);
            resolve(event.data.content);
          }
        }
      };
      
      window.addEventListener('message', messageHandler);
      
      // Envoyer la demande d'extraction
      try {
        iframe.contentWindow?.postMessage({
          type: 'GRAVIS_EXTRACT_REQUEST',
          origin: window.location.origin
        }, '*');
      } catch (e) {
        clearTimeout(timeout);
        resolve(null);
      }
    });
  }

  extractFromShadowDOM() {
    const shadowContents = [];
    
    // Trouver tous les éléments avec Shadow DOM
    const shadowRoots = this.findAllShadowRoots(document.body);
    
    console.log(`🌑 Found ${shadowRoots.length} Shadow DOM root(s)`);
    
    shadowRoots.forEach((shadowRoot, index) => {
      try {
        const content = this.extractFromDocument(shadowRoot);
        if (content.trim().length > 50) {
          shadowContents.push({
            index: index,
            content: content,
            elementTag: shadowRoot.host?.tagName || 'unknown'
          });
        }
      } catch (e) {
        console.warn('⚠️ Error extracting from Shadow DOM:', e);
      }
    });
    
    return shadowContents;
  }

  findAllShadowRoots(element) {
    const shadowRoots = [];
    
    // Vérifier l'élément actuel
    if (element.shadowRoot) {
      shadowRoots.push(element.shadowRoot);
    }
    
    // Parcourir récursivement tous les enfants
    const children = element.querySelectorAll('*');
    children.forEach(child => {
      if (child.shadowRoot) {
        shadowRoots.push(child.shadowRoot);
        // Récursion dans le Shadow DOM
        const nestedShadows = this.findAllShadowRoots(child.shadowRoot);
        shadowRoots.push(...nestedShadows);
      }
    });
    
    return shadowRoots;
  }

  async extractDynamicContent() {
    const dynamicContents = [];
    
    // 1. Contenu lazy-loaded
    const lazyContent = await this.triggerLazyLoading();
    if (lazyContent) dynamicContents.push(lazyContent);
    
    // 2. Contenu généré par JS après un délai
    const delayedContent = await this.waitForDynamicContent();
    if (delayedContent) dynamicContents.push(delayedContent);
    
    // 3. Contenu dans les éléments cachés mais importantes
    const hiddenContent = this.extractHiddenImportantContent();
    if (hiddenContent) dynamicContents.push(hiddenContent);
    
    return dynamicContents;
  }

  async triggerLazyLoading() {
    // Simuler le scroll pour déclencher le lazy loading
    const originalScrollY = window.scrollY;
    
    try {
      // Scroll rapide vers le bas
      window.scrollTo(0, document.body.scrollHeight);
      await this.wait(500);
      
      // Scroll vers le haut
      window.scrollTo(0, 0);
      await this.wait(500);
      
      // Restaurer position originale
      window.scrollTo(0, originalScrollY);
      
      // Chercher du nouveau contenu qui pourrait avoir été chargé
      const newElements = document.querySelectorAll('[data-loaded="true"], .loaded, .lazy-loaded');
      if (newElements.length > 0) {
        console.log(`🔄 Triggered lazy loading, found ${newElements.length} new elements`);
        return Array.from(newElements).map(el => el.textContent?.trim()).filter(Boolean).join('\n');
      }
    } catch (e) {
      console.warn('⚠️ Lazy loading trigger failed:', e);
    }
    
    return null;
  }

  async waitForDynamicContent() {
    const initialLength = document.body.textContent?.length || 0;
    
    // Attendre 1 seconde pour du contenu dynamique
    await this.wait(1000);
    
    const finalLength = document.body.textContent?.length || 0;
    
    if (finalLength > initialLength + 100) {
      console.log(`📈 Dynamic content detected: ${finalLength - initialLength} new characters`);
      return `[Dynamic content detected: ${finalLength - initialLength} characters added]`;
    }
    
    return null;
  }

  extractHiddenImportantContent() {
    // Chercher du contenu dans des éléments cachés mais sémantiquement importants
    const hiddenSelectors = [
      '[style*="display: none"]',
      '[style*="visibility: hidden"]',
      '.hidden',
      '.sr-only',
      '[aria-hidden="true"]'
    ];
    
    const hiddenContents = [];
    
    hiddenSelectors.forEach(selector => {
      const elements = document.querySelectorAll(selector);
      elements.forEach(el => {
        // Vérifier si l'élément contient du contenu important
        if (this.isImportantHiddenContent(el)) {
          const content = el.textContent?.trim();
          if (content && content.length > 20) {
            hiddenContents.push(content);
          }
        }
      });
    });
    
    if (hiddenContents.length > 0) {
      console.log(`👻 Found ${hiddenContents.length} important hidden content sections`);
      return hiddenContents.join('\n\n');
    }
    
    return null;
  }

  isImportantHiddenContent(element) {
    const text = element.textContent?.toLowerCase() || '';
    const classList = element.className?.toLowerCase() || '';
    
    // Contenu caché mais important (métadonnées, descriptions, etc.)
    const importantKeywords = [
      'description', 'summary', 'abstract', 'excerpt',
      'metadata', 'schema', 'data-', 'aria-label'
    ];
    
    return importantKeywords.some(keyword => 
      text.includes(keyword) || classList.includes(keyword)
    );
  }

  combineContent(contents) {
    const combined = [];
    
    // Contenu principal
    if (contents.main) {
      combined.push('=== MAIN CONTENT ===');
      combined.push(contents.main);
    }
    
    // Contenu des frames
    if (contents.frames.length > 0) {
      combined.push('\n=== IFRAME CONTENT ===');
      contents.frames.forEach((frame, index) => {
        combined.push(`--- Frame ${index + 1} (${frame.src}) ---`);
        combined.push(frame.content);
      });
    }
    
    // Contenu Shadow DOM
    if (contents.shadow.length > 0) {
      combined.push('\n=== SHADOW DOM CONTENT ===');
      contents.shadow.forEach((shadow, index) => {
        combined.push(`--- Shadow ${index + 1} (${shadow.elementTag}) ---`);
        combined.push(shadow.content);
      });
    }
    
    // Contenu dynamique
    if (contents.dynamic.length > 0) {
      combined.push('\n=== DYNAMIC CONTENT ===');
      contents.dynamic.forEach(dynamic => {
        combined.push(dynamic);
      });
    }
    
    return combined.join('\n').slice(0, 100000); // Limite 100KB
  }

  assessComplexity() {
    const frames = document.querySelectorAll('iframe').length;
    const shadowRoots = this.findAllShadowRoots(document.body).length;
    const scripts = document.querySelectorAll('script').length;
    
    if (frames > 3 || shadowRoots > 2 || scripts > 20) return 'high';
    if (frames > 0 || shadowRoots > 0 || scripts > 10) return 'medium';
    return 'low';
  }

  fallbackMainExtraction() {
    // Extraction basique si l'extracteur principal n'est pas disponible
    const candidates = [
      'main', 'article', '[role="main"]',
      '.content', '.post', '.article',
      '#content', '#main'
    ];
    
    for (const selector of candidates) {
      const element = document.querySelector(selector);
      if (element) {
        return this.cleanText(element.textContent || '');
      }
    }
    
    return this.cleanText(document.body.textContent || '').slice(0, 50000);
  }

  cleanText(text) {
    return text
      .replace(/\s+/g, ' ')
      .replace(/^\s+|\s+$/g, '')
      .replace(/\n\s*\n/g, '\n\n');
  }

  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Instance globale
window.advancedExtractor = new AdvancedContentExtractor();

// Listener pour les demandes d'extraction avancée
window.addEventListener('message', (event) => {
  if (event.data.type === 'GRAVIS_EXTRACT_REQUEST') {
    console.log('📨 Received extraction request from parent');
    
    // Extraire le contenu et répondre
    const extractor = new AdvancedContentExtractor();
    extractor.extractWithAdvancedFeatures().then(result => {
      event.source?.postMessage({
        type: 'GRAVIS_EXTRACT_RESPONSE',
        content: result?.content || ''
      }, event.origin);
    }).catch(error => {
      console.error('❌ Error in frame extraction:', error);
      event.source?.postMessage({
        type: 'GRAVIS_EXTRACT_RESPONSE',
        content: ''
      }, event.origin);
    });
  }
});

console.log('✅ Advanced Content Extractor ready');

} // End of if (typeof window.AdvancedContentExtractor === 'undefined')