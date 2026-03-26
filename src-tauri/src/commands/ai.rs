//! AI Chat Tauri commands for streaming responses and thread management.
//!
//! Uses Tauri's IPC Channel for efficient streaming of AI events.

use std::sync::Arc;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};
use wealthvn_ai::{
    AiError, AiStreamEvent, ChatRepositoryTrait, ChatThread, ListThreadsRequest, SendMessageRequest,
    ThreadPage,
};

use crate::context::ServiceContext;

use super::error::CommandResult;

/// Request for updating thread title or pinned status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateThreadRequest {
    pub id: String,
    pub title: Option<String>,
    pub is_pinned: Option<bool>,
}

/// Stream a chat message and receive AI events through a Tauri Channel.
///
/// The channel will receive `AiStreamEvent` objects:
/// - `system`: Initial event with thread_id, run_id, message_id
/// - `textDelta`: Partial text content
/// - `reasoningDelta`: Optional reasoning/thinking content
/// - `toolCall`: Tool invocation request
/// - `toolResult`: Tool execution result
/// - `error`: Error event
/// - `done`: Terminal event with final message
///
/// Returns Ok(()) when the stream completes successfully.
#[tauri::command]
pub async fn stream_chat(
    context: State<'_, Arc<ServiceContext>>,
    request: SendMessageRequest,
    on_event: Channel<AiStreamEvent>,
) -> CommandResult<()> {
    let service = context.ai_chat_service();

    let mut event_stream = service.send_message(request).await?;

    // Stream events to the frontend via the Tauri channel
    while let Some(event) = event_stream.next().await {
        if let Err(e) = on_event.send(event) {
            log::error!("Failed to send AI event to channel: {}", e);
            break;
        }
    }

    Ok(())
}

// ============================================================================
// Thread Management Commands
// ============================================================================

/// List all chat threads with cursor-based pagination and optional search.
///
/// Returns a `ThreadPage` with threads, next_cursor, and has_more flag.
#[tauri::command]
pub async fn list_threads(
    context: State<'_, Arc<ServiceContext>>,
    cursor: Option<String>,
    limit: Option<u32>,
    search: Option<String>,
) -> CommandResult<ThreadPage> {
    let service = context.ai_chat_service();
    let request = ListThreadsRequest {
        cursor,
        limit,
        search,
    };
    let page = service.list_threads_paginated(&request)?;
    Ok(page)
}

/// Get a single chat thread by ID.
#[tauri::command]
pub async fn get_thread(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
) -> CommandResult<Option<ChatThread>> {
    let service = context.ai_chat_service();
    let thread = service.get_thread(&thread_id)?;
    Ok(thread)
}

/// Create a new chat thread.
#[tauri::command]
pub async fn create_thread(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<ChatThread> {
    let service = context.ai_chat_service();
    let thread = service.create_thread().await?;
    Ok(thread)
}

/// Update a chat thread's title and/or pinned status.
#[tauri::command]
pub async fn update_thread(
    context: State<'_, Arc<ServiceContext>>,
    request: UpdateThreadRequest,
) -> CommandResult<ChatThread> {
    let service = context.ai_chat_service();

    // Update title if provided
    if let Some(title) = request.title {
        service.update_thread_title(&request.id, title).await?;
    }

    // Update pinned status if provided
    if let Some(is_pinned) = request.is_pinned {
        service.update_thread_pinned(&request.id, is_pinned).await?;
    }

    // Get updated thread
    let thread = service
        .get_thread(&request.id)?
        .ok_or_else(|| AiError::ThreadNotFound(request.id.clone()))?;
    Ok(thread)
}

/// Delete a chat thread and all its messages.
#[tauri::command]
pub async fn delete_thread(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
) -> CommandResult<()> {
    let service = context.ai_chat_service();
    service.delete_thread(&thread_id).await?;
    Ok(())
}

/// Pin a thread to the top of the list.
#[tauri::command]
pub async fn pin_thread(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
    pinned: bool,
) -> CommandResult<ChatThread> {
    let service = context.ai_chat_service();
    service.update_thread_pinned(&thread_id, pinned).await?;

    let thread = service
        .get_thread(&thread_id)?
        .ok_or_else(|| AiError::ThreadNotFound(thread_id))?;
    Ok(thread)
}

// ============================================================================
// Provider and Model Commands
// ============================================================================

/// Simple provider info response for listing available providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub icon: String,
    pub description: String,
    pub default_model: String,
    pub documentation_url: String,
}

