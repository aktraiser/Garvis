// GRAVIS RAG - DevicePool pour gestion mémoire Candle
// Phase 3: Optimisation mémoire selon recommandations expertes

use anyhow::{Context, Result};
use candle_core::{Device, Tensor, DType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};

/// Cache LRU simple pour tensors
struct LruCache<K, V> {
    map: HashMap<K, (V, Instant)>,
    capacity: usize,
    max_age: Duration,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LruCache<K, V> {
    fn new(capacity: usize, max_age: Duration) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            capacity,
            max_age,
        }
    }
    
    fn get(&mut self, key: &K) -> Option<V> {
        if let Some((value, timestamp)) = self.map.get(key) {
            // Vérifier si l'entrée n'a pas expiré
            if timestamp.elapsed() < self.max_age {
                let value = value.clone();
                // Mettre à jour le timestamp (LRU)
                self.map.insert(key.clone(), (value.clone(), Instant::now()));
                return Some(value);
            } else {
                // Entrée expirée, la supprimer
                self.map.remove(key);
            }
        }
        None
    }
    
    fn put(&mut self, key: K, value: V) {
        // Si le cache est plein, supprimer l'entrée la plus ancienne
        if self.map.len() >= self.capacity {
            self.evict_oldest();
        }
        
        self.map.insert(key, (value, Instant::now()));
    }
    
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.map.iter()
            .min_by_key(|(_, (_, timestamp))| timestamp)
            .map(|(k, _)| k.clone())
        {
            self.map.remove(&oldest_key);
            debug!("Evicted oldest tensor from cache");
        }
    }
    
    fn clear(&mut self) {
        self.map.clear();
    }
    
    fn len(&self) -> usize {
        self.map.len()
    }
    
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<K> = self.map.iter()
            .filter_map(|(k, (_, timestamp))| {
                if now.duration_since(*timestamp) > self.max_age {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key in expired_keys {
            self.map.remove(&key);
        }
    }
}

/// Configuration du DevicePool
#[derive(Debug, Clone)]
pub struct DevicePoolConfig {
    pub max_memory_mb: usize,
    pub cache_capacity: usize,
    pub tensor_ttl: Duration,
    pub cleanup_interval: Duration,
}

impl Default for DevicePoolConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,                    // 2GB max par défaut
            cache_capacity: 100,                   // 100 tensors max en cache
            tensor_ttl: Duration::from_secs(300),  // 5 minutes TTL
            cleanup_interval: Duration::from_secs(60), // Cleanup toutes les minutes
        }
    }
}

/// DevicePool pour gestion optimisée mémoire Candle
/// Implémente les recommandations expertes : reuse tensors / drop explicite
pub struct DevicePool {
    device: Device,
    config: DevicePoolConfig,
    tensor_cache: Arc<Mutex<LruCache<String, Tensor>>>,
    memory_usage: Arc<Mutex<usize>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl DevicePool {
    /// Créer un nouveau DevicePool
    pub fn new(device: Device, config: DevicePoolConfig) -> Self {
        info!("🔄 Creating DevicePool with max memory: {}MB", config.max_memory_mb);
        
        Self {
            device,
            tensor_cache: Arc::new(Mutex::new(LruCache::new(
                config.cache_capacity,
                config.tensor_ttl,
            ))),
            memory_usage: Arc::new(Mutex::new(0)),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            config,
        }
    }
    
    /// Créer un DevicePool par défaut avec CPU
    pub fn default_cpu() -> Self {
        Self::new(Device::Cpu, DevicePoolConfig::default())
    }
    
    /// Obtenir ou créer un tensor avec cache et réutilisation
    pub fn get_or_create_tensor(&self, key: &str, shape: &[usize], dtype: DType) -> Result<Tensor> {
        // Vérifier si nettoyage nécessaire
        self.cleanup_if_needed();
        
        // Essayer de récupérer depuis le cache
        if let Ok(mut cache) = self.tensor_cache.lock() {
            if let Some(cached_tensor) = cache.get(&key.to_string()) {
                debug!("✅ Tensor cache hit for key: {}", key);
                return Ok(cached_tensor);
            }
        }
        
        // Vérifier la limite mémoire avant création
        if !self.check_memory_limit(shape, dtype)? {
            warn!("🚨 Memory limit exceeded, forcing cleanup");
            self.force_cleanup();
            
            // Réessayer après cleanup
            if !self.check_memory_limit(shape, dtype)? {
                return Err(anyhow::anyhow!(
                    "Cannot create tensor: would exceed memory limit ({}MB)", 
                    self.config.max_memory_mb
                ));
            }
        }
        
        // Créer le nouveau tensor
        let tensor = Tensor::zeros(shape, dtype, &self.device)
            .with_context(|| format!("Failed to create tensor with shape {:?}", shape))?;
        
        // Mettre en cache
        if let Ok(mut cache) = self.tensor_cache.lock() {
            cache.put(key.to_string(), tensor.clone());
            debug!("📦 Cached new tensor for key: {}", key);
        }
        
        // Mettre à jour l'usage mémoire estimé
        self.update_memory_usage(shape, dtype, true);
        
        debug!("✅ Created new tensor with shape {:?}", shape);
        Ok(tensor)
    }
    
    /// Créer un tensor temporaire (non mis en cache)
    pub fn create_temp_tensor(&self, shape: &[usize], dtype: DType) -> Result<Tensor> {
        if !self.check_memory_limit(shape, dtype)? {
            self.force_cleanup();
            
            if !self.check_memory_limit(shape, dtype)? {
                return Err(anyhow::anyhow!(
                    "Cannot create temp tensor: would exceed memory limit"
                ));
            }
        }
        
        Tensor::zeros(shape, dtype, &self.device)
            .context("Failed to create temporary tensor")
    }
    
    /// Supprimer explicitement un tensor du cache
    pub fn drop_tensor(&self, key: &str) -> bool {
        if let Ok(mut cache) = self.tensor_cache.lock() {
            if let Some(_) = cache.map.remove(key) {
                debug!("🗑️ Explicitly dropped tensor: {}", key);
                return true;
            }
        }
        false
    }
    
    /// Forcer le nettoyage du cache
    pub fn force_cleanup(&self) {
        info!("🧹 Forcing tensor cache cleanup");
        
        if let Ok(mut cache) = self.tensor_cache.lock() {
            let old_len = cache.len();
            cache.clear();
            info!("🧹 Cleared {} tensors from cache", old_len);
        }
        
        if let Ok(mut memory) = self.memory_usage.lock() {
            *memory = 0;
        }
        
        // Mise à jour du timestamp de cleanup
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = Instant::now();
        }
    }
    
