// GRAVIS Intelligent Content Extractor
// Phase 1: Mozilla Readability + JSON-LD + Table parsing + Content classification

console.log('🧠 Intelligent Extractor loaded');

/**
 * Lightweight Readability implementation
 */
class SmartReadability {
  constructor(doc) {
    this.doc = doc;
  }

  parse() {
    try {
      // Find main content using multiple strategies
      const strategies = [
        () => this.findBySemanticTags(),
        () => this.findByReadabilityScore(),
        () => this.findByContentSize()
      ];

      for (const strategy of strategies) {
        const result = strategy();
        if (result && result.content.length > 200) {
          return result;
        }
      }

      // Fallback to body
      return {
        title: this.doc.title || '',
        content: this.doc.body.textContent || '',
        textContent: this.doc.body.textContent || '',
        length: this.doc.body.textContent?.length || 0,
        byline: this.findByline(),
        siteName: this.findSiteName(),
        excerpt: (this.doc.body.textContent || '').substring(0, 280)
      };
    } catch (error) {
      console.error('❌ Readability parsing failed:', error);
      return null;
    }
  }

  findBySemanticTags() {
    const candidates = [
      this.doc.querySelector('main'),
      this.doc.querySelector('article'),
      this.doc.querySelector('[role="main"]'),
      this.doc.querySelector('.content'),
      this.doc.querySelector('#content'),
      this.doc.querySelector('.post-content'),
      this.doc.querySelector('.entry-content'),
      this.doc.querySelector('.article-content')
    ].filter(Boolean);

    for (const candidate of candidates) {
      const text = candidate.textContent.trim();
      if (text.length > 200) {
        return {
          title: this.doc.title || '',
          content: candidate.innerHTML,
          textContent: text,
          length: text.length,
          byline: this.findByline(),
          siteName: this.findSiteName(),
          excerpt: text.substring(0, 280)
        };
      }
    }
    return null;
  }

  findByReadabilityScore() {
    const paragraphs = Array.from(this.doc.querySelectorAll('p, div'));
    let bestParent = null;
    let bestScore = 0;

    paragraphs.forEach(p => {
      const text = p.textContent.trim();
      if (text.length < 50) return;

      const parent = p.parentElement;
      if (!parent) return;

      // Simple scoring based on text density
      const score = text.length + 
                   (text.match(/[.!?]/g) || []).length * 10 +
                   (text.match(/[,;]/g) || []).length * 2;

      if (score > bestScore) {
        bestScore = score;
        bestParent = parent;
      }
    });

    if (bestParent) {
      const text = bestParent.textContent.trim();
      return {
        title: this.doc.title || '',
        content: bestParent.innerHTML,
        textContent: text,
        length: text.length,
        byline: this.findByline(),
        siteName: this.findSiteName(),
        excerpt: text.substring(0, 280)
      };
    }
    return null;
  }

  findByContentSize() {
    const candidates = Array.from(this.doc.querySelectorAll('div, section, article'))
      .filter(el => {
        const text = el.textContent.trim();
        return text.length > 200 && text.length < 50000;
      })
      .sort((a, b) => b.textContent.length - a.textContent.length);

    if (candidates.length > 0) {
      const best = candidates[0];
      const text = best.textContent.trim();
      return {
        title: this.doc.title || '',
        content: best.innerHTML,
        textContent: text,
        length: text.length,
        byline: this.findByline(),
        siteName: this.findSiteName(),
        excerpt: text.substring(0, 280)
      };
    }
    return null;
  }

  findByline() {
    const selectors = [
      'a[rel="author"]',
      '*[itemprop*="author"]',
      '.byline',
      '.author',
      '*[class*="author"]',
      '*[id*="author"]'
    ];

    for (const selector of selectors) {
      const element = this.doc.querySelector(selector);
      if (element) {
        const text = element.textContent.trim();
        if (text && text.length < 100) {
          return text;
        }
      }
    }
    return '';
  }

  findSiteName() {
    const meta = this.doc.querySelector('meta[property="og:site_name"]');
    if (meta) return meta.getAttribute('content') || '';
    
    const title = this.doc.title || '';
    const parts = title.split(/[\|\-–—]/);
    if (parts.length > 1) {
      return parts[parts.length - 1].trim();
    }
    
    return new URL(window.location.href).hostname;
  }
}

/**
 * JSON-LD and Structured Data extractor
 */
class StructuredDataExtractor {
  constructor(doc) {
    this.doc = doc;
  }

  extract() {
    const structured = {
      jsonLd: this.extractJsonLd(),
      microdata: this.extractMicrodata(),
      opengraph: this.extractOpenGraph(),
      meta: this.extractMetaTags()
    };

    return structured;
  }

