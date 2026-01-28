//! sandwich-storage: S3 storage and caching layer for the Sandwich app
//!
//! This crate provides storage operations for fetching layers from S3,
//! caching composites, and managing a multi-tier cache (memory + S3).

pub mod cache;
pub mod local;
pub mod s3;

use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use bytes::Bytes;
use futures::future::try_join_all;
use sandwich_core::{LayerParam, View};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, warn};

pub use cache::{CacheStats, ImageCache};
pub use local::LocalStorage;
pub use s3::S3Storage;

/// Storage backend trait
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn fetch_layer(
        &self,
        category: &str,
        sku: &str,
        view: View,
        extension: &str,
    ) -> Result<Option<Bytes>>;

    async fn fetch_cached(&self, cache_key: &str) -> Result<Option<Bytes>>;
    async fn save_to_cache(&self, cache_key: &str, data: &[u8]) -> Result<()>;
    async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>>;
}

#[async_trait::async_trait]
impl StorageBackend for S3Storage {
    async fn fetch_layer(
        &self,
        category: &str,
        sku: &str,
        view: View,
        extension: &str,
    ) -> Result<Option<Bytes>> {
        S3Storage::fetch_layer(self, category, sku, view, extension).await
    }

    async fn fetch_cached(&self, cache_key: &str) -> Result<Option<Bytes>> {
        S3Storage::fetch_cached(self, cache_key).await
    }

    async fn save_to_cache(&self, cache_key: &str, data: &[u8]) -> Result<()> {
        S3Storage::save_to_cache(self, cache_key, data).await
    }

    async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>> {
        S3Storage::fetch_cached_json(self, key).await
    }
}

#[async_trait::async_trait]
impl StorageBackend for LocalStorage {
    async fn fetch_layer(
        &self,
        category: &str,
        sku: &str,
        view: View,
        extension: &str,
    ) -> Result<Option<Bytes>> {
        LocalStorage::fetch_layer(self, category, sku, view, extension).await
    }

    async fn fetch_cached(&self, cache_key: &str) -> Result<Option<Bytes>> {
        LocalStorage::fetch_cached(self, cache_key).await
    }

    async fn save_to_cache(&self, cache_key: &str, data: &[u8]) -> Result<()> {
        LocalStorage::save_to_cache(self, cache_key, data).await
    }

    async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>> {
        LocalStorage::fetch_cached_json(self, key).await
    }
}

/// High-level storage service that combines storage backend and caching
pub struct StorageService {
    backend: Arc<dyn StorageBackend>,
    cache: Arc<ImageCache>,
}

impl StorageService {
    /// Create a new storage service with S3 backend
    pub fn new_s3(s3_client: Client, bucket: String, cache_capacity: usize) -> Self {
        let backend = Arc::new(S3Storage::new(s3_client, bucket));
        let cache = Arc::new(ImageCache::new(backend.clone(), cache_capacity));

        Self { backend, cache }
    }

    /// Create a new storage service with local filesystem backend
    pub fn new_local(base_path: PathBuf, cache_capacity: usize) -> Self {
        let backend = Arc::new(LocalStorage::new(base_path));
        let cache = Arc::new(ImageCache::new(backend.clone(), cache_capacity));

        Self { backend, cache }
    }

    /// Legacy constructor for backward compatibility
    #[deprecated(note = "Use new_s3() instead")]
    pub fn new(s3_client: Client, bucket: String, cache_capacity: usize) -> Self {
        Self::new_s3(s3_client, bucket, cache_capacity)
    }

    /// Fetch the base plate image
    pub async fn fetch_base_plate(&self, view: View) -> Result<Bytes> {
        let plate_value = view.plate_value();

        self.backend
            .fetch_layer("plate", plate_value, view, "jpg")
            .await?
            .context("Base plate not found")
    }

    /// Fetch multiple layers in parallel
    pub async fn fetch_layers(
        &self,
        params: &[LayerParam],
        view: View,
    ) -> Result<Vec<Option<Bytes>>> {
        let futures = params.iter().map(|param| {
            let backend = self.backend.clone();
            let category = param.category.clone();
            let sku = param.sku.as_str().to_string();

            async move { backend.fetch_layer(&category, &sku, view, "png").await }
        });

        try_join_all(futures).await
    }

    /// Get a cached composite
    pub async fn get_cached_composite(&self, cache_key: &str) -> Result<Option<Bytes>> {
        self.cache.get(cache_key).await
    }

    /// Save a composite to cache
    pub async fn save_composite(&self, cache_key: &str, data: Bytes) -> Result<()> {
        self.cache.put(cache_key, data).await
    }

    /// Fetch cached JSON data (e.g., product list)
    pub async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>> {
        self.backend.fetch_cached_json(key).await
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    /// Clear memory cache
    pub async fn clear_cache(&self) {
        self.cache.clear_memory().await;
    }
}

/// Fetch layers with logging and filtering
pub async fn fetch_and_filter_layers(
    storage: &StorageService,
    params: &[LayerParam],
    view: View,
) -> Result<(Vec<Bytes>, usize, usize)> {
    let layers = storage.fetch_layers(params, view).await?;

    let requested_count = params.len();
    let found_layers: Vec<Bytes> = layers.into_iter().flatten().collect();
    let found_count = found_layers.len();

    if found_count < requested_count {
        warn!(
            "Found {}/{} requested layers for view {}",
            found_count,
            requested_count,
            view.as_str()
        );

        // Log which layers were missing
        for (i, param) in params.iter().enumerate() {
            if i >= found_layers.len() {
                debug!("Missing layer: {}/{}", param.category, param.sku.as_str());
            }
        }
    }

    Ok((found_layers, requested_count, found_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_service_creation() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        let service = StorageService::new(client, "test-bucket".to_string(), 100);

        let stats = service.cache_stats().await;
        assert_eq!(stats.memory_capacity, 100);
    }
}
