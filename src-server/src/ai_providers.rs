//! AI Provider settings and model listing endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};

use crate::AppState;
use wealthvn_ai::{
    AiProvidersResponse, ListModelsResponse, ProviderApiError, SetDefaultProviderRequest,
    UpdateProviderSettingsRequest,
};

/// GET /api/v1/ai/providers
///
/// List all AI providers with their settings and available models.
pub async fn get_ai_providers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AiProvidersResponse>, ApiError> {
    let response = state
        .ai_provider_service
        .get_ai_providers()
        .map_err(ApiError::Ai)?;
    Ok(Json(response))
}

/// PUT /api/v1/ai/providers/settings
///
/// Update AI provider settings.
pub async fn update_provider_settings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateProviderSettingsRequest>,
) -> Result<Json<()>, ApiError> {
    state
        .ai_provider_service
        .update_provider_settings(request)
        .await
        .map_err(ApiError::Ai)?;
    Ok(Json(()))
}

/// POST /api/v1/ai/providers/default
///
/// Set the default AI provider.
pub async fn set_default_provider(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SetDefaultProviderRequest>,
) -> Result<Json<()>, ApiError> {
    state
        .ai_provider_service
        .set_default_provider(request)
        .await
        .map_err(ApiError::Ai)?;
    Ok(Json(()))
}

/// GET /api/v1/ai/providers/:provider_id/models
///
/// List available models from a provider.
/// Fetches models from the provider's API using backend-stored secrets.
/// Frontend never needs to send API keys - they are retrieved internally.
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Result<Json<ListModelsResponse>, ApiError> {
    let response = state
        .ai_provider_service
        .list_models(&provider_id)
        .await
        .map_err(ApiError::ProviderApi)?;
    Ok(Json(response))
}

/// Error type for AI provider endpoints.
#[derive(Debug)]
pub enum ApiError {
    Ai(wealthvn_ai::AiError),
    ProviderApi(wealthvn_ai::ProviderApiError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            ApiError::Ai(e) => {
                let status = match &e {
                    wealthvn_ai::AiError::InvalidInput(_) => axum::http::StatusCode::BAD_REQUEST,
                    wealthvn_ai::AiError::MissingApiKey(_) => axum::http::StatusCode::BAD_REQUEST,
                    wealthvn_ai::AiError::Provider(_) => axum::http::StatusCode::BAD_GATEWAY,
                    wealthvn_ai::AiError::ToolNotFound(_) => axum::http::StatusCode::BAD_REQUEST,
                    wealthvn_ai::AiError::ToolNotAllowed(_) => axum::http::StatusCode::FORBIDDEN,
                    wealthvn_ai::AiError::ToolExecutionFailed(_) => {
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    }
                    wealthvn_ai::AiError::ThreadNotFound(_) => axum::http::StatusCode::NOT_FOUND,
                    wealthvn_ai::AiError::InvalidCursor(_) => axum::http::StatusCode::BAD_REQUEST,
                    wealthvn_ai::AiError::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    wealthvn_ai::AiError::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, e.code().to_string(), e.to_string())
            }
            ApiError::ProviderApi(e) => {
                let status = match e {
                    wealthvn_ai::ProviderApiError::InvalidCredentials => {
                        axum::http::StatusCode::UNAUTHORIZED
                    }
                    wealthvn_ai::ProviderApiError::NetworkError(_) => {
                        axum::http::StatusCode::BAD_GATEWAY
                    }
                    wealthvn_ai::ProviderApiError::RateLimited => {
                        axum::http::StatusCode::TOO_MANY_REQUESTS
                    }
                    wealthvn_ai::ProviderApiError::InvalidResponse(_) => {
                        axum::http::StatusCode::BAD_GATEWAY
                    }
                    wealthvn_ai::ProviderApiError::ProviderNotFound => {
                        axum::http::StatusCode::NOT_FOUND
                    }
                    wealthvn_ai::ProviderApiError::AuthenticationFailed => {
                        axum::http::StatusCode::UNAUTHORIZED
                    }
                    wealthvn_ai::ProviderApiError::QuotaExceeded => {
                        axum::http::StatusCode::PAYLOAD_TOO_LARGE
                    }
                    wealthvn_ai::ProviderApiError::Internal(_) => {
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    }
                };
                (status, "provider_error".to_string(), e.to_string())
            }
        };

        let body = serde_json::json!({
            "code": code,
            "error": message
        });

        (status, Json(body)).into_response()
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ai/providers", get(get_ai_providers))
        .route("/ai/providers/settings", put(update_provider_settings))
        .route("/ai/providers/default", post(set_default_provider))
        .route("/ai/providers/{provider_id}/models", get(list_models))
}

use axum::response::IntoResponse;