  extractJsonLd() {
    const scripts = this.doc.querySelectorAll('script[type="application/ld+json"]');
    const jsonLdData = [];

    scripts.forEach(script => {
      try {
        const data = JSON.parse(script.textContent);
        jsonLdData.push(data);
      } catch (error) {
        console.warn('⚠️ Invalid JSON-LD:', error);
      }
    });

    return jsonLdData;
  }

  extractMicrodata() {
    const microdata = {};
    const itemProps = this.doc.querySelectorAll('[itemprop]');

    itemProps.forEach(element => {
      const prop = element.getAttribute('itemprop');
      const content = element.getAttribute('content') || 
                     element.textContent.trim();
      
      if (prop && content) {
        microdata[prop] = content;
      }
    });

    return microdata;
  }

  extractOpenGraph() {
    const og = {};
    const metaTags = this.doc.querySelectorAll('meta[property^="og:"]');

    metaTags.forEach(meta => {
      const property = meta.getAttribute('property');
      const content = meta.getAttribute('content');
      if (property && content) {
        og[property] = content;
      }
    });

    return og;
  }

  extractMetaTags() {
    const meta = {};
    const metaTags = this.doc.querySelectorAll('meta[name]');

    metaTags.forEach(metaTag => {
      const name = metaTag.getAttribute('name');
      const content = metaTag.getAttribute('content');
      if (name && content) {
        meta[name] = content;
      }
    });

    return meta;
  }
}

/**
 * Table to JSON converter
 */
class TableExtractor {
  constructor(doc) {
    this.doc = doc;
  }

  extract() {
    const tables = this.doc.querySelectorAll('table');
    const tablesData = [];

    tables.forEach(table => {
      try {
        const tableData = this.tableToJson(table);
        if (tableData.rows.length > 0) {
          tablesData.push(tableData);
        }
      } catch (error) {
        console.warn('⚠️ Table extraction failed:', error);
      }
    });

    return tablesData;
  }

  tableToJson(table) {
    const rows = Array.from(table.querySelectorAll('tr'));
    const headers = [];
    const data = [];

    // Extract headers
    const headerRow = rows[0];
    if (headerRow) {
      const headerCells = headerRow.querySelectorAll('th, td');
      headerCells.forEach(cell => {
        headers.push(cell.textContent.trim());
      });
    }

    // Extract data rows
    for (let i = headers.length > 0 ? 1 : 0; i < rows.length; i++) {
      const row = rows[i];
      const cells = row.querySelectorAll('td, th');
      const rowData = [];

      cells.forEach(cell => {
        rowData.push(cell.textContent.trim());
      });

      if (rowData.some(cell => cell.length > 0)) {
        data.push(rowData);
      }
    }

    return {
      headers: headers.length > 0 ? headers : null,
      rows: data,
      rowCount: data.length,
      columnCount: headers.length || (data[0]?.length || 0)
    };
  }
}

/**
 * Content type classifier
 */
class ContentClassifier {
  constructor(doc, url, text, structured, tables) {
    this.doc = doc;
    this.url = url;
    this.text = text.toLowerCase();
    this.structured = structured;
    this.tables = tables;
  }

  classify() {
    // Check JSON-LD for explicit types
    if (this.structured.jsonLd) {
      for (const jsonLd of this.structured.jsonLd) {
        const type = this.getJsonLdType(jsonLd);
        if (type) return type;
      }
    }

    // Heuristic classification
    if (this.isCommercePage()) return 'commerce';
    if (this.isArticlePage()) return 'article';
    if (this.isTableDataset()) return 'table_dataset';
    if (this.isEmailLike()) return 'email_like';
    
    return 'generic';
  }

  getJsonLdType(jsonLd) {
    const type = Array.isArray(jsonLd) ? jsonLd[0]['@type'] : jsonLd['@type'];
    
    if (!type) return null;

    const typeStr = Array.isArray(type) ? type[0] : type;
    
    if (['Product', 'Offer', 'AggregateOffer'].includes(typeStr)) return 'commerce';
    if (['Article', 'NewsArticle', 'BlogPosting'].includes(typeStr)) return 'article';
    if (['Dataset', 'Table'].includes(typeStr)) return 'table_dataset';
    
    return null;
  }

  isCommercePage() {
    const commerceSignals = [
      // Currency symbols
      /[€$£¥₹]/,
      // Commerce terms
      /\b(prix|price|tarif|cost|coût|buy|acheter|commander|order|cart|panier|add to cart|ajouter au panier)\b/,
      // Currency codes
      /\b(eur|usd|gbp|cad|chf|jpy)\b/,
      // Shopping indicators
      /\b(stock|disponible|available|livraison|shipping|delivery)\b/
    ];

    return commerceSignals.some(pattern => pattern.test(this.text)) ||
           this.doc.querySelector('[data-price], .price, #price, .cost, .tarif');
  }

  isArticlePage() {
    const articleSignals = [
      /\b(article|author|auteur|published|publié|byline|reading time|temps de lecture)\b/,
      /\b(news|actualité|journal|magazine|blog|post|story|reportage)\b/
    ];

    return articleSignals.some(pattern => pattern.test(this.text)) ||
           this.doc.querySelector('article, .article, .post, .news, .blog-post, time[datetime]');
  }

  isTableDataset() {
    return this.tables.length > 0 && 
           this.tables.some(table => table.rowCount > 5) &&
           this.text.match(/\b(data|dataset|statistics|stats|tableau|données|results|résultats)\b/);
  }

  isEmailLike() {
    const emailSignals = [
      /\b(from|to|de|à|subject|objet|sent|envoyé|received|reçu)\b/,
      /@[\w.-]+\.[a-z]{2,}/,
      /\b(reply|répondre|forward|transférer|cc|bcc)\b/
    ];

    return emailSignals.some(pattern => pattern.test(this.text));
  }

  getConfidence() {
    // Return a confidence score between 0 and 1
    const hasStructuredData = this.structured.jsonLd.length > 0 || 
                             Object.keys(this.structured.microdata).length > 0;
    
    if (hasStructuredData) return 0.9;
    if (this.tables.length > 0) return 0.8;
    return 0.6;
  }
}

/**
 * Main intelligent extraction function
 */
window.intelligentExtraction = function(mode = 'auto') {
  console.log('🧠 Starting intelligent extraction, mode:', mode);
  
  try {
    // 1. Check for user selection first
    const selection = window.getSelection()?.toString()?.trim();
    if (selection && selection.length > 50) {
      console.log('✂️ Using user selection');
      return {
        page_type: 'user_selection',
        url: window.location.href,
        title: document.title,
        main_text: selection,
        structured: {},
        tables: [],
        meta: {
          extraction_method: 'user_selection',
          extraction_confidence: 1.0,
          language: document.documentElement.lang || 'unknown'
        }
      };
    }

    // 2. Smart content extraction using Readability
    const readability = new SmartReadability(document);
    const readabilityResult = readability.parse();
    
    if (!readabilityResult) {
      throw new Error('Readability extraction failed');
    }

    // 3. Extract structured data
    const structuredExtractor = new StructuredDataExtractor(document);
    const structured = structuredExtractor.extract();

    // 4. Extract tables
    const tableExtractor = new TableExtractor(document);
    const tables = tableExtractor.extract();

    // 5. Classify content type
    const classifier = new ContentClassifier(
      document, 
      window.location.href, 
      readabilityResult.textContent, 
      structured, 
      tables
    );
    const contentType = classifier.classify();
    const confidence = classifier.getConfidence();

    // 6. Build final structured payload
    const payload = {
      page_type: contentType,
      url: window.location.href,
      title: readabilityResult.title,
      main_text: readabilityResult.textContent,
      structured: {
        jsonld: structured.jsonLd,
        microdata: structured.microdata,
        opengraph: structured.opengraph
      },
      tables: tables,
      meta: {
        byline: readabilityResult.byline,
        site_name: readabilityResult.siteName,
        excerpt: readabilityResult.excerpt,
        word_count: readabilityResult.textContent.split(/\s+/).length,
        extraction_method: 'intelligent_' + contentType,
        extraction_confidence: confidence,
        language: document.documentElement.lang || 'unknown',
        has_structured_data: structured.jsonLd.length > 0 || Object.keys(structured.microdata).length > 0,
        table_count: tables.length
      }
    };

    console.log('✅ Intelligent extraction completed:', {
      type: contentType,
      confidence: confidence,
      wordCount: payload.meta.word_count,
      hasStructured: payload.meta.has_structured_data,
      tableCount: payload.meta.table_count
    });

    return payload;

  } catch (error) {
    console.error('❌ Intelligent extraction failed:', error);
    
    // Fallback to simple extraction
    const content = document.body.textContent.trim().slice(0, 10000);
    return {
      page_type: 'generic',
      url: window.location.href,
      title: document.title,
      main_text: content,
      structured: {},
      tables: [],
      meta: {
        extraction_method: 'fallback_body',
        extraction_confidence: 0.3,
        language: document.documentElement.lang || 'unknown',
        error: error.message
      }
    };
  }
};

console.log('✅ Intelligent Extractor ready');