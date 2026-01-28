mod middleware;
mod routes;

use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use sandwich_storage::StorageService;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load AWS configuration
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);

    // Get bucket name from environment
    let bucket_name = std::env::var("AWS_BUCKET_NAME")
        .unwrap_or_else(|_| "sandwich-bucket".to_string());

    info!("Using S3 bucket: {}", bucket_name);

    // Create storage service
    let storage = Arc::new(StorageService::new(s3_client, bucket_name, 1000));

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // API routes with authentication middleware
        .route("/create", post(routes::create_composite))
        .route("/products", get(routes::get_products))
        .layer(from_fn(middleware::validate_webhook))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // Shared state
        .with_state(storage);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
