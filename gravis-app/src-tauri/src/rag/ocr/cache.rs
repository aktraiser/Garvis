// GRAVIS OCR - Cache système pour résultats Tesseract
// Phase 2: Cache Blake3 + LRU optimisé pour Command-based approach

use super::{OcrResult, Result};
use blake3;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tracing::{debug, info};

/// Configuration du cache OCR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size_mb: usize,           // Cache LRU 256MB
    pub ttl_hours: u64,              // TTL en heures
    pub persistent: bool,            // Sauvegarder sur disque
    pub cache_directory: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 256,
            ttl_hours: 24,
            persistent: false, // En mémoire par défaut
            cache_directory: None,
        }
    }
}

/// Entrée de cache avec timestamp et métadonnées
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    result: OcrResult,
    created_at: SystemTime,
    access_count: u64,
    file_size: u64,
    file_modified: SystemTime,
    file_hash: String,
}

impl CacheEntry {
    fn new(result: OcrResult, file_size: u64, file_modified: SystemTime, file_hash: String) -> Self {
        Self {
            result,
            created_at: SystemTime::now(),
            access_count: 1,
            file_size,
            file_modified,
            file_hash,
        }
    }
    
    fn is_fresh(&self, ttl_hours: u64) -> bool {
        let ttl = Duration::from_secs(ttl_hours * 3600);
        self.created_at.elapsed().unwrap_or(Duration::MAX) < ttl
    }
    
    fn is_valid_for_file(&self, file_size: u64, file_modified: SystemTime, file_hash: &str) -> bool {
        self.file_size == file_size && 
        self.file_modified == file_modified &&
        self.file_hash == file_hash
    }
    
    fn touch(&mut self) {
        self.access_count += 1;
    }
}

/// Cache principal OCR
pub struct OcrCache {
    config: CacheConfig,
    memory_cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    stats: Arc<Mutex<CacheStats>>,
}

/// Statistiques du cache
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: u64,
    pub entries_count: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
    
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / 1_048_576.0
    }
}

