// GRAVIS Extension - PDF & Paywall Handler
// Phase 1: Détection et gestion des PDFs et paywalls

console.log('📄 PDF & Paywall Handler loading...');

class PDFPaywallHandler {
  constructor() {
    this.pdfDetectionMethods = [];
    this.paywallIndicators = [];
    this.isInitialized = false;
    
    this.init();
  }

  init() {
    this.setupPDFDetection();
    this.setupPaywallDetection();
    this.isInitialized = true;
    console.log('✅ PDF & Paywall Handler initialized');
  }

  setupPDFDetection() {
    this.pdfDetectionMethods = [
      () => this.detectByContentType(),
      () => this.detectByURL(),
      () => this.detectByEmbeds(),
      () => this.detectByViewerElements(),
      () => this.detectByObjectElements()
    ];
  }

  setupPaywallDetection() {
    this.paywallIndicators = [
      // Textes classiques de paywall
      'subscribe to continue',
      'login to read',
      'premium content',
      'subscribe for unlimited access',
      'upgrade to read',
      'become a member',
      'unlock this article',
      'this article is for subscribers only',
      'continue reading with',
      'get unlimited access',
      
      // Textes français
      'abonnez-vous pour continuer',
      'connectez-vous pour lire',
      'contenu premium',
      'contenu réservé aux abonnés',
      'débloquer cet article',
      'accès illimité',
      
      // Sélecteurs CSS courants
      '.paywall',
      '.subscription-wall',
      '.premium-content',
      '.article-gate',
      '.subscriber-only',
      '[data-testid="paywall"]',
      '.login-wall',
      '.premium-banner'
    ];
  }

  // === DÉTECTION PDF ===

  isPDF() {
    console.log('🔍 Checking if current page is a PDF...');
    
    for (const method of this.pdfDetectionMethods) {
      try {
        const result = method();
        if (result.isPDF) {
          console.log(`✅ PDF detected via ${result.method}:`, result.details);
          return result;
        }
      } catch (error) {
        console.warn('⚠️ PDF detection method failed:', error);
      }
    }
    
    console.log('❌ No PDF detected');
    return { isPDF: false, method: 'none' };
  }

  detectByContentType() {
    const contentType = document.contentType || '';
    if (contentType.includes('application/pdf')) {
      return {
        isPDF: true,
        method: 'content_type',
        details: { contentType }
      };
    }
    return { isPDF: false };
  }

  detectByURL() {
    const url = window.location.href.toLowerCase();
    const pathname = window.location.pathname.toLowerCase();
    
    if (url.includes('.pdf') || pathname.endsWith('.pdf')) {
      return {
        isPDF: true,
        method: 'url',
        details: { url, pathname }
      };
    }
    return { isPDF: false };
  }

  detectByEmbeds() {
    const embeds = document.querySelectorAll('embed[type="application/pdf"]');
    if (embeds.length > 0) {
      return {
        isPDF: true,
        method: 'embed_elements',
        details: { count: embeds.length, sources: Array.from(embeds).map(e => e.src) }
      };
    }
    return { isPDF: false };
  }

  detectByViewerElements() {
    // Détecter les viewers PDF intégrés
    const viewerSelectors = [
      '#viewer', // PDF.js
      '.pdfViewer',
      '.pdf-viewer',
      '[data-pdf-viewer]',
      'embed[src*=".pdf"]',
      'iframe[src*=".pdf"]'
    ];
    
    for (const selector of viewerSelectors) {
      const elements = document.querySelectorAll(selector);
      if (elements.length > 0) {
        return {
          isPDF: true,
          method: 'viewer_elements',
          details: { selector, count: elements.length }
        };
      }
    }
    return { isPDF: false };
  }

  detectByObjectElements() {
    const objects = document.querySelectorAll('object[data*=".pdf"], object[type="application/pdf"]');
    if (objects.length > 0) {
      return {
        isPDF: true,
        method: 'object_elements',
        details: { count: objects.length }
      };
    }
    return { isPDF: false };
  }

  // === DÉTECTION PAYWALL ===

