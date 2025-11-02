// GRAVIS Extension - Intelligent Cache Manager
// Phase 1: Cache intelligent pour optimiser les performances

console.log('💾 Cache Manager loading...');

class IntelligentCacheManager {
  constructor() {
    this.cache = new Map();
    this.urlHashes = new Map();
    this.metrics = {
      hits: 0,
      misses: 0,
      evictions: 0,
      totalRequests: 0
    };
    
    // Configuration
    this.config = {
      maxEntries: 100,           // Nombre max d'entrées
      maxSizeBytes: 10 * 1024 * 1024, // 10MB max
      defaultTTL: 5 * 60 * 1000,      // 5 minutes
      shortTTL: 2 * 60 * 1000,        // 2 minutes (pages dynamiques)
      longTTL: 30 * 60 * 1000,        // 30 minutes (pages statiques)
      cleanupInterval: 2 * 60 * 1000   // Cleanup toutes les 2 minutes
    };
    
    this.init();
  }

  async init() {
    await this.loadPersistedCache();
    this.startCleanupTimer();
    this.setupStorageListeners();
    console.log('✅ Intelligent Cache Manager initialized');
  }

  async loadPersistedCache() {
    try {
      const result = await chrome.storage.local.get(['gravis_cache_data', 'gravis_cache_metrics']);
      
      if (result.gravis_cache_data) {
        const cacheData = JSON.parse(result.gravis_cache_data);
        const now = Date.now();
        
        // Restaurer les entrées non expirées
        for (const [key, entry] of Object.entries(cacheData)) {
          if (entry.expiresAt > now) {
            this.cache.set(key, entry);
          }
        }
        
        console.log(`📦 Restored ${this.cache.size} cache entries from storage`);
      }
      
      if (result.gravis_cache_metrics) {
        this.metrics = { ...this.metrics, ...JSON.parse(result.gravis_cache_metrics) };
      }
      
    } catch (error) {
      console.warn('⚠️ Failed to load persisted cache:', error);
    }
  }

  async persistCache() {
    try {
      const cacheData = {};
      const now = Date.now();
      
      // Sauvegarder seulement les entrées non expirées
      for (const [key, entry] of this.cache.entries()) {
        if (entry.expiresAt > now) {
          cacheData[key] = entry;
        }
      }
      
      await chrome.storage.local.set({
        'gravis_cache_data': JSON.stringify(cacheData),
        'gravis_cache_metrics': JSON.stringify(this.metrics)
      });
      
    } catch (error) {
      console.warn('⚠️ Failed to persist cache:', error);
    }
  }

  startCleanupTimer() {
    setInterval(() => {
      this.cleanup();
    }, this.config.cleanupInterval);
  }

  setupStorageListeners() {
    // Persister le cache toutes les 30 secondes
    setInterval(() => {
      this.persistCache();
    }, 30000);
    
    // Persister avant fermeture
    window.addEventListener('beforeunload', () => {
      this.persistCache();
    });
  }

  // === GESTION DU CACHE ===

  generateKey(url, options = {}) {
    // Normaliser l'URL
    const normalizedUrl = this.normalizeUrl(url);
    
    // Inclure les options dans la clé si nécessaire
    const keyParts = [normalizedUrl];
    
    if (options.mode) keyParts.push(`mode:${options.mode}`);
    if (options.includeMetadata) keyParts.push('meta:true');
    if (options.userAgent) keyParts.push(`ua:${this.hashString(options.userAgent)}`);
    
    return keyParts.join('|');
  }

  normalizeUrl(url) {
    try {
      const urlObj = new URL(url);
      
      // Supprimer les paramètres de tracking courants
      const trackingParams = [
        'utm_source', 'utm_medium', 'utm_campaign', 'utm_content', 'utm_term',
        'fbclid', 'gclid', 'ref', 'source', 'campaign',
        '_ga', '_gid', 'mc_cid', 'mc_eid'
      ];
      
      trackingParams.forEach(param => {
        urlObj.searchParams.delete(param);
      });
      
      // Normaliser le hash (supprimer si c'est juste pour le tracking)
      if (urlObj.hash && !this.isImportantHash(urlObj.hash)) {
        urlObj.hash = '';
      }
      
      return urlObj.toString();
    } catch {
      return url;
    }
  }

