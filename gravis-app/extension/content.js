// GRAVIS Extension - Content Script
// Phase 0 Spike - Smart Content Extraction

console.log('🌐 GRAVIS Extension content script loaded on:', window.location.href);

// Classe pour extraction intelligente de contenu
class SmartContentExtractor {
  constructor() {
    this.isProcessing = false;
  }

  extract() {
    if (this.isProcessing) {
      console.warn('⚠️ Extraction already in progress');
      return null;
    }

    this.isProcessing = true;
    
    try {
      // 1. Sélection utilisateur (priorité max)
      const selection = window.getSelection()?.toString()?.trim();
      if (selection && selection.length > 50) {
        console.log('✂️ Using user selection:', selection.length, 'chars');
        return {
          method: 'user_selection',
          content: selection,
          confidence: 1.0,
          source: 'selection'
        };
      }
      
      // 2. Readability (si disponible)
      if (window.Readability) {
        try {
          const documentClone = document.cloneNode(true);
          const article = new Readability(documentClone).parse();
          
          if (article && article.textContent && article.textContent.length > 100) {
            console.log('📖 Using Readability extraction:', article.textContent.length, 'chars');
            return {
              method: 'readability',
              content: article.textContent,
              confidence: 0.9,
              metadata: {
                title: article.title,
                byline: article.byline,
                excerpt: article.excerpt
              },
              source: 'readability'
            };
          }
        } catch (e) {
          console.warn('⚠️ Readability failed:', e.message);
        }
      }
      
      // 3. Heuristiques candidats
      const candidates = this.getCandidateElements();
      const best = this.selectBestCandidate(candidates);
      
      if (best && best.score > 0.3) {
        console.log('🎯 Using heuristic extraction:', best.selector, 'score:', best.score);
        return {
          method: 'heuristic',
          content: this.cleanText(best.element.textContent),
          confidence: best.score,
          selector: best.selector,
          source: 'heuristic'
        };
      }
      
      // 4. Fallback body (filtré)
      const bodyText = this.extractBodyText();
      console.log('🔄 Using fallback body extraction:', bodyText.length, 'chars');
      return {
        method: 'fallback',
        content: bodyText,
        confidence: 0.3,
        source: 'body_fallback'
      };
      
    } finally {
      this.isProcessing = false;
    }
  }

  getCandidateElements() {
    const selectors = [
      'article', 'main', '[role="main"]',
      '.content', '.post', '.article', '.entry',
      '#content', '#main', '#article', '#post',
      '.post-content', '.entry-content', '.article-body',
      '.story-body', '.article-text', '.post-text',
      '[data-testid="article-body"]', '[data-component="ArticleBody"]'
    ];
    
    const candidates = [];
    
    selectors.forEach(selector => {
      try {
        const elements = document.querySelectorAll(selector);
        elements.forEach(element => {
          if (element && element.textContent) {
            candidates.push({
              element,
              selector,
              score: this.scoreElement(element)
            });
          }
        });
      } catch (e) {
        // Ignore invalid selectors
      }
    });
    
    return candidates;
  }

  selectBestCandidate(candidates) {
    if (!candidates.length) return null;
    
    // Trier par score décroissant
    candidates.sort((a, b) => b.score - a.score);
    
    // Retourner le meilleur candidat
    return candidates[0];
  }

  scoreElement(element) {
    let score = 0;
    const text = element.textContent?.trim() || '';
    
    if (text.length < 100) return 0; // Trop court
    
    // Score basé sur la longueur
    if (text.length > 500) score += 0.3;
    if (text.length > 1500) score += 0.2;
    if (text.length > 3000) score += 0.1;
    
    // Score basé sur les paragraphes
    const paragraphs = element.querySelectorAll('p').length;
    score += Math.min(paragraphs * 0.05, 0.3);
    
    // Ratio liens vs texte (moins de liens = mieux)
    const links = element.querySelectorAll('a').length;
    const linkRatio = links / Math.max(paragraphs, 1);
    if (linkRatio < 0.2) score += 0.2;
    else if (linkRatio > 0.5) score -= 0.3;
    
    // Classes et IDs positifs
    const className = (element.className || '').toLowerCase();
    const idName = (element.id || '').toLowerCase();
    const combinedNames = className + ' ' + idName;
    
    if (/content|article|post|entry|main|story|text/.test(combinedNames)) {
      score += 0.3;
    }
    
    // Classes et IDs négatifs
    if (/nav|sidebar|footer|header|menu|comment|ads|social|share/.test(combinedNames)) {
      score -= 0.5;
    }
    
    // Position dans le DOM (éléments plus profonds souvent meilleurs)
    const depth = this.getElementDepth(element);
    if (depth > 3 && depth < 10) score += 0.1;
    
    return Math.max(0, Math.min(1, score));
  }

