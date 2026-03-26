//! WealthVN AI - LLM orchestration using rig-core.
//!
//! This crate provides the AI assistant functionality for WealthVN,
//! handling the model <-> tools <-> model orchestration loop and streaming
//! `AiStreamEvent` to Tauri/Axum consumers.

pub mod chat;
pub mod env;
pub mod error;
pub mod provider_model;
pub mod providers;
pub mod title_generator;
pub mod tools;
pub mod types;

// Re-export main types for convenience
pub use env::{AiEnvironment, SecretStore, SecretStoreError, TauriAiEnvironment};
pub use error::AiError;
pub use providers::{AiProviderServiceTrait, ProviderService, SimpleSettings};

// Chat service types
pub use chat::{ChatConfig, ChatService};

// Title generator
pub use title_generator::{TitleGenerator, TitleGeneratorConfig, TitleGeneratorTrait, truncate_to_title};

// Tool types
pub use tools::{
    DEFAULT_TOOLS_ALLOWLIST as TOOLS_ALLOWLIST, GetAccountsArgs, GetGoalsArgs,
    GetHoldingsArgs, GetPerformanceArgs, SearchActivitiesArgs, GetValuationsArgs,
    ToolDefinition, ToolRegistry, ToolContext, ToolResult, ToolWithContext, ToolSet,
    // Constants
    DEFAULT_PAGE_SIZE, DEFAULT_TOOLS_ALLOWLIST, DEFAULT_VALUATIONS_DAYS,
    MAX_ACCOUNTS, MAX_ACTIVITIES_ROWS, MAX_GOALS, MAX_HOLDINGS,
    MAX_VALUATIONS_POINTS,
};
pub use types::{
    // Streaming and request types
    AiStreamEvent,
    // Domain types (chat thread, message, content)
    ChatMessage,
    ChatMessageContent,
    ChatMessagePart,
    ChatMessageRole,
    ChatModelConfig,
    ChatRepositoryResult,
    ChatRepositoryTrait,
    ChatThread,
    ChatThreadConfig,
    // Pagination types
    ListThreadsRequest,
    SendMessageRequest,
    SimpleChatMessage,
    ThreadPage,
    ToolCall,
    ToolResultData,
    UsageStats,
    // Constants
    CHAT_CONFIG_SCHEMA_VERSION,
    CHAT_CONTENT_SCHEMA_VERSION,
    CHAT_MAX_CONTENT_SIZE_BYTES,
};

// Provider model types
pub use provider_model::{
    // Catalog types
    AiProviderCatalog,
    // User settings types
    AiProviderSettings,
    // Merged view types
    AiProvidersResponse,
    CapabilityInfo,
    CatalogModel,
    CatalogProvider,
    ConnectionField,
    // Provider config types
    FetchedModel,
    ListModelsResponse,
    MergedModel,
    MergedProvider,
    ModelCapabilities,
    // Update types
    ModelCapabilityOverrideUpdate,
    ModelCapabilityOverrides,
    // Provider API error
    ProviderApiError,
    ProviderConfig,
    ProviderDefaultConfig,
    ProviderUserSettings,
    SetDefaultProviderRequest,
    UpdateProviderSettingsRequest,
    // Constants
    AI_PROVIDER_SETTINGS_KEY,
    AI_PROVIDER_SETTINGS_SCHEMA_VERSION,
};