  isImportantHash(hash) {
    // Garder les hash qui semblent être des ancres de contenu
    const importantPatterns = [
      /^#[a-zA-Z]/, // Commence par une lettre
      /^#section/, 
      /^#chapter/,
      /^#heading/
    ];
    
    return importantPatterns.some(pattern => pattern.test(hash));
  }

  hashString(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(36);
  }

  calculateContentHash(content) {
    // Hash simple du contenu pour détecter les changements
    return this.hashString(content.title + content.mainContent.slice(0, 1000));
  }

  determineContentType(url, content) {
    const urlLower = url.toLowerCase();
    const titleLower = (content.title || '').toLowerCase();
    const contentLower = (content.mainContent || '').toLowerCase();
    
    // Articles de news
    if (urlLower.includes('/news/') || 
        urlLower.includes('/article/') ||
        titleLower.includes('breaking') ||
        contentLower.includes('published') ||
        contentLower.includes('updated')) {
      return 'news';
    }
    
    // Documentation
    if (urlLower.includes('/docs/') ||
        urlLower.includes('/documentation/') ||
        urlLower.includes('/api/') ||
        titleLower.includes('documentation') ||
        contentLower.includes('api reference')) {
      return 'documentation';
    }
    
    // Réseaux sociaux
    if (urlLower.includes('twitter.com') ||
        urlLower.includes('facebook.com') ||
        urlLower.includes('linkedin.com') ||
        urlLower.includes('instagram.com')) {
      return 'social';
    }
    
    // E-commerce
    if (urlLower.includes('/product/') ||
        urlLower.includes('/shop/') ||
        contentLower.includes('add to cart') ||
        contentLower.includes('price')) {
      return 'ecommerce';
    }
    
    // Pages statiques
    if (urlLower.includes('/about') ||
        urlLower.includes('/contact') ||
        urlLower.includes('/privacy') ||
        urlLower.includes('/terms')) {
      return 'static';
    }
    
    return 'general';
  }

  calculateTTL(url, content, contentType) {
    switch (contentType) {
      case 'news':
        return this.config.shortTTL; // News change rapidement
        
      case 'social':
        return this.config.shortTTL / 2; // Réseaux sociaux très dynamiques
        
      case 'documentation':
        return this.config.longTTL; // Documentation stable
        
      case 'static':
        return this.config.longTTL * 2; // Pages statiques très stables
        
      case 'ecommerce':
        return this.config.defaultTTL; // Prix peuvent changer
        
      default:
        return this.config.defaultTTL;
    }
  }

  calculateSize(entry) {
    return JSON.stringify(entry).length;
  }

  shouldCache(url, content) {
    // Ne pas cacher si le contenu est trop petit
    if (!content.mainContent || content.mainContent.length < 100) {
      return false;
    }
    
    // Ne pas cacher les pages d'erreur
    if (content.title && content.title.toLowerCase().includes('error')) {
      return false;
    }
    
    // Ne pas cacher les pages de login/auth
    const urlLower = url.toLowerCase();
    if (urlLower.includes('/login') || 
        urlLower.includes('/auth') ||
        urlLower.includes('/signin')) {
      return false;
    }
    
    return true;
  }

  // === API PRINCIPALE ===

  async get(url, options = {}) {
    const key = this.generateKey(url, options);
    this.metrics.totalRequests++;
    
    const entry = this.cache.get(key);
    
    if (!entry) {
      this.metrics.misses++;
      console.log('🔍 Cache miss for:', url);
      return null;
    }
    
    // Vérifier l'expiration
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      this.metrics.misses++;
      console.log('⏰ Cache expired for:', url);
      return null;
    }
    
    // Mettre à jour les statistiques d'accès
    entry.lastAccessed = Date.now();
    entry.accessCount++;
    
    this.metrics.hits++;
    console.log('🎯 Cache hit for:', url, `(${entry.accessCount} accesses)`);
    
