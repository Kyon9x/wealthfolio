//! API router for WealthVN web server.
//!
//! Composes all route modules into a single API router with middleware.

use std::sync::Arc;

use crate::AppState;
use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

mod ai;
mod ai_providers;

/// Health check endpoint.
pub async fn healthz() -> &'static str {
    "ok"
}

/// Readiness check endpoint.
pub async fn readyz() -> &'static str {
    "ok"
}

/// Build the main API router with all routes and middleware.
pub fn app_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new().allow_origin(Any);

    // Build protected API routes
    let protected_api = Router::new()
        .merge(ai::router())
        .merge(ai_providers::router());

    // Public API routes (health, readiness)
    let api = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route("/readyz", axum::routing::get(readyz))
        .merge(protected_api)
        .with_state(state.clone());

    // Nest under /api/v1
    Router::new()
        .nest("/api/v1", api)
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        path = %request.uri().path(),
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