  getElementDepth(element) {
    let depth = 0;
    let current = element;
    while (current.parentElement) {
      depth++;
      current = current.parentElement;
    }
    return depth;
  }

  extractBodyText() {
    // Cloner le body pour ne pas modifier l'original
    const bodyClone = document.body.cloneNode(true);
    
    // Supprimer les éléments indésirables
    const unwantedSelectors = [
      'script', 'style', 'nav', 'header', 'footer',
      '.nav', '.navbar', '.menu', '.sidebar',
      '.ads', '.advertisement', '.social', '.share',
      '.comments', '.comment-section'
    ];
    
    unwantedSelectors.forEach(selector => {
      const elements = bodyClone.querySelectorAll(selector);
      elements.forEach(el => el.remove());
    });
    
    const text = bodyClone.textContent || '';
    return this.cleanText(text).slice(0, 50000); // Limite 50KB
  }

  cleanText(text) {
    if (!text) return '';
    
    return text
      .replace(/\s+/g, ' ') // Normaliser les espaces
      .replace(/^\s+|\s+$/g, '') // Trim
      .replace(/\n\s*\n/g, '\n\n') // Normaliser les sauts de ligne
      .replace(/[^\S\n]+/g, ' '); // Nettoyer les espaces non-visibles
  }

  // Détection spécialisée
  isPDFPage() {
    return document.contentType === 'application/pdf' ||
           location.pathname.toLowerCase().endsWith('.pdf') ||
           document.querySelector('embed[type="application/pdf"]') ||
           document.title.toLowerCase().includes('pdf');
  }

  detectPageType() {
    const url = window.location.href.toLowerCase();
    const title = document.title.toLowerCase();
    
    if (this.isPDFPage()) return 'pdf';
    if (url.includes('github.com')) return 'code';
    if (url.includes('stackoverflow.com')) return 'qa';
    if (url.includes('wikipedia.org')) return 'wiki';
    if (title.includes('documentation') || url.includes('/docs/')) return 'docs';
    
    return 'article';
  }
}

// Instance globale d'extracteur
const extractor = new SmartContentExtractor();

