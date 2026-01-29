use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use birl_core::{compose_layers, generate_cache_key, parse_params, LayerNormalizer, View};
use birl_storage::StorageService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Request body for POST /create
#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    /// Comma-separated parameters: "category/sku,category/sku,..."
    pub p: String,
    /// View to render (default: front)
    #[serde(default = "default_view")]
    pub view: View,
    /// Bypass cache and force regeneration
    #[serde(default)]
    pub bypass_cache: bool,
}

fn default_view() -> View {
    View::Front
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /create - Create a composite image
pub async fn create_composite(
    State(storage): State<Arc<StorageService>>,
    Json(request): Json<CreateRequest>,
) -> Response {
    if let Err(e) = create_composite_impl(storage, request).await {
        error!("Error creating composite: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response();
    }

    StatusCode::OK.into_response()
}

async fn create_composite_impl(
    storage: Arc<StorageService>,
    request: CreateRequest,
) -> anyhow::Result<Response> {
    let CreateRequest {
        p,
        view,
        bypass_cache,
    } = request;

    // Fetch base plate image
    let base_image_data = storage.fetch_base_plate(view).await?;

    // If no parameters provided, return just the base plate
    if p.trim().is_empty() {
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/jpeg")],
            base_image_data,
        )
            .into_response());
    }

    // Parse and normalize parameters
    let params = parse_params(&p);
    let normalizer = LayerNormalizer::new(view, &params);
    let normalized_params = normalizer.normalize_all(&params);

    // Generate cache key
    let cache_key = generate_cache_key(&normalized_params, view, view.plate_value());

    // Check cache (unless bypassing)
    if !bypass_cache {
        if let Some(cached_data) = storage.get_cached_composite(&cache_key).await? {
            info!("Serving cached image: {}", cache_key);
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                cached_data,
            )
                .into_response());
        }
    }

    // Fetch layers in parallel
    let layers_result = storage.fetch_layers(&normalized_params, view).await?;

    // Filter out None values and collect into Vec<Bytes>
    let layers: Vec<_> = layers_result.into_iter().flatten().collect();

    // Log if some layers are missing
    let requested_count = normalized_params.len();
    let found_count = layers.len();

    if found_count < requested_count {
        warn!(
            "Found {}/{} requested layers for view {}",
            found_count,
            requested_count,
            view.as_str()
        );
    }

    // Compose the image
    let composite_data = compose_layers(&base_image_data, layers)?;

    // Only cache if all requested images were found
    if requested_count == found_count {
        if let Err(e) = storage.save_composite(&cache_key, composite_data.clone()).await {
            error!("Failed to save to cache: {}", e);
            // Don't fail the request if caching fails
        }
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/jpeg")],
        composite_data,
    )
        .into_response())
}
