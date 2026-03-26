//! WealthVN web server binary.
//!
//! HTTP server providing REST API for the WealthVN portfolio tracker.

use wealthvn_server_lib::{app_router, build_state, init_tracing, Config};
use axum::http::StatusCode;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    init_tracing();

    let state = build_state(&config).await?;

    // Set up static file serving
    let static_dir = std::path::PathBuf::from(&config.static_dir);
    let index_file = static_dir.join("index.html");

    let static_service = if static_dir.exists() {
        ServeDir::new(static_dir).fallback(ServeFile::new(index_file))
    } else {
        // If static dir doesn't exist, return a simple response
        let not_found_service = axum::routing::any(|| async {
            (StatusCode::NOT_FOUND, "Static files not found. Build the frontend first.")
        });
        axum::service::not_found_service(not_found_service)
    };

    let router = app_router(state)
        .fallback_service(static_service);

    tracing::info!("Listening on {}", config.listen_addr);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
