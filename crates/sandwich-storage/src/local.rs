use anyhow::{Context, Result};
use bytes::Bytes;
use sandwich_core::View;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Local filesystem storage for development and testing
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    /// Create a new local storage with a base directory
    /// Expected structure: {base_path}/{view}/{category}/{sku}.{ext}
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Fetch a layer image from local filesystem
    /// Path format: {base_path}/{view}/{category}/{sku}.{extension}
    /// Also searches in subdirectories if not found directly
    pub async fn fetch_layer(
        &self,
        category: &str,
        sku: &str,
        view: View,
        extension: &str,
    ) -> Result<Option<Bytes>> {
        let filename = format!("{}.{}", sku, extension);

        // Try direct path first
        let direct_path = self.base_path.join(format!(
            "{}/{}/{}",
            view.as_str(),
            category,
            filename
        ));

        if let Ok(data) = tokio::fs::read(&direct_path).await {
            debug!("Fetched layer: {} ({} bytes)", direct_path.display(), data.len());
            return Ok(Some(Bytes::from(data)));
        }

        // If not found, search in subdirectories
        let category_path = self.base_path.join(format!("{}/{}", view.as_str(), category));

        if let Ok(mut entries) = tokio::fs::read_dir(&category_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().is_dir() {
                    let subdir_path = entry.path().join(&filename);
                    if let Ok(data) = tokio::fs::read(&subdir_path).await {
                        debug!("Fetched layer from subdir: {} ({} bytes)", subdir_path.display(), data.len());
                        return Ok(Some(Bytes::from(data)));
                    }
                }
            }
        }

        debug!("Layer not found: {}/{}/{}", view.as_str(), category, filename);
        Ok(None)
    }

    /// Fetch a cached composite image
    /// Path format: {base_path}/cache/{cache_key}.jpg
    pub async fn fetch_cached(&self, cache_key: &str) -> Result<Option<Bytes>> {
        let path = self
            .base_path
            .join(format!("cache/{}.jpg", cache_key));

        match tokio::fs::read(&path).await {
            Ok(data) => {
                debug!("Cache hit: {} ({} bytes)", cache_key, data.len());
                Ok(Some(Bytes::from(data)))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("Cache miss: {}", cache_key);
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to read cache {}: {}", path.display(), e);
                Ok(None)
            }
        }
    }

    /// Save a composite image to cache
    pub async fn save_to_cache(&self, cache_key: &str, data: &[u8]) -> Result<()> {
        let path = self
            .base_path
            .join(format!("cache/{}.jpg", cache_key));

        // Create cache directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create cache directory")?;
        }

        tokio::fs::write(&path, data)
            .await
            .context("Failed to write cache file")?;

        debug!("Saved to cache: {} ({} bytes)", cache_key, data.len());

        Ok(())
    }

    /// Fetch cached JSON data
    pub async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>> {
        let path = self
            .base_path
            .join(format!("cache/{}.json", key));

        match tokio::fs::read_to_string(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_local_storage_creation() {
        let storage = LocalStorage::new("/tmp/sandwich-test");
        assert_eq!(storage.base_path(), Path::new("/tmp/sandwich-test"));
    }

    #[tokio::test]
    async fn test_fetch_layer_not_found() {
        let storage = LocalStorage::new("/tmp/nonexistent");
        let result = storage
            .fetch_layer("hoodies", "test", View::Front, "png")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
