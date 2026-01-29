use anyhow::{Context, Result};
use birl_core::{compose_layers, generate_cache_key, parse_params, LayerNormalizer, View};
use birl_storage::StorageService;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ComposeOptions {
    pub view: View,
    pub params: String,
    pub output: Option<String>,
    pub bypass_cache: bool,
}

pub async fn compose_command(storage: Arc<StorageService>, options: ComposeOptions) -> Result<()> {
    let start = std::time::Instant::now();

    info!(
        "Composing image: view={}, params={}",
        options.view.as_str(),
        options.params
    );

    // Fetch base plate
    let base_image_data = storage
        .fetch_base_plate(options.view)
        .await
        .context("Failed to fetch base plate")?;

    // Parse and normalize parameters
    let params = parse_params(&options.params);
    let normalizer = LayerNormalizer::new(options.view, &params);
    let normalized_params = normalizer.normalize_all(&params);

    info!("Normalized to {} layers", normalized_params.len());

    // Generate cache key
    let cache_key = generate_cache_key(
        &normalized_params,
        options.view,
        options.view.plate_value(),
    );

    // Check cache (unless bypassing)
    if !options.bypass_cache {
        if let Some(cached_data) = storage.get_cached_composite(&cache_key).await? {
            info!("Found cached composite: {}", cache_key);

            if let Some(output_path) = &options.output {
                std::fs::write(output_path, cached_data)
                    .context("Failed to write output file")?;
                info!("Wrote cached image to {}", output_path);
            } else {
                println!("Cache hit: {}.jpg", cache_key);
            }

            info!("Completed in {:?} (cached)", start.elapsed());
            return Ok(());
        }
    }

    // Fetch layers in parallel
    let layers_result = storage
        .fetch_layers(&normalized_params, options.view)
        .await?;

    // Filter out None values
    let layers: Vec<_> = layers_result.into_iter().flatten().collect();

    let requested_count = normalized_params.len();
    let found_count = layers.len();

    if found_count < requested_count {
        warn!(
            "Found {}/{} requested layers",
            found_count, requested_count
        );
    } else {
        info!("Fetched all {} layers", found_count);
    }

    // Compose the image
    info!("Compositing layers...");
    let composite_data = compose_layers(&base_image_data, layers)
        .context("Failed to compose layers")?;

    // Save to cache if all layers were found
    if requested_count == found_count {
        storage
            .save_composite(&cache_key, composite_data.clone())
            .await
            .context("Failed to save to cache")?;
        info!("Saved to cache: {}", cache_key);
    }

    // Write output file
    if let Some(output_path) = &options.output {
        std::fs::write(output_path, &composite_data)
            .context("Failed to write output file")?;
        info!("Wrote image to {}", output_path);
    } else {
        println!("Composite created: {}.jpg ({} bytes)", cache_key, composite_data.len());
    }

    info!("Completed in {:?}", start.elapsed());

    Ok(())
}