// Fonction principale d'extraction avec Phase 1 features
async function extractContent(mode = 'auto') {
  console.log('🔄 Starting content extraction, mode:', mode);
  
  try {
    const currentUrl = window.location.href;
    
    // 1. Vérifier le consentement si disponible
    if (window.GRAVIS_CONSENT) {
      const hasConsent = await window.GRAVIS_CONSENT.check(currentUrl);
      if (!hasConsent) {
        console.log('❌ Consent denied for', currentUrl);
        return;
      }
    }
    
    // 2. Analyser PDF/Paywall si disponible
    let specialHandling = null;
    if (window.GRAVIS_PDF_PAYWALL) {
      specialHandling = await window.GRAVIS_PDF_PAYWALL.getOptimalContent();
      if (specialHandling.method !== 'standard_extraction') {
        console.log('🔧 Special content handling:', specialHandling.method);
        // Envoyer le contenu spécialisé
        await sendExtractedContent({
          url: currentUrl,
          title: document.title,
          mainContent: specialHandling.content || '[Contenu spécialisé]',
          selectedText: null,
          extraction_method: specialHandling.method,
          metadata: {
            isSpecialContent: true,
            warning: specialHandling.warning || null,
            fallbackNeeded: specialHandling.fallbackNeeded || false
          },
          timestamp: Date.now()
        });
        return;
      }
    }
    
    // 3. Vérifier le cache si disponible
    let result = null;
    if (window.GRAVIS_CACHE) {
      result = await window.GRAVIS_CACHE.getOrExtract(currentUrl, async () => {
        return await performExtraction(mode);
      }, { mode });
      
      if (result && result._fromCache) {
        console.log('📦 Using cached content');
      }
    } else {
      result = await performExtraction(mode);
    }
    
    if (!result) {
      throw new Error('No content extracted');
    }
    
    // 4. Envoyer le contenu extrait
    await sendExtractedContent({
      url: currentUrl,
      title: document.title,
      mainContent: result.content,
      selectedText: result.selectedText || (result.method === 'user_selection' ? result.content : null),
      extraction_method: `extension_${result.method}`,
      metadata: {
        method: result.method,
        confidence: result.confidence || 0.8,
        source: result.source,
        pageType: extractor.detectPageType(),
        selector: result.selector || null,
        readabilityMeta: result.metadata || null,
        fromCache: result._fromCache || false
      },
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error('❌ Extraction failed:', error);
    throw error;
  }
}

async function performExtraction(mode) {
  console.log('🔄 performExtraction called with mode:', mode);
  let result;
  
  // Mapper les modes du popup vers les modes internes
  const actualMode = mode === 'page' ? 'auto' : mode;
  console.log('📝 Mapped mode:', mode, '=>', actualMode);
  
  // Utiliser l'extracteur avancé si disponible et mode auto
  if (actualMode === 'auto' && window.advancedExtractor) {
    console.log('🔧 Using advanced extractor...');
    try {
      result = await window.advancedExtractor.extractWithAdvancedFeatures();
    } catch (advancedError) {
      console.warn('⚠️ Advanced extraction failed, falling back to basic:', advancedError);
      result = extractor.extract();
    }
  } else {
    console.log('🔧 Using basic extractor...');
    result = extractor.extract();
  }
  
  console.log('📤 Extraction result:', {
    method: result?.method,
    contentLength: result?.content?.length,
    confidence: result?.confidence
  });
  
  return result;
}

async function sendExtractedContent(payload) {
  console.log('📦 Sending extraction payload:', {
    method: payload.extraction_method,
    confidence: payload.metadata?.confidence,
    contentLength: payload.mainContent?.length,
    fromCache: payload.metadata?.fromCache
  });
    
  // Envoyer au background script
  chrome.runtime.sendMessage({
    type: 'GRAVIS_EXTRACT',
    payload: payload
  }).then(response => {
    if (response?.ok) {
      console.log('✅ Content sent to GRAVIS successfully');
      showExtractionFeedback('success', 'Content sent to GRAVIS!');
    } else {
      console.error('❌ Failed to send content:', response?.error);
      showExtractionFeedback('error', response?.error || 'Extraction failed');
    }
  }).catch(error => {
    console.error('❌ Send message error:', error);
    showExtractionFeedback('error', error.message);
  });
}

// Feedback visuel simple
function showExtractionFeedback(type, message) {
  // Créer une notification temporaire
  const notification = document.createElement('div');
  notification.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: ${type === 'success' ? '#4CAF50' : '#f44336'};
    color: white;
    padding: 12px 20px;
    border-radius: 4px;
    z-index: 10000;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    transition: opacity 0.3s ease;
  `;
  
  notification.textContent = `🤖 GRAVIS: ${message}`;
  document.body.appendChild(notification);
  
  // Auto-remove après 3 secondes
  setTimeout(() => {
    notification.style.opacity = '0';
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    }, 300);
  }, 3000);
}

// Écouter les messages du background script ou popup
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'EXTRACT_CONTENT') {
    extractContent(message.mode || 'auto').then(() => {
      sendResponse({ ok: true });
    }).catch(error => {
      console.error('❌ Extract content error:', error);
      sendResponse({ ok: false, error: error.message });
    });
    return true; // Indique une réponse asynchrone
  }
});

// Log de démarrage
console.log('✅ GRAVIS content script ready, extractor initialized');