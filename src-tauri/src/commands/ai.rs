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
#[tauri::command]
pub async fn get_ai_settings(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<wealthvn_ai::SimpleSettings> {
    Ok(context.ai_provider_service().get_settings()?)
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

/// Update settings for a specific AI provider.
#[tauri::command]
pub async fn update_provider_settings(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
    enabled: Option<bool>,
    api_key: Option<String>,
) -> Result<(), String> {
    let service = context.ai_provider_service();
    if let Some(key) = api_key {
        service.set_api_key(&provider_id, &key).map_err(|e| e.to_string())?;
    }
    // TODO: Handle enabled flag when provider service supports it
    Ok(())
}

/// Set or clear the default AI provider.
#[tauri::command]
pub async fn set_default_provider(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
) -> Result<(), String> {
    // TODO: Implement default provider setting
    log::info!("Setting default provider to: {}", provider_id);
    Ok(())
}

/// List available models from a provider.
#[tauri::command]
pub async fn list_ai_models(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
) -> Result<Vec<wealthvn_ai::FetchedModel>, String> {
    // TODO: Implement model listing
    log::info!("Listing models for provider: {}", provider_id);
    Ok(vec![])
}

// ============================================================================
// Tool Result Commands
// ============================================================================

/// Update a tool result in the database.
/// Used to persist state like submission status after user actions.
#[tauri::command]
pub async fn update_tool_result(
    context: State<'_, Arc<ServiceContext>>,
    thread_id: String,
    tool_call_id: String,
    result_patch: serde_json::Value,
) -> CommandResult<()> {
    // TODO: Implement tool result persistence
    // For now, this is a no-op as tool results are stored in the message content
    Ok(())
}