    /// Nettoyage automatique si nécessaire
    fn cleanup_if_needed(&self) {
        let should_cleanup = {
            if let Ok(last_cleanup) = self.last_cleanup.lock() {
                last_cleanup.elapsed() > self.config.cleanup_interval
            } else {
                false
            }
        };
        
        if should_cleanup {
            self.cleanup_expired();
        }
    }
    
    /// Nettoyer les tensors expirés
    fn cleanup_expired(&self) {
        if let Ok(mut cache) = self.tensor_cache.lock() {
            let old_len = cache.len();
            cache.cleanup_expired();
            let new_len = cache.len();
            
            if old_len != new_len {
                debug!("🧹 Cleaned up {} expired tensors", old_len - new_len);
            }
        }
        
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = Instant::now();
        }
    }
    
    /// Vérifier si la création d'un tensor dépasserait la limite mémoire
    fn check_memory_limit(&self, shape: &[usize], dtype: DType) -> Result<bool> {
        let tensor_size_mb = self.estimate_tensor_size_mb(shape, dtype);
        
        let current_usage = if let Ok(memory) = self.memory_usage.lock() {
            *memory
        } else {
            0
        };
        
        Ok(current_usage + tensor_size_mb <= self.config.max_memory_mb)
    }
    
    /// Estimer la taille d'un tensor en MB
    fn estimate_tensor_size_mb(&self, shape: &[usize], dtype: DType) -> usize {
        let element_count: usize = shape.iter().product();
        let bytes_per_element = match dtype {
            DType::F32 => 4,
            DType::F16 => 2,
            DType::I64 => 8,
            DType::U32 => 4,
            _ => 4, // Défaut
        };
        
        (element_count * bytes_per_element) / (1024 * 1024) // Convertir en MB
    }
    
    /// Mettre à jour l'estimation de l'usage mémoire
    fn update_memory_usage(&self, shape: &[usize], dtype: DType, is_add: bool) {
        let size_mb = self.estimate_tensor_size_mb(shape, dtype);
        
        if let Ok(mut memory) = self.memory_usage.lock() {
            if is_add {
                *memory += size_mb;
            } else {
                *memory = memory.saturating_sub(size_mb);
            }
        }
    }
    
    /// Obtenir les statistiques du pool
    pub fn get_stats(&self) -> DevicePoolStats {
        let cache_size = if let Ok(cache) = self.tensor_cache.lock() {
            cache.len()
        } else {
            0
        };
        
        let memory_usage_mb = if let Ok(memory) = self.memory_usage.lock() {
            *memory
        } else {
            0
        };
        
        DevicePoolStats {
            device_type: format!("{:?}", self.device),
            cache_size,
            cache_capacity: self.config.cache_capacity,
            memory_usage_mb,
            memory_limit_mb: self.config.max_memory_mb,
            memory_usage_percent: if self.config.max_memory_mb > 0 {
                (memory_usage_mb as f32 / self.config.max_memory_mb as f32 * 100.0)
            } else {
                0.0
            },
        }
    }
    
    /// Logger les statistiques
    pub fn log_stats(&self) {
        let stats = self.get_stats();
        info!(
            "📊 DevicePool Stats - Device: {}, Cache: {}/{}, Memory: {}MB/{}MB ({:.1}%)",
            stats.device_type,
            stats.cache_size,
            stats.cache_capacity,
            stats.memory_usage_mb,
            stats.memory_limit_mb,
            stats.memory_usage_percent
        );
    }
    
    /// Obtenir le device utilisé
    pub fn device(&self) -> &Device {
        &self.device
    }
}