  isPaywalled() {
    console.log('🔍 Checking for paywall indicators...');
    
    const results = {
      isPaywalled: false,
      confidence: 0,
      indicators: [],
      publicContent: null
    };
    
    // Vérifier les sélecteurs CSS
    const cssIndicators = this.checkPaywallSelectors();
    if (cssIndicators.length > 0) {
      results.isPaywalled = true;
      results.confidence += 0.4;
      results.indicators.push(...cssIndicators);
    }
    
    // Vérifier le texte de la page
    const textIndicators = this.checkPaywallText();
    if (textIndicators.length > 0) {
      results.isPaywalled = true;
      results.confidence += 0.3;
      results.indicators.push(...textIndicators);
    }
    
    // Vérifier les patterns de contenu tronqué
    const truncationIndicators = this.checkContentTruncation();
    if (truncationIndicators.length > 0) {
      results.isPaywalled = true;
      results.confidence += 0.2;
      results.indicators.push(...truncationIndicators);
    }
    
    // Vérifier les overlays/modals
    const overlayIndicators = this.checkPaywallOverlays();
    if (overlayIndicators.length > 0) {
      results.isPaywalled = true;
      results.confidence += 0.5;
      results.indicators.push(...overlayIndicators);
    }
    
    // Extraire le contenu public disponible
    if (results.isPaywalled) {
      results.publicContent = this.extractPublicContent();
    }
    
    results.confidence = Math.min(1.0, results.confidence);
    
    if (results.isPaywalled) {
      console.log(`🚧 Paywall detected with ${Math.round(results.confidence * 100)}% confidence:`, results.indicators);
    } else {
      console.log('✅ No paywall detected');
    }
    
    return results;
  }

  checkPaywallSelectors() {
    const indicators = [];
    
    const selectors = this.paywallIndicators.filter(indicator => indicator.startsWith('.') || indicator.startsWith('['));
    
    for (const selector of selectors) {
      try {
        const elements = document.querySelectorAll(selector);
        if (elements.length > 0) {
          indicators.push({
            type: 'css_selector',
            value: selector,
            count: elements.length,
            visible: Array.from(elements).some(el => this.isElementVisible(el))
          });
        }
      } catch (error) {
        console.warn('⚠️ Invalid selector:', selector, error);
      }
    }
    
    return indicators;
  }

  checkPaywallText() {
    const indicators = [];
    const bodyText = document.body.textContent?.toLowerCase() || '';
    
    const textIndicators = this.paywallIndicators.filter(indicator => !indicator.startsWith('.') && !indicator.startsWith('['));
    
    for (const indicator of textIndicators) {
      if (bodyText.includes(indicator.toLowerCase())) {
        indicators.push({
          type: 'text_content',
          value: indicator,
          found: true
        });
      }
    }
    
    return indicators;
  }

  checkContentTruncation() {
    const indicators = [];
    
    // Chercher des signes de contenu tronqué
    const truncationPatterns = [
      /\.\.\.\s*continue reading/i,
      /\.\.\.\s*read more/i,
      /\.\.\.\s*subscribe/i,
      /\.\.\.\s*sign up/i,
      /continue reading/i,
      /read the full article/i,
      /see the rest/i
    ];
    
    const text = document.body.textContent || '';
    
    for (const pattern of truncationPatterns) {
      if (pattern.test(text)) {
        indicators.push({
          type: 'content_truncation',
          pattern: pattern.source,
          found: true
        });
      }
    }
    
    // Vérifier si le contenu semble anormalement court pour un article
    const articleElements = document.querySelectorAll('article, .article, .post, .content');
    for (const article of articleElements) {
      const textLength = article.textContent?.length || 0;
      const paragraphs = article.querySelectorAll('p').length;
      
      if (textLength < 500 && paragraphs < 3) {
        indicators.push({
          type: 'suspiciously_short',
          textLength,
          paragraphs,
          threshold: 'less than 500 chars or 3 paragraphs'
        });
      }
    }
    
    return indicators;
  }

  checkPaywallOverlays() {
    const indicators = [];
    
    // Chercher des overlays/modals potentiels
    const overlaySelectors = [
      '[style*="position: fixed"]',
      '[style*="position: absolute"]',
      '.modal',
      '.overlay',
      '.popup',
      '[role="dialog"]'
    ];
    
    for (const selector of overlaySelectors) {
      const elements = document.querySelectorAll(selector);
      for (const element of elements) {
        if (this.isElementVisible(element) && this.elementContainsPaywallText(element)) {
          indicators.push({
            type: 'paywall_overlay',
            selector,
            text: element.textContent?.slice(0, 100) + '...'
          });
        }
      }
    }
    
    return indicators;
  }

