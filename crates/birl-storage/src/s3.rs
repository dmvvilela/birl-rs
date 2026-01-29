use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use bytes::Bytes;
use birl_core::View;
use tracing::{debug, warn};

/// S3 client wrapper for fetching and saving images
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    /// Create a new S3 storage client
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }

    /// Fetch a layer image from S3
    /// Path format: birl/{view}/{category}/{sku}.{extension}
    pub async fn fetch_layer(
        &self,
        category: &str,
        sku: &str,
        view: View,
        extension: &str,
    ) -> Result<Option<Bytes>> {
        let key = format!("birl/{}/{}/{}.{}", view.as_str(), category, sku, extension);

        match self.fetch_object(&key).await {
            Ok(data) => {
                debug!("Fetched layer: {} ({} bytes)", key, data.len());
                Ok(Some(data))
            }
            Err(e) => {
                warn!("Failed to fetch layer {}: {}", key, e);
                Ok(None)
            }
        }
    }

    /// Fetch a cached composite image from S3
    /// Path format: birl/cache/{cache_key}.jpg
    pub async fn fetch_cached(&self, cache_key: &str) -> Result<Option<Bytes>> {
        let key = format!("birl/cache/{}.jpg", cache_key);

        match self.fetch_object(&key).await {
            Ok(data) => {
                debug!("Cache hit: {} ({} bytes)", cache_key, data.len());
                Ok(Some(data))
            }
            Err(_) => {
                debug!("Cache miss: {}", cache_key);
                Ok(None)
            }
        }
    }

    /// Save a composite image to S3 cache
    pub async fn save_to_cache(&self, cache_key: &str, data: &[u8]) -> Result<()> {
        let key = format!("birl/cache/{}.jpg", cache_key);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(data.to_vec().into())
            .content_type("image/jpeg")
            .send()
            .await
            .context("Failed to save to cache")?;

        debug!("Saved to cache: {} ({} bytes)", cache_key, data.len());

        Ok(())
    }

    /// Fetch a cached JSON file from S3
    /// Path format: birl/cache/{key}.json
    pub async fn fetch_cached_json(&self, key: &str) -> Result<Option<String>> {
        let s3_key = format!("birl/cache/{}.json", key);

        match self.fetch_object(&s3_key).await {
            Ok(data) => {
                let json = String::from_utf8(data.to_vec())
                    .context("Failed to convert JSON to string")?;
                Ok(Some(json))
            }
            Err(_) => Ok(None),
        }
    }

    /// Generic fetch object from S3
    async fn fetch_object(&self, key: &str) -> Result<Bytes> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("Failed to fetch object: {}", key))?;

        let data = response
            .body
            .collect()
            .await
            .context("Failed to read object body")?
            .into_bytes();

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are integration tests that require actual S3 credentials
    // They're marked with #[ignore] by default

    #[tokio::test]
    #[ignore]
    async fn test_s3_fetch_layer() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        let bucket = std::env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME not set");
        let storage = S3Storage::new(client, bucket);

        // This is a test that would need actual S3 setup
        let result = storage
            .fetch_layer("plate", "swatthermals-black", View::Front, "jpg")
            .await;

        assert!(result.is_ok());
    }
}
