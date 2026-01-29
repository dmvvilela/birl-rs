use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

/// Validate webhook headers
/// This is a placeholder implementation - customize based on your auth needs
pub async fn validate_webhook(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // For now, we'll allow all requests
    // TODO: Implement proper webhook validation based on Hookdeck or your auth provider
    //
    // Example implementation:
    // - Check for X-Webhook-Signature header
    // - Verify HMAC signature
    // - Check for API key in Authorization header

    let has_auth = request
        .headers()
        .get("authorization")
        .or_else(|| request.headers().get("x-api-key"))
        .is_some();

    if !has_auth {
        // In development, we might want to allow requests without auth
        // In production, uncomment the following:
        // warn!("Unauthorized request");
        // return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

/// Middleware to validate Hookdeck webhook signature
/// Reference: https://hookdeck.com/docs/verify-webhooks
#[allow(dead_code)]
pub async fn validate_hookdeck_signature(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get the signature from headers
    let signature = request.headers().get("x-hookdeck-signature");

    if signature.is_none() {
        warn!("Missing Hookdeck signature");
        // In development mode, allow without signature
        // return Err(StatusCode::UNAUTHORIZED);
    }

    // TODO: Implement HMAC verification
    // 1. Get webhook secret from environment
    // 2. Compute HMAC of request body
    // 3. Compare with signature header

    Ok(next.run(request).await)
}