impl OcrCache {
    /// Créer un nouveau cache OCR
    pub async fn new(config: CacheConfig) -> Result<Self> {
        if !config.enabled {
            info!("📋 OCR cache disabled");
            return Ok(Self {
                config,
                memory_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1).unwrap()))),
                stats: Arc::new(Mutex::new(CacheStats::default())),
            });
        }
        
        // Calculer la capacité du cache basée sur la taille mémoire
        let estimated_entry_size = 50_000; // ~50KB par entrée en moyenne
        let max_entries = (config.max_size_mb * 1_048_576) / estimated_entry_size;
        let capacity = NonZeroUsize::new(max_entries.max(10)).unwrap();
        
        info!("✅ OCR cache initialized: {}MB, ~{} entries", 
              config.max_size_mb, capacity);
        
        Ok(Self {
            config,
            memory_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            stats: Arc::new(Mutex::new(CacheStats::default())),
        })
    }
    
    /// Générer une clé de cache pour un fichier image
    #[allow(dead_code)]
    async fn generate_cache_key(&self, image_path: &Path, languages: &[String]) -> Result<String> {
        // Hash du contenu du fichier
        let content = fs::read(image_path).await?;
        let file_hash = blake3::hash(&content);
        
        // Métadonnées du fichier
        let metadata = fs::metadata(image_path).await?;
        let file_size = metadata.len();
        let modified_time = metadata.modified().unwrap_or(UNIX_EPOCH);
        let modified_timestamp = modified_time.duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        // Combinaison des paramètres
        let languages_str = languages.join(",");
        let key_content = format!("{}:{}:{}:{}", 
                                 file_hash.to_hex(), file_size, modified_timestamp, languages_str);
        
        // Hash final pour la clé
        let cache_key = blake3::hash(key_content.as_bytes());
        Ok(cache_key.to_hex().to_string())
    }
    
    /// Calculer le hash rapide d'un fichier (sans le lire entièrement)
    async fn quick_file_hash(&self, image_path: &Path) -> Result<String> {
        let metadata = fs::metadata(image_path).await?;
        let file_size = metadata.len();
        let modified_time = metadata.modified().unwrap_or(UNIX_EPOCH);
        let modified_timestamp = modified_time.duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        // Hash basé sur path + taille + date modif (plus rapide)
        let path_str = image_path.to_string_lossy();
        let quick_content = format!("{}:{}:{}", path_str, file_size, modified_timestamp);
        let hash = blake3::hash(quick_content.as_bytes());
        
        Ok(hash.to_hex().to_string())
    }
    
    /// Récupérer un résultat depuis le cache
    pub async fn get_image_result(&self, image_path: &Path) -> Result<Option<OcrResult>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        // Obtenir métadonnées fichier pour validation
        let metadata = match fs::metadata(image_path).await {
            Ok(m) => m,
            Err(_) => return Ok(None), // Fichier inexistant
        };
        
        let file_size = metadata.len();
        let file_modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        
        // Générer clé de cache rapide
        let quick_hash = self.quick_file_hash(image_path).await?;
        
        // Rechercher dans le cache mémoire
        if let Ok(mut cache) = self.memory_cache.lock() {
            // Chercher avec différentes variantes de clés
            let possible_keys = vec![
                quick_hash.clone(),
                format!("{}:eng", quick_hash),
                format!("{}:fra", quick_hash),
                format!("{}:eng,fra", quick_hash),
            ];
            
            for key in possible_keys {
                if let Some(entry) = cache.get_mut(&key) {
                    // Vérifier la validité (TTL + modifications fichier)
                    if entry.is_fresh(self.config.ttl_hours) && 
                       entry.is_valid_for_file(file_size, file_modified, &quick_hash) {
                        
                        entry.touch();
                        
                        // Mettre à jour les stats
                        if let Ok(mut stats) = self.stats.lock() {
                            stats.hits += 1;
                        }
                        
                        debug!("✅ Cache hit for image: {:?}", image_path);
                        return Ok(Some(entry.result.clone()));
                    } else {
                        // Entrée expirée ou fichier modifié
                        cache.pop(&key);
                        debug!("🗑️ Cache entry expired/invalid for: {:?}", image_path);
                    }
                }
            }
        }
        
        // Cache miss
        if let Ok(mut stats) = self.stats.lock() {
            stats.misses += 1;
        }
        
        debug!("❌ Cache miss for image: {:?}", image_path);
        Ok(None)
    }
    
    /// Stocker un résultat dans le cache
    pub async fn store_image_result(&self, image_path: &Path, result: &OcrResult) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Obtenir métadonnées fichier
        let metadata = fs::metadata(image_path).await?;
        let file_size = metadata.len();
        let file_modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        
        // Générer clé et hash
        let quick_hash = self.quick_file_hash(image_path).await?;
        let cache_key = format!("{}:{}", quick_hash, result.language);
        
        // Créer entrée de cache
        let entry = CacheEntry::new(result.clone(), file_size, file_modified, quick_hash);
        
        // Estimer la taille de l'entrée
        let entry_size = Self::estimate_entry_size(&entry);
        
        // Stocker dans le cache mémoire
        if let Ok(mut cache) = self.memory_cache.lock() {
            if let Some(_evicted) = cache.put(cache_key.clone(), entry) {
                // Une entrée a été évincée
                if let Ok(mut stats) = self.stats.lock() {
                    stats.evictions += 1;
                }
            }
            
            // Mettre à jour les stats
            if let Ok(mut stats) = self.stats.lock() {
                stats.entries_count = cache.len();
                stats.memory_usage_bytes += entry_size;
            }
        }
        
        debug!("💾 Cached result for image: {:?} ({}KB)", 
               image_path, entry_size / 1024);
        
        Ok(())
    }
    
    /// Estimer la taille mémoire d'une entrée
    fn estimate_entry_size(entry: &CacheEntry) -> u64 {
        // Estimation basique : texte + métadonnées + bounding boxes
        let text_size = entry.result.text.len() as u64;
        let bbox_size = entry.result.bounding_boxes.len() as u64 * 150; // ~150 bytes par bbox
        let metadata_overhead = 2000; // Overhead fixe plus important
        
        text_size + bbox_size + metadata_overhead
    }
    
    /// Obtenir les statistiques du cache
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }
    
    /// Vider le cache
    pub fn clear(&self) -> Result<()> {
        if let Ok(mut cache) = self.memory_cache.lock() {
            cache.clear();
        }
        
        if let Ok(mut stats) = self.stats.lock() {
            *stats = CacheStats::default();
        }
        
        info!("🗑️ OCR cache cleared");
        Ok(())
    }
    
    /// Optimiser le cache (nettoyer les entrées expirées)
    pub fn optimize(&self) -> Result<u32> {
        if !self.config.enabled {
            return Ok(0);
        }
        
        let mut removed_count = 0;
        
        if let Ok(mut cache) = self.memory_cache.lock() {
            let mut keys_to_remove = Vec::new();
            
            // Identifier les entrées expirées
            for (key, entry) in cache.iter() {
                if !entry.is_fresh(self.config.ttl_hours) {
                    keys_to_remove.push(key.clone());
                }
            }
            
            // Supprimer les entrées expirées
            for key in keys_to_remove {
                cache.pop(&key);
                removed_count += 1;
            }
            
            // Mettre à jour les stats
            if let Ok(mut stats) = self.stats.lock() {
                stats.entries_count = cache.len();
                stats.evictions += removed_count as u64;
            }
        }
        
        if removed_count > 0 {
            info!("🧹 Cache optimized: removed {} expired entries", removed_count);
        }
        
        Ok(removed_count)
    }
    
    /// Obtenir les informations détaillées du cache  
    pub fn get_cache_info(&self) -> HashMap<String, serde_json::Value> {
        let mut info = HashMap::new();
        
        if let Ok(stats) = self.stats.lock() {
            info.insert("enabled".to_string(), serde_json::Value::Bool(self.config.enabled));
            info.insert("max_size_mb".to_string(), serde_json::Value::Number(self.config.max_size_mb.into()));
            info.insert("ttl_hours".to_string(), serde_json::Value::Number(self.config.ttl_hours.into()));
            info.insert("hits".to_string(), serde_json::Value::Number(stats.hits.into()));
            info.insert("misses".to_string(), serde_json::Value::Number(stats.misses.into()));
            info.insert("hit_rate".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(stats.hit_rate()).unwrap_or(serde_json::Number::from(0))
            ));
            info.insert("entries_count".to_string(), serde_json::Value::Number(stats.entries_count.into()));
            info.insert("memory_usage_mb".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(stats.memory_usage_mb()).unwrap_or(serde_json::Number::from(0))
            ));
        }
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_cache_creation() {
        let config = CacheConfig::default();
        let cache = OcrCache::new(config).await.unwrap();
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        
        println!("✅ Cache creation test passed");
    }
    
    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = CacheConfig::default();
        let cache = OcrCache::new(config).await.unwrap();
        
        // Créer un fichier temporaire
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Écrire du contenu
        std::fs::write(temp_path, "test content").unwrap();
        
        let hash1 = cache.quick_file_hash(temp_path).await.unwrap();
        let hash2 = cache.quick_file_hash(temp_path).await.unwrap();
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // Blake3 hex = 64 chars
        
        println!("✅ Cache key generation test passed: {}", hash1);
    }
    
    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        
        assert_eq!(stats.hit_rate(), 0.0);
        
        stats.hits = 8;
        stats.misses = 2;
        
        assert_eq!(stats.hit_rate(), 0.8);
        
        println!("✅ Cache stats test passed");
    }
    
    #[tokio::test]
    async fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        
        let cache = OcrCache::new(config).await.unwrap();
        
        // Créer un fichier temporaire
        let temp_file = NamedTempFile::new().unwrap();
        let result = cache.get_image_result(temp_file.path()).await.unwrap();
        
        assert!(result.is_none());
        println!("✅ Disabled cache test passed");
    }
}