    return {
      ...entry.content,
      _fromCache: true,
      _cachedAt: entry.createdAt,
      _accessCount: entry.accessCount
    };
  }

  async set(url, content, options = {}) {
    if (!this.shouldCache(url, content)) {
      console.log('🚫 Skipping cache for:', url, '(content not suitable)');
      return false;
    }
    
    const key = this.generateKey(url, options);
    const contentType = this.determineContentType(url, content);
    const ttl = options.ttl || this.calculateTTL(url, content, contentType);
    const contentHash = this.calculateContentHash(content);
    
    const entry = {
      content: { ...content },
      createdAt: Date.now(),
      expiresAt: Date.now() + ttl,
      lastAccessed: Date.now(),
      accessCount: 1,
      contentType,
      contentHash,
      url: url,
      size: 0 // Sera calculé après
    };
    
    entry.size = this.calculateSize(entry);
    
    // Vérifier si une entrée existe déjà avec le même hash
    const existingEntry = this.cache.get(key);
    if (existingEntry && existingEntry.contentHash === contentHash) {
      // Le contenu n'a pas changé, juste mettre à jour l'expiration
      existingEntry.expiresAt = Date.now() + ttl;
      existingEntry.lastAccessed = Date.now();
      console.log('🔄 Updated cache expiration for:', url);
      return true;
    }
    
    // Vérifier les limites avant d'ajouter
    await this.ensureSpace(entry.size);
    
    this.cache.set(key, entry);
    
    console.log(`💾 Cached content for: ${url} (${contentType}, ${Math.round(ttl/1000)}s TTL, ${entry.size} bytes)`);
    
    return true;
  }

  async ensureSpace(newEntrySize) {
    const currentSize = this.getTotalSize();
    const maxSize = this.config.maxSizeBytes;
    
    // Vérifier la taille totale
    if (currentSize + newEntrySize > maxSize) {
      console.log(`🧹 Cache cleanup needed: ${currentSize + newEntrySize} > ${maxSize} bytes`);
      await this.evictOldEntries(newEntrySize);
    }
    
    // Vérifier le nombre d'entrées
    if (this.cache.size >= this.config.maxEntries) {
      console.log(`🧹 Cache cleanup needed: ${this.cache.size} >= ${this.config.maxEntries} entries`);
      await this.evictOldEntries(0);
    }
  }

  async evictOldEntries(spaceNeeded) {
    const entries = Array.from(this.cache.entries());
    
    // Trier par score de priorité (LRU + fréquence + type)
    entries.sort(([keyA, entryA], [keyB, entryB]) => {
      const scoreA = this.calculateEvictionScore(entryA);
      const scoreB = this.calculateEvictionScore(entryB);
      return scoreA - scoreB; // Score plus bas = éviction en premier
    });
    
    let freedSpace = 0;
    let evictedCount = 0;
    
    for (const [key, entry] of entries) {
      if (freedSpace >= spaceNeeded && this.cache.size < this.config.maxEntries * 0.8) {
        break; // Assez d'espace libéré
      }
      
      freedSpace += entry.size;
      this.cache.delete(key);
      evictedCount++;
      this.metrics.evictions++;
    }
    
    console.log(`🗑️ Evicted ${evictedCount} entries, freed ${freedSpace} bytes`);
  }

  calculateEvictionScore(entry) {
    const now = Date.now();
    const age = now - entry.createdAt;
    const timeSinceAccess = now - entry.lastAccessed;
    
    // Score basé sur:
    // - Age (plus ancien = score plus bas)
    // - Fréquence d'accès (plus accédé = score plus haut)
    // - Type de contenu (statique = score plus haut)
    // - Temps depuis dernier accès
    
    let score = 0;
    
    // Fréquence d'accès (0-100)
    score += Math.min(entry.accessCount * 10, 100);
    
    // Type de contenu
    const typeBonus = {
      'static': 50,
      'documentation': 40,
      'general': 20,
      'ecommerce': 10,
      'news': 5,
      'social': 0
    };
    score += typeBonus[entry.contentType] || 0;
    
    // Pénalité pour l'âge (0-50)
    const agePenalty = Math.min(age / (24 * 60 * 60 * 1000) * 10, 50);
    score -= agePenalty;
    
    // Pénalité pour temps sans accès (0-30)
    const accessPenalty = Math.min(timeSinceAccess / (60 * 60 * 1000) * 5, 30);
    score -= accessPenalty;
    
    return Math.max(0, score);
  }

  // === MAINTENANCE ===

  cleanup() {
    const now = Date.now();
    let cleanedCount = 0;
    
    for (const [key, entry] of this.cache.entries()) {
      if (now > entry.expiresAt) {
        this.cache.delete(key);
        cleanedCount++;
      }
    }
    
    if (cleanedCount > 0) {
      console.log(`🧹 Cleaned up ${cleanedCount} expired cache entries`);
    }
  }

  clear() {
    const size = this.cache.size;
    this.cache.clear();
    this.persistCache();
    console.log(`🗑️ Cleared all ${size} cache entries`);
  }

  // === STATISTIQUES ===

  getStats() {
    const totalSize = this.getTotalSize();
    const hitRate = this.metrics.totalRequests > 0 ? 
      (this.metrics.hits / this.metrics.totalRequests * 100).toFixed(1) : 0;
    
    return {
      entries: this.cache.size,
      maxEntries: this.config.maxEntries,
      totalSize: totalSize,
      maxSize: this.config.maxSizeBytes,
      usage: {
        entriesPercent: (this.cache.size / this.config.maxEntries * 100).toFixed(1),
        sizePercent: (totalSize / this.config.maxSizeBytes * 100).toFixed(1)
      },
      metrics: {
        ...this.metrics,
        hitRate: hitRate + '%'
      },
      byContentType: this.getStatsByContentType()
    };
  }

  getTotalSize() {
    let total = 0;
    for (const entry of this.cache.values()) {
      total += entry.size;
    }
    return total;
  }

  getStatsByContentType() {
    const stats = {};
    
    for (const entry of this.cache.values()) {
      const type = entry.contentType;
      if (!stats[type]) {
        stats[type] = { count: 0, totalSize: 0, avgAccess: 0 };
      }
      
      stats[type].count++;
      stats[type].totalSize += entry.size;
      stats[type].avgAccess = (stats[type].avgAccess + entry.accessCount) / stats[type].count;
    }
    
    return stats;
  }

  // === API PUBLIQUE ===

  async getOrExtract(url, extractorFunction, options = {}) {
    // Essayer le cache d'abord
    const cached = await this.get(url, options);
    if (cached) {
      return cached;
    }
    
    // Cache miss - extraire le contenu
    console.log('🔄 Cache miss, extracting fresh content for:', url);
    const content = await extractorFunction();
    
    if (content) {
      await this.set(url, content, options);
    }
    
    return content;
  }

  prefetch(urls) {
    // TODO: Implémentation du préchargement
    console.log('🚀 Prefetch requested for', urls.length, 'URLs');
  }

  invalidate(urlPattern) {
    let invalidatedCount = 0;
    const pattern = new RegExp(urlPattern);
    
    for (const [key, entry] of this.cache.entries()) {
      if (pattern.test(entry.url)) {
        this.cache.delete(key);
        invalidatedCount++;
      }
    }
    
    console.log(`🗑️ Invalidated ${invalidatedCount} cache entries matching: ${urlPattern}`);
    return invalidatedCount;
  }
}

// Instance globale
window.cacheManager = new IntelligentCacheManager();

// API pour les autres scripts
window.GRAVIS_CACHE = {
  get: (url, options) => window.cacheManager.get(url, options),
  set: (url, content, options) => window.cacheManager.set(url, content, options),
  getOrExtract: (url, extractor, options) => window.cacheManager.getOrExtract(url, extractor, options),
  clear: () => window.cacheManager.clear(),
  getStats: () => window.cacheManager.getStats(),
  invalidate: (pattern) => window.cacheManager.invalidate(pattern)
};

console.log('✅ Intelligent Cache Manager ready');