  elementContainsPaywallText(element) {
    const text = element.textContent?.toLowerCase() || '';
    const paywalKeywords = ['subscribe', 'premium', 'unlock', 'member', 'sign up', 'pay'];
    return paywalKeywords.some(keyword => text.includes(keyword));
  }

  isElementVisible(element) {
    const style = window.getComputedStyle(element);
    return style.display !== 'none' && 
           style.visibility !== 'hidden' && 
           style.opacity !== '0';
  }

  // === EXTRACTION CONTENU PUBLIC ===

  extractPublicContent() {
    console.log('📄 Extracting public content from paywalled page...');
    
    const publicContent = {
      title: this.extractTitle(),
      excerpt: this.extractExcerpt(),
      metadata: this.extractMetadata(),
      headings: this.extractHeadings(),
      summary: this.extractSummary()
    };
    
    return publicContent;
  }

  extractTitle() {
    // Priorité: meta title, h1, title tag
    const metaTitle = document.querySelector('meta[property="og:title"]')?.content ||
                     document.querySelector('meta[name="title"]')?.content;
    
    if (metaTitle) return metaTitle;
    
    const h1 = document.querySelector('h1');
    if (h1) return h1.textContent?.trim();
    
    return document.title;
  }

  extractExcerpt() {
    const excerptSelectors = [
      'meta[property="og:description"]',
      'meta[name="description"]',
      '.excerpt',
      '.summary',
      '.teaser',
      '.lead',
      '.intro'
    ];
    
    for (const selector of excerptSelectors) {
      const element = document.querySelector(selector);
      if (element) {
        const content = element.content || element.textContent;
        if (content && content.trim().length > 50) {
          return content.trim();
        }
      }
    }
    
    return null;
  }

  extractMetadata() {
    const metadata = {};
    
    // Auteur
    const author = document.querySelector('meta[name="author"]')?.content ||
                  document.querySelector('[rel="author"]')?.textContent ||
                  document.querySelector('.author')?.textContent;
    if (author) metadata.author = author.trim();
    
    // Date de publication
    const publishDate = document.querySelector('meta[property="article:published_time"]')?.content ||
                       document.querySelector('time[pubdate]')?.datetime ||
                       document.querySelector('.publish-date')?.textContent;
    if (publishDate) metadata.publishDate = publishDate.trim();
    
    // Tags/catégories
    const tags = Array.from(document.querySelectorAll('meta[property="article:tag"]'))
                      .map(el => el.content);
    if (tags.length > 0) metadata.tags = tags;
    
    return metadata;
  }

  extractHeadings() {
    const headings = [];
    const headingElements = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    
    for (const heading of headingElements) {
      const text = heading.textContent?.trim();
      if (text && text.length > 10) {
        headings.push({
          level: parseInt(heading.tagName.charAt(1)),
          text: text
        });
      }
    }
    
    return headings.slice(0, 10); // Limiter à 10 titres
  }

  extractSummary() {
    // Essayer d'extraire les premiers paragraphes visibles
    const paragraphs = [];
    const pElements = document.querySelectorAll('p');
    
    for (const p of pElements) {
      if (this.isElementVisible(p)) {
        const text = p.textContent?.trim();
        if (text && text.length > 50 && !this.isPaywallParagraph(text)) {
          paragraphs.push(text);
          if (paragraphs.length >= 3) break; // Max 3 paragraphes
        }
      }
    }
    
    return paragraphs.join('\n\n');
  }

  isPaywallParagraph(text) {
    const paywallKeywords = ['subscribe', 'premium', 'member', 'unlock', 'sign up'];
    const lowerText = text.toLowerCase();
    return paywallKeywords.some(keyword => lowerText.includes(keyword));
  }

  // === GESTION PDF ===

  async handlePDF() {
    console.log('📄 Handling PDF content...');
    
    try {
      // Méthode 1: Essayer de récupérer la sélection
      const selection = await this.tryPDFSelection();
      if (selection && selection.length > 50) {
        return {
          method: 'pdf_selection',
          content: selection,
          isPDF: true,
          success: true
        };
      }
      
      // Méthode 2: Essayer d'extraire le texte visible
      const visibleText = this.extractPDFVisibleText();
      if (visibleText && visibleText.length > 100) {
        return {
          method: 'pdf_visible_text',
          content: visibleText,
          isPDF: true,
          success: true
        };
      }
      
      // Méthode 3: Fallback vers AWCS OCR
      return {
        method: 'pdf_fallback_ocr',
        content: '[PDF détecté - Extraction OCR recommandée via AWCS]',
        isPDF: true,
        success: false,
        fallbackNeeded: true
      };
      
    } catch (error) {
      console.error('❌ PDF handling error:', error);
      return {
        method: 'pdf_error',
        content: '[Erreur lors de l\'extraction PDF]',
        isPDF: true,
        success: false,
        error: error.message
      };
    }
  }

  async tryPDFSelection() {
    try {
      const selection = window.getSelection();
      if (selection && selection.toString().trim().length > 50) {
        console.log('✅ PDF selection found:', selection.toString().length, 'characters');
        return selection.toString().trim();
      }
    } catch (error) {
      console.warn('⚠️ PDF selection failed:', error);
    }
    
    return null;
  }

  extractPDFVisibleText() {
    // Essayer d'extraire le texte des éléments spécifiques aux viewers PDF
    const pdfTextSelectors = [
      '.textLayer', // PDF.js
      '.pdf-text',
      '[data-pdf-text]',
      '.page-text'
    ];
    
    let extractedText = '';
    
    for (const selector of pdfTextSelectors) {
      const elements = document.querySelectorAll(selector);
      for (const element of elements) {
        const text = element.textContent?.trim();
        if (text) {
          extractedText += text + '\n';
        }
      }
    }
    
    if (extractedText.length > 100) {
      console.log('✅ PDF visible text extracted:', extractedText.length, 'characters');
      return extractedText.trim();
    }
    
    return null;
  }

  // === API PUBLIQUE ===

  getPageAnalysis() {
    const analysis = {
      timestamp: Date.now(),
      url: window.location.href,
      pdf: this.isPDF(),
      paywall: this.isPaywalled(),
      extractionRecommendation: null
    };
    
    // Recommandation d'extraction
    if (analysis.pdf.isPDF) {
      analysis.extractionRecommendation = 'pdf_extraction';
    } else if (analysis.paywall.isPaywalled) {
      analysis.extractionRecommendation = 'public_content_only';
    } else {
      analysis.extractionRecommendation = 'full_extraction';
    }
    
    return analysis;
  }

  async getOptimalContent() {
    const analysis = this.getPageAnalysis();
    
    if (analysis.pdf.isPDF) {
      return await this.handlePDF();
    } else if (analysis.paywall.isPaywalled) {
      return {
        method: 'paywall_public_content',
        content: this.formatPublicContent(analysis.paywall.publicContent),
        isPaywalled: true,
        warning: 'Contenu protégé détecté. Seul le contenu public a été extrait.'
      };
    } else {
      return {
        method: 'standard_extraction',
        content: null, // Sera géré par l'extracteur principal
        shouldProceed: true
      };
    }
  }

  formatPublicContent(publicContent) {
    if (!publicContent) return 'Aucun contenu public disponible.';
    
    const parts = [];
    
    if (publicContent.title) {
      parts.push(`# ${publicContent.title}`);
    }
    
    if (publicContent.metadata?.author) {
      parts.push(`**Auteur:** ${publicContent.metadata.author}`);
    }
    
    if (publicContent.metadata?.publishDate) {
      parts.push(`**Date:** ${publicContent.metadata.publishDate}`);
    }
    
    if (publicContent.excerpt) {
      parts.push(`**Résumé:** ${publicContent.excerpt}`);
    }
    
    if (publicContent.headings && publicContent.headings.length > 0) {
      parts.push('**Titres:**');
      publicContent.headings.forEach(heading => {
        const indent = '  '.repeat(heading.level - 1);
        parts.push(`${indent}- ${heading.text}`);
      });
    }
    
    if (publicContent.summary) {
      parts.push('**Contenu visible:**');
      parts.push(publicContent.summary);
    }
    
    parts.push('\n*[Contenu complet disponible avec abonnement]*');
    
    return parts.join('\n\n');
  }
}

// Instance globale
window.pdfPaywallHandler = new PDFPaywallHandler();

// API pour les autres scripts
window.GRAVIS_PDF_PAYWALL = {
  isPDF: () => window.pdfPaywallHandler.isPDF(),
  isPaywalled: () => window.pdfPaywallHandler.isPaywalled(),
  handlePDF: () => window.pdfPaywallHandler.handlePDF(),
  getAnalysis: () => window.pdfPaywallHandler.getPageAnalysis(),
  getOptimalContent: () => window.pdfPaywallHandler.getOptimalContent()
};

console.log('✅ PDF & Paywall Handler ready');