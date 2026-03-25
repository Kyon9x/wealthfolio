//! Server configuration from environment variables.

use std::time::Duration;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Address to listen on (e.g., "127.0.0.1:8080")
    pub listen_addr: String,

    /// Path to the SQLite database
    pub db_path: String,

    /// Directory for static files (frontend build)
    pub static_dir: String,

    /// Request timeout duration
    pub request_timeout: Duration,

    /// CORS allowed origins
    pub cors_allow_origins: Vec<String>,

    /// Secrets encryption key (optional)
    pub secrets_encryption_key: Option<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Environment variables:
    /// - `WF_LISTEN_ADDR`: Address to listen on (default: "127.0.0.1:8080")
    /// - `WF_DB_PATH`: Path to SQLite database (default: "./db/wealthvn.db")
    /// - `WF_STATIC_DIR`: Static files directory (default: "./dist")
    /// - `WF_REQUEST_TIMEOUT_MS`: Request timeout in milliseconds (default: 30000)
    /// - `WF_CORS_ALLOW_ORIGINS`: Comma-separated list of allowed origins (default: "*")
    /// - `WF_SECRETS_KEY`: Encryption key for secrets (optional)
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let listen_addr = std::env::var("WF_LISTEN_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let db_path = std::env::var("WF_DB_PATH")
            .unwrap_or_else(|_| "./db/wealthvn.db".to_string());

        let static_dir = std::env::var("WF_STATIC_DIR")
            .unwrap_or_else(|_| "./dist".to_string());

        let request_timeout_ms = std::env::var("WF_REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30000);

        let cors_allow_origins = std::env::var("WF_CORS_ALLOW_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let secrets_encryption_key = std::env::var("WF_SECRETS_KEY").ok();

        Config {
            listen_addr,
            db_path,
            static_dir,
            request_timeout: Duration::from_millis(request_timeout_ms),
            cors_allow_origins,
            secrets_encryption_key,
        }
    }
}
