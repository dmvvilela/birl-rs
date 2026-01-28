use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sandwich_storage::StorageService;
use serde::Serialize;
use std::sync::Arc;
use tracing::error;

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// GET /products - Fetch cached products from S3
pub async fn get_products(State(storage): State<Arc<StorageService>>) -> Response {
    match get_products_impl(storage).await {
        Ok(json) => (StatusCode::OK, json).into_response(),
        Err(e) => {
            error!("Error fetching products: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch products data".to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn get_products_impl(storage: Arc<StorageService>) -> anyhow::Result<String> {
    const CACHE_KEY: &str = "products-dynamic-cache";

    let json_data = storage
        .fetch_cached_json(CACHE_KEY)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Products cache not found"))?;

    Ok(json_data)
}