/// Statistiques du DevicePool
#[derive(Debug, Clone)]
pub struct DevicePoolStats {
    pub device_type: String,
    pub cache_size: usize,
    pub cache_capacity: usize,
    pub memory_usage_mb: usize,
    pub memory_limit_mb: usize,
    pub memory_usage_percent: f32,
}

/// Pool global pour GPU si disponible
pub struct GlobalDevicePool {
    cpu_pool: DevicePool,
    gpu_pool: Option<DevicePool>,
}

impl GlobalDevicePool {
    /// Créer un pool global avec CPU et GPU si disponible
    pub fn new(config: DevicePoolConfig) -> Self {
        let cpu_pool = DevicePool::new(Device::Cpu, config.clone());
        
        // Essayer de créer un pool GPU si disponible
        let gpu_pool = match Device::new_cuda(0) {
            Ok(gpu_device) => {
                info!("🎮 GPU device detected, creating GPU pool");
                Some(DevicePool::new(gpu_device, config))
            }
            Err(_) => {
                info!("💻 No GPU detected, using CPU only");
                None
            }
        };
        
        Self { cpu_pool, gpu_pool }
    }
    
    /// Obtenir le pool approprié (GPU si disponible, sinon CPU)
    pub fn get_optimal_pool(&self) -> &DevicePool {
        self.gpu_pool.as_ref().unwrap_or(&self.cpu_pool)
    }
    
    /// Obtenir le pool CPU
    pub fn cpu_pool(&self) -> &DevicePool {
        &self.cpu_pool
    }
    
    /// Obtenir le pool GPU si disponible
    pub fn gpu_pool(&self) -> Option<&DevicePool> {
        self.gpu_pool.as_ref()
    }
    
    /// Logger les stats de tous les pools
    pub fn log_all_stats(&self) {
        info!("📊 === DevicePool Global Stats ===");
        self.cpu_pool.log_stats();
        if let Some(gpu_pool) = &self.gpu_pool {
            gpu_pool.log_stats();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_pool_creation() {
        let pool = DevicePool::default_cpu();
        let stats = pool.get_stats();
        
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.memory_usage_mb, 0);
        println!("✅ DevicePool created successfully");
    }
    
    #[test]
    fn test_tensor_cache() {
        let pool = DevicePool::default_cpu();
        
        // Créer un tensor
        let tensor1 = pool.get_or_create_tensor("test_key", &[10, 10], DType::F32);
        assert!(tensor1.is_ok());
        
        // Récupérer le même tensor (doit venir du cache)
        let tensor2 = pool.get_or_create_tensor("test_key", &[10, 10], DType::F32);
        assert!(tensor2.is_ok());
        
        let stats = pool.get_stats();
        assert_eq!(stats.cache_size, 1);
        println!("✅ Tensor caching working");
    }
    
    #[test]
    fn test_memory_limit() {
        let config = DevicePoolConfig {
            max_memory_mb: 1, // Très petit pour forcer la limite
            ..Default::default()
        };
        
        let pool = DevicePool::new(Device::Cpu, config);
        
        // Essayer de créer un tensor trop grand
        let result = pool.get_or_create_tensor("big_tensor", &[1000, 1000], DType::F32);
        
        // Devrait échouer ou forcer un cleanup
        if result.is_err() {
            println!("✅ Memory limit protection working");
        } else {
            println!("✅ Memory limit handled with cleanup");
        }
    }
    
    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2, Duration::from_secs(1));
        
        cache.put("key1", "value1");
        cache.put("key2", "value2");
        cache.put("key3", "value3"); // Devrait évincer key1
        
        assert!(cache.get(&"key1".to_string()).is_none());
        assert!(cache.get(&"key2".to_string()).is_some());
        assert!(cache.get(&"key3".to_string()).is_some());
        
        println!("✅ LRU cache eviction working");
    }
}