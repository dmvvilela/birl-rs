use crate::StorageBackend;
use anyhow::Result;
use bytes::Bytes;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Multi-tier image cache (LRU in-memory + persistent storage)
pub struct ImageCache {
    /// In-memory LRU cache
    memory: Arc<Mutex<LruCache<String, Arc<Bytes>>>>,
    /// Storage backend (S3 or local filesystem)
    backend: Arc<dyn StorageBackend>,
}

impl ImageCache {
    /// Create a new image cache
    pub fn new(backend: Arc<dyn StorageBackend>, capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());

        Self {
            memory: Arc::new(Mutex::new(LruCache::new(capacity))),
            backend,
        }
    }

    /// Get a cached composite image
    /// First checks memory cache, then backend cache
    pub async fn get(&self, cache_key: &str) -> Result<Option<Bytes>> {
        // Check memory cache first
        {
            let mut cache = self.memory.lock().await;
            if let Some(data) = cache.get(cache_key) {
                debug!("Memory cache hit: {}", cache_key);
                return Ok(Some((**data).clone()));
            }
        }

        // Check backend cache
        if let Some(data) = self.backend.fetch_cached(cache_key).await? {
            debug!("Backend cache hit: {}", cache_key);

            // Store in memory cache for future requests
            let arc_data = Arc::new(data.clone());
            let mut cache = self.memory.lock().await;
            cache.put(cache_key.to_string(), arc_data);

            return Ok(Some(data));
        }

        debug!("Cache miss: {}", cache_key);
        Ok(None)
    }

    /// Save a composite image to cache
    /// Saves to both memory and backend
    pub async fn put(&self, cache_key: &str, data: Bytes) -> Result<()> {
        // Save to backend
        self.backend.save_to_cache(cache_key, &data).await?;

        // Save to memory cache
        let arc_data = Arc::new(data);
        let mut cache = self.memory.lock().await;
        cache.put(cache_key.to_string(), arc_data);

        info!("Cached composite: {}", cache_key);

        Ok(())
    }

    /// Clear memory cache
    pub async fn clear_memory(&self) {
        let mut cache = self.memory.lock().await;
        cache.clear();
        info!("Memory cache cleared");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.memory.lock().await;
        CacheStats {
            memory_entries: cache.len(),
            memory_capacity: cache.cap().get(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub memory_capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::Client;

    #[tokio::test]
    async fn test_cache_creation() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        let s3 = Arc::new(S3Storage::new(client, "test-bucket".to_string()));
        let cache = ImageCache::new(s3, 100);

        let stats = cache.stats().await;
        assert_eq!(stats.memory_capacity, 100);
        assert_eq!(stats.memory_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        let s3 = Arc::new(S3Storage::new(client, "test-bucket".to_string()));
        let cache = ImageCache::new(s3, 100);

        // Put data in memory cache
        let data = Bytes::from("test data");
        {
            let mut mem_cache = cache.memory.lock().await;
            mem_cache.put("test-key".to_string(), Arc::new(data.clone()));
        }

        // Get from memory cache
        let result = {
            let mut mem_cache = cache.memory.lock().await;
            mem_cache.get("test-key").map(|d| (**d).clone())
        };

        assert_eq!(result, Some(data));
    }
}
