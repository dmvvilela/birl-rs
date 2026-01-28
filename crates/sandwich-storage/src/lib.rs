//! sandwich-storage: S3 storage and caching layer for the Sandwich app
//!
//! This crate provides storage operations for fetching layers from S3,
//! caching composites, and managing a multi-tier cache (memory + S3).

pub mod cache;
pub mod s3;

use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use bytes::Bytes;
use futures::future::try_join_all;
use sandwich_core::{LayerParam, View};
use std::sync::Arc;
use tracing::{debug, warn};

pub use cache::{CacheStats, ImageCache};
pub use s3::S3Storage;

/// High-level storage service that combines S3 and caching
pub struct StorageService {
    s3: Arc<S3Storage>,
    cache: Arc<ImageCache>,
}

impl StorageService {
    /// Create a new storage service
    pub fn new(s3_client: Client, bucket: String, cache_capacity: usize) -> Self {
        let s3 = Arc::new(S3Storage::new(s3_client, bucket));
        let cache = Arc::new(ImageCache::new(s3.clone(), cache_capacity));

        Self { s3, cache }
    }

    /// Fetch the base plate image
    pub async fn fetch_base_plate(&self, view: View) -> Result<Bytes> {
        let plate_value = view.plate_value();

        self.s3
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
            let s3 = self.s3.clone();
            let category = param.category.clone();
            let sku = param.sku.as_str().to_string();

            async move { s3.fetch_layer(&category, &sku, view, "png").await }
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
        self.s3.fetch_cached_json(key).await
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