/// List all available AI providers from the catalog.
#[tauri::command]
pub async fn list_providers(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<Vec<ProviderInfo>> {
    let catalog = context.ai_provider_service().get_provider_catalog();
    let providers = catalog
        .into_iter()
        .map(|p| ProviderInfo {
            id: p.id,
            name: p.name,
            provider_type: p.provider_type,
            icon: p.icon,
            description: p.description,
            default_model: p.default_model,
            documentation_url: p.documentation_url.unwrap_or_default(),
        })
        .collect();
    Ok(providers)
}

/// Get the current AI settings (selected provider, model, API key status).
///
/// Returns a merged view of all providers combining catalog defaults
/// with user-configured settings from the settings service.
#[tauri::command]
pub async fn get_ai_settings(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<wealthvn_ai::provider_model::AiProvidersResponse> {
    let catalog = context.ai_provider_service().get_provider_catalog();
    let capabilities = context.ai_provider_service().get_capabilities();
    let settings_service = context.settings_service();

    // Get the default provider from settings
    let default_provider = settings_service
        .get_setting_value("ai_default_provider")
        .ok();

    // Merge each provider with user settings
    let mut merged_providers = Vec::new();
    for provider in catalog {
        let provider_id = provider.id.clone();

        // Get user settings for this provider
        let enabled_key = format!("ai_provider_{}_enabled", provider_id);
        let url_key = format!("ai_provider_{}_url", provider_id);
        let favorite_models_key = format!("ai_provider_{}_favorite_models", provider_id);
        let model_overrides_key = format!("ai_provider_{}_model_overrides", provider_id);
        let tools_allowlist_key = format!("ai_provider_{}_tools_allowlist", provider_id);
        let selected_model_key = format!("ai_provider_{}_selected_model", provider_id);

        // Parse user settings
        let enabled: bool = settings_service
            .get_setting_value(&enabled_key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(provider.default_config.enabled);

        let custom_url = settings_service
            .get_setting_value(&url_key)
            .ok()
            .filter(|v| !v.is_empty());

        let selected_model = settings_service
            .get_setting_value(&selected_model_key)
            .ok()
            .filter(|v| !v.is_empty());

        let favorite_models: Vec<String> = settings_service
            .get_setting_value(&favorite_models_key)
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();

        let model_capability_overrides: std::collections::HashMap<String, wealthvn_ai::provider_model::ModelCapabilityOverrides> =
            settings_service
                .get_setting_value(&model_overrides_key)
                .ok()
                .and_then(|v| serde_json::from_str(&v).ok())
                .unwrap_or_default();

        let tools_allowlist: Option<Vec<String>> = settings_service
            .get_setting_value(&tools_allowlist_key)
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok());

        // Check if provider has API key
        let has_api_key = context.ai_provider_service().has_api_key(&provider_id);

        // Build merged models list
        let mut merged_models = Vec::new();
        for model in &provider.models {
            let has_overrides = model_capability_overrides.contains_key(&model.id);
            // Start with catalog capabilities
            let mut capabilities = model.capabilities.clone();
            // Apply overrides if present
            if let Some(overrides) = model_capability_overrides.get(&model.id) {
                if let Some(v) = overrides.tools {
                    capabilities.tools = v;
                }
                if let Some(v) = overrides.thinking {
                    capabilities.thinking = v;
                }
                if let Some(v) = overrides.vision {
                    capabilities.vision = v;
                }
                if let Some(v) = overrides.streaming {
                    capabilities.streaming = v;
                }
            }

            merged_models.push(wealthvn_ai::provider_model::MergedModel {
                id: model.id.clone(),
                name: None,
                capabilities,
                is_catalog: true,
                is_favorite: favorite_models.contains(&model.id),
                has_capability_overrides: has_overrides,
            });
        }

        // Determine if this is the default provider
        let is_default = default_provider.as_ref() == Some(&provider_id);

        merged_providers.push(wealthvn_ai::provider_model::MergedProvider {
            // From catalog
            id: provider.id.clone(),
            name: provider.name.clone(),
            provider_type: provider.provider_type.clone(),
            icon: provider.icon.clone(),
            description: provider.description.clone(),
            env_key: provider.env_key.clone().unwrap_or_default(),
            connection_fields: provider.connection_fields.clone(),
            models: merged_models,
            default_model: provider.default_model.clone(),
            documentation_url: provider.documentation_url.unwrap_or_default(),

            // From user settings
            enabled,
            favorite: false, // Not implemented yet
            selected_model: selected_model.clone(),
            custom_url: custom_url.clone(),
            priority: provider.default_config.priority,
            favorite_models: favorite_models.clone(),
            model_capability_overrides,
            tools_allowlist,

            // Computed
            has_api_key,
            is_default,
            supports_model_listing: provider.provider_type == "api" || provider.provider_type == "local",
        });
    }

    // Sort by priority
    merged_providers.sort_by_key(|p| p.priority);

    Ok(wealthvn_ai::provider_model::AiProvidersResponse {
        providers: merged_providers,
        capabilities,
        default_provider,
    })
}

/// Get capability info for AI features.
#[tauri::command]
pub async fn get_capabilities(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<std::collections::HashMap<String, wealthvn_ai::CapabilityInfo>> {
    Ok(context.ai_provider_service().get_capabilities())
}

// ============================================================================
// Secret Management Commands
// ============================================================================

/// Save an API key for an AI provider.
/// Uses the OS keyring for secure storage.
#[tauri::command]
pub async fn save_api_key(
    provider_id: String,
    api_key: String,
    context: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    context
        .ai_provider_service()
        .set_api_key(&provider_id, &api_key)
        .map_err(|e| e.to_string())
}

/// Get an API key for an AI provider.
/// Retrieves from OS keyring.
#[tauri::command]
pub async fn get_api_key(
    provider_id: String,
    context: State<'_, Arc<ServiceContext>>,
) -> Result<Option<String>, String> {
    context
        .ai_provider_service()
        .get_api_key(&provider_id)
        .map_err(|e| e.to_string())
}

/// Delete an API key for an AI provider.
/// Removes from OS keyring.
#[tauri::command]
pub async fn delete_api_key(
    provider_id: String,
    context: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    context
        .ai_provider_service()
        .delete_api_key(&provider_id)
        .map_err(|e| e.to_string())
}

/// Check if a provider has an API key stored.
#[tauri::command]
pub async fn has_api_key(
    provider_id: String,
    context: State<'_, Arc<ServiceContext>>,
) -> Result<bool, String> {
    Ok(context.ai_provider_service().has_api_key(&provider_id))
}

// ============================================================================
// Thread Messages Commands
// ============================================================================

/// Get all messages for a chat thread.
#[tauri::command]
pub async fn get_thread_messages(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
) -> CommandResult<Vec<wealthvn_ai::ChatMessage>> {
    let service = context.ai_chat_service();
    let messages = service.get_messages(&thread_id)?;
    Ok(messages)
}

// ============================================================================
// Thread Tags Commands
// ============================================================================

/// Add a tag to a thread.
#[tauri::command]
pub async fn add_thread_tag(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
    tag: String,
) -> CommandResult<()> {
    let repository = context.ai_chat_repository();
    repository.add_tag(&thread_id, &tag).await?;
    Ok(())
}

/// Remove a tag from a thread.
#[tauri::command]
pub async fn remove_thread_tag(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
    tag: String,
) -> CommandResult<()> {
    let repository = context.ai_chat_repository();
    repository.remove_tag(&thread_id, &tag).await?;
    Ok(())
}

/// Get all tags for a thread.
#[tauri::command]
pub async fn get_thread_tags(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
) -> CommandResult<Vec<String>> {
    let repository = context.ai_chat_repository();
    let tags = repository.get_tags(&thread_id)?;
    Ok(tags)
}

// ============================================================================
// Provider Settings Commands
// ============================================================================

use wealthvn_ai::provider_model::ModelCapabilityOverrides;

/// Update settings for a specific AI provider.
///
/// This command handles updating all aspects of a provider's configuration:
/// - enabled/disabled state
/// - API key (via secret storage)
/// - custom URL for local providers
/// - favorite models list
/// - model capability overrides
/// - tools allowlist
#[tauri::command]
pub async fn update_provider_settings(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
    enabled: Option<bool>,
    custom_url: Option<String>,
    favorite_models: Option<Vec<String>>,
    model_capability_override: Option<ModelCapabilityOverrideData>,
    tools_allowlist: Option<Option<Vec<String>>>,
    selected_model: Option<String>,
) -> Result<(), String> {
    // Handle enabled state - store in settings service
    if let Some(enabled_val) = enabled {
        let key = format!("ai_provider_{}_enabled", provider_id);
        let value = if enabled_val { "true" } else { "false" };
        context
            .settings_service()
            .set_setting_value(&key, value)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Handle custom URL - store in settings service
    if let Some(url) = custom_url {
        let key = format!("ai_provider_{}_url", provider_id);
        context
            .settings_service()
            .set_setting_value(&key, &url)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Handle favorite models - store as JSON array
    if let Some(models) = favorite_models {
        let key = format!("ai_provider_{}_favorite_models", provider_id);
        let json_value = serde_json::to_string(&models)
            .map_err(|e| format!("Failed to serialize favorite models: {}", e))?;
        context
            .settings_service()
            .set_setting_value(&key, &json_value)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Handle model capability override - store as JSON object
    if let Some(override_data) = model_capability_override {
        let key = format!("ai_provider_{}_model_overrides", provider_id);
        // Get existing overrides
        let mut overrides_map: std::collections::HashMap<String, ModelCapabilityOverrides> =
            context
                .settings_service()
                .get_setting_value(&key)
                .ok()
                .and_then(|v| serde_json::from_str(&v).ok())
                .unwrap_or_default();

        if let Some(overrides) = override_data.overrides {
            overrides_map.insert(override_data.model_id, overrides);
        } else {
            overrides_map.remove(&override_data.model_id);
        }

        let json_value = serde_json::to_string(&overrides_map)
            .map_err(|e| format!("Failed to serialize model overrides: {}", e))?;
        context
            .settings_service()
            .set_setting_value(&key, &json_value)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Handle tools allowlist - store as JSON array (null means all enabled)
    if let Some(allowlist) = tools_allowlist {
        let key = format!("ai_provider_{}_tools_allowlist", provider_id);
        let json_value = serde_json::to_string(&allowlist)
            .map_err(|e| format!("Failed to serialize tools allowlist: {}", e))?;
        context
            .settings_service()
            .set_setting_value(&key, &json_value)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Handle selected model
    if let Some(model) = selected_model {
        let key = format!("ai_provider_{}_selected_model", provider_id);
        context
            .settings_service()
            .set_setting_value(&key, &model)
            .await
            .map_err(|e: wealthvn_core::Error| e.to_string())?;
    }

    // Note: API key is handled separately via the secret management commands
    // (set_secret, get_secret, delete_secret) which use the OS keyring

    Ok(())
}

/// Helper struct for deserializing model capability override data
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilityOverrideData {
    model_id: String,
    overrides: Option<ModelCapabilityOverrides>,
}

/// Set or clear the default AI provider.
#[tauri::command]
pub async fn set_default_provider(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
) -> Result<(), String> {
    context
        .settings_service()
        .set_setting_value("ai_default_provider", &provider_id)
        .await
        .map_err(|e: wealthvn_core::Error| e.to_string())?;
    log::info!("Set default AI provider to: {}", provider_id);
    Ok(())
}

/// List available models from a provider.
///
/// For API providers, this queries the provider's API to get available models.
/// For local providers, it returns models from the provider configuration.
#[tauri::command]
pub async fn list_ai_models(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
) -> Result<Vec<wealthvn_ai::provider_model::FetchedModel>, String> {
    let service = context.ai_provider_service();

    // Check if provider has an API key (if required)
    let has_api_key = service.has_api_key(&provider_id);

    // For now, return empty list if no API key
    // TODO: Implement actual API calls to list models from providers
    if !has_api_key && provider_id != "ollama" {
        return Ok(vec![]);
    }

    // For Ollama, we could implement local model listing
    // For API providers, we'd need to call their respective APIs
    log::info!("Listing models for provider: {} (has_api_key: {})", provider_id, has_api_key);

    // Return empty for now - this will be implemented with actual API calls
    Ok(vec![])
}

// ============================================================================
// Tool Result Commands
// ============================================================================

/// Update a tool result in the database.
/// Used to persist state like submission status after user actions.
#[tauri::command]
pub async fn update_tool_result(
    _context: State<'_, Arc<ServiceContext>>,
    _thread_id: String,
    _tool_call_id: String,
    _result_patch: serde_json::Value,
) -> CommandResult<()> {
    // TODO: Implement tool result persistence
    // For now, this is a no-op as tool results are stored in the message content
    Ok(())
}
