// AI Chat Adapter - Tauri-specific implementation for WealthVN
// Provides streaming chat interface and AI provider management

import { Channel } from "@tauri-apps/api/core";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { invokeTauri } from "./index";

// ============================================================================
// Type Definitions matching the Rust API
// ============================================================================

/**
 * Model configuration for AI chat requests.
 */
export interface AiChatModelConfig {
  /** Provider ID (e.g., "openai", "anthropic"). */
  provider?: string;
  /** Model ID (e.g., "gpt-4o", "claude-3-sonnet"). */
  model?: string;
  /** Override thinking/reasoning capability for this request. */
  thinking?: boolean;
}

/**
 * Request payload for sending an AI chat message.
 */
export interface AiSendMessageRequest {
  /** Thread ID (creates new thread if not provided). */
  threadId?: string;
  /** The message content. */
  content: string;
  /** Model configuration (provider and model selection). */
  config?: AiChatModelConfig;
  /** Override provider ID (uses default if not specified). @deprecated Use config.provider */
  providerId?: string;
  /** Override model ID (uses provider default if not specified). @deprecated Use config.model */
  modelId?: string;
  /** Tool allowlist for this request (uses all if not specified). */
  allowedTools?: string[];
  /** Parent message ID for edit operations. When set, AI context is truncated to this message. */
  parentMessageId?: string;
}

/**
 * A single part of a message's content.
 */
export type ChatMessagePart =
  | { type: "system"; content: string }
  | { type: "text"; content: string }
  | { type: "reasoning"; content: string }
  | {
      type: "toolCall";
      toolCallId: string;
      name: string;
      arguments: Record<string, unknown>;
    }
  | {
      type: "toolResult";
      toolCallId: string;
      success: boolean;
      data: unknown;
      meta?: Record<string, unknown>;
      error?: string;
    }
  | { type: "error"; code: string; message: string };

/**
 * Structured message content from the backend.
 */
export interface ChatMessageContent {
  schemaVersion: number;
  parts: ChatMessagePart[];
  truncated?: boolean;
}

/**
 * AI chat message from the backend.
 */
export interface ChatMessage {
  id: string;
  threadId: string;
  role: "user" | "assistant";
  content: ChatMessageContent;
  createdAt: string;
}

/**
 * AI thread structure from the backend API.
 */
export interface AiThread {
  id: string;
  title: string;
  isPinned: boolean;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

/**
 * Paginated response for AI threads.
 */
export interface ThreadPage {
  threads: AiThread[];
  nextCursor: string | null;
  hasMore: boolean;
}

/**
 * Request parameters for listing AI threads.
 */
export interface ListThreadsRequest {
  cursor?: string;
  limit?: number;
  search?: string;
}

/**
 * Request for updating thread title or pinned status.
 */
export interface UpdateThreadRequest {
  id: string;
  title?: string;
  isPinned?: boolean;
}

/**
 * Tool call structure.
 */
export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

/**
 * Tool result structure.
 */
export interface ToolResult {
  toolCallId: string;
  success: boolean;
  data: unknown;
  meta?: Record<string, unknown>;
  error?: string;
}

/**
 * Token usage statistics.
 */
export interface UsageStats {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
}

// ============================================================================
// Stream Event Types
// ============================================================================

/**
 * Base fields present in all stream events for correlation.
 */
interface AiStreamEventBase {
  /** Thread ID for this conversation */
  threadId: string;
  /** Run ID for this streaming session (uuid7) */
  runId: string;
}

/**
 * System event - sent first in the stream with metadata.
 */
interface SystemEvent extends AiStreamEventBase {
  type: "system";
  /** The message ID being generated */
  messageId: string;
}

/**
 * Text delta event - partial text content.
 */
interface TextDeltaEvent extends AiStreamEventBase {
  type: "textDelta";
  /** The message ID this delta belongs to */
  messageId: string;
  /** The text content delta */
  delta: string;
}

/**
 * Reasoning delta event - partial reasoning/thinking content.
 */
interface ReasoningDeltaEvent extends AiStreamEventBase {
  type: "reasoningDelta";
  /** The message ID this delta belongs to */
  messageId: string;
  /** The reasoning content delta */
  delta: string;
}

/**
 * Tool call event - model wants to call a tool.
 */
interface ToolCallEvent extends AiStreamEventBase {
  type: "toolCall";
  /** The message ID this tool call belongs to */
  messageId: string;
  /** The tool call details (structured JSON) */
  toolCall: ToolCall;
}

/**
 * Tool result event - tool execution completed.
 */
interface ToolResultEvent extends AiStreamEventBase {
  type: "toolResult";
  /** The message ID this result belongs to */
  messageId: string;
  /** The tool result (structured JSON) */
  result: ToolResult;
}

/**
 * Error event - something went wrong.
 */
interface ErrorEvent extends AiStreamEventBase {
  type: "error";
  /** The message ID (if available) */
  messageId?: string;
  /** Error code for programmatic handling */
  code: string;
  /** Human-readable error message */
  message: string;
}

/**
 * Done event - stream completed (terminal).
 */
interface DoneEvent extends AiStreamEventBase {
  type: "done";
  /** The message ID of the completed message */
  messageId: string;
  /** The final complete message */
  message: ChatMessage;
  /** Usage statistics (if available) */
  usage?: UsageStats;
}

/**
 * Thread title updated event.
 */
interface ThreadTitleUpdatedEvent extends AiStreamEventBase {
  type: "threadTitleUpdated";
  /** The new thread title */
  title: string;
}

/**
 * Union type for all stream events.
 */
export type AiStreamEvent =
  | SystemEvent
  | TextDeltaEvent
  | ReasoningDeltaEvent
  | ToolCallEvent
  | ToolResultEvent
  | ErrorEvent
  | DoneEvent
  | ThreadTitleUpdatedEvent;

// ============================================================================
// Error Handling Types
// ============================================================================

/**
 * Error information for display in the UI.
 */
export interface ChatError {
  /** Error code for programmatic handling */
  code: string;
  /** User-friendly error message */
  message: string;
  /** Whether the error is retryable */
  retryable: boolean;
}

/**
 * Maps error codes to user-friendly messages and retry eligibility.
 */
export const ERROR_CODE_MAP: Record<
  string,
  { message: string; retryable: boolean }
> = {
  providerNotConfigured: {
    message: "AI provider is not configured. Please set up a provider in Settings.",
    retryable: false,
  },
  missingApiKey: {
    message: "API key is missing. Please add your API key in Settings.",
    retryable: false,
  },
  modelNotFound: {
    message: "The selected model is not available. Please choose a different model.",
    retryable: false,
  },
  toolNotFound: {
    message: "A required tool is not available. Please try again.",
    retryable: true,
  },
  toolNotAllowed: {
    message: "A tool is not allowed for this conversation. Please try again.",
    retryable: true,
  },
  toolExecutionError: {
    message: "A tool failed to execute. Please try again.",
    retryable: true,
  },
  providerError: {
    message: "The AI provider returned an error. Please try again.",
    retryable: true,
  },
  threadNotFound: {
    message: "Conversation not found. Please start a new conversation.",
    retryable: false,
  },
  invalidInput: {
    message: "Invalid input. Please check your message and try again.",
    retryable: false,
  },
  internal: {
    message: "An unexpected error occurred. Please try again.",
    retryable: true,
  },
  cancelled: {
    message: "Response was cancelled.",
    retryable: true,
  },
  network: {
    message: "Network error. Please check your connection and try again.",
    retryable: true,
  },
};

/**
 * Parse an error code and return a ChatError with user-friendly message.
 */
export function parseErrorCode(code: string, rawMessage?: string): ChatError {
  const mapped = ERROR_CODE_MAP[code];
  if (mapped) {
    return { code, message: mapped.message, retryable: mapped.retryable };
  }
  return {
    code,
    message: rawMessage ?? "An unexpected error occurred. Please try again.",
    retryable: true,
  };
}

// ============================================================================
// Provider Types
// ============================================================================

/**
 * Model capabilities.
 */
export interface ModelCapabilities {
  tools?: boolean;
  thinking?: boolean;
  vision?: boolean;
  streaming?: boolean;
}

/**
 * Capability info from catalog.
 */
export interface CapabilityInfo {
  name: string;
  description: string;
  icon: string;
}

/**
 * Connection field for provider configuration.
 */
export interface ConnectionField {
  key: string;
  label: string;
  type: "text" | "password";
  placeholder: string;
  required: boolean;
  helpUrl?: string;
}

/**
 * Provider default config.
 */
export interface ProviderDefaultConfig {
  enabled: boolean;
  priority: number;
  url?: string;
}

/**
 * Model catalog entry.
 */
export interface ModelCatalogEntry {
  capabilities: ModelCapabilities;
}

/**
 * Provider catalog entry.
 */
export interface ProviderCatalogEntry {
  name: string;
  type: string;
  icon: string;
  description: string;
  envKey?: string;
  defaultConfig: ProviderDefaultConfig;
  connectionFields: ConnectionField[];
  models: Record<string, ModelCatalogEntry>;
  defaultModel: string;
  titleModelId?: string;
  documentationUrl?: string;
}

/**
 * Provider capabilities from backend.
 */
export interface ProviderCapabilities {
  instruments: string;
  coverage: string;
  features: string[];
}

/**
 * Model capability overrides for user settings.
 */
export interface ModelCapabilityOverrides {
  tools?: boolean;
  thinking?: boolean;
  vision?: boolean;
  streaming?: boolean;
}

/**
 * Fetched model from provider API.
 */
export interface FetchedModel {
  id: string;
  name: string;
  description?: string;
}

/**
 * Merged provider with catalog data and user settings.
 */
export interface MergedProvider {
  id: string;
  name: string;
  description: string;
  type: "api" | "local";
  icon: string;
  enabled: boolean;
  isDefault: boolean;
  hasApiKey: boolean;
  priority: number;
  customUrl?: string;
  documentationUrl?: string;
  supportsModelListing: boolean;
  connectionFields: ConnectionField[];
  models: MergedModel[];
  favoriteModels?: string[];
  selectedModel?: string;
  modelCapabilityOverrides: Record<string, ModelCapabilityOverrides>;
  toolsAllowlist?: string[] | null;
}

/**
 * Merged model with catalog data.
 */
export interface MergedModel {
  id: string;
  name: string;
  capabilities?: ModelCapabilities;
  isCatalog: boolean;
}

/**
 * AI provider setting with user configuration.
 */
export interface AiProviderSetting {
  id: string;
  name: string;
  description: string | null;
  url: string | null;
  priority: number;
  enabled: boolean;
  logoFilename: string | null;
  capabilities: ProviderCapabilities | null;
  requiresApiKey: boolean;
  hasApiKey: boolean;
  isDefault?: boolean;
}

/**
 * Response containing all AI providers.
 */
export interface AiProvidersResponse {
  providers: AiProviderSetting[];
}

/**
 * Request to update provider settings.
 */
export interface UpdateProviderSettingsRequest {
  providerId: string;
  enabled?: boolean;
  apiKey?: string;
  customUrl?: string;
  selectedModel?: string;
  favoriteModels?: string[];
  modelCapabilityOverride?: {
    modelId: string;
    overrides: ModelCapabilityOverrides;
  };
  toolsAllowlist?: string[] | null;
}

/**
 * Request to set default provider.
 */
export interface SetDefaultProviderRequest {
  providerId: string;
}

/**
 * Response from listing models.
 */
export interface ListModelsResponse {
  models: Array<{
    id: string;
    name: string;
    description?: string;
  }>;
}

/**
 * Request to update a tool result.
 */
export interface UpdateToolResultRequest {
  threadId: string;
  toolCallId: string;
  resultPatch: Record<string, unknown>;
}

// ============================================================================
// Streaming Utilities
// ============================================================================

/**
 * Stream AI chat responses via Tauri IPC.
 *
 * Uses Tauri's Channel for efficient streaming of events from the backend.
 *
 * @param request - The chat message request
 * @param signal - Optional AbortSignal for cancellation
 * @yields AiStreamEvent objects from the stream
 */
export async function* streamAiChat(
  request: AiSendMessageRequest,
  signal?: AbortSignal,
): AsyncGenerator<AiStreamEvent, void, undefined> {
  const channel = new Channel<AiStreamEvent>();
  const queue: AiStreamEvent[] = [];
  let done = false;
  let pendingResolve: (() => void) | null = null;

  const notifyPending = () => {
    if (pendingResolve) {
      pendingResolve();
      pendingResolve = null;
    }
  };

  channel.onmessage = (event: AiStreamEvent) => {
    queue.push(event);
    notifyPending();
  };

  const invokePromise = tauriInvoke("stream_ai_chat", {
    request,
    onEvent: channel,
  })
    .catch((err) => {
      queue.push({
        type: "error",
        threadId: request.threadId ?? "",
        runId: "",
        messageId: undefined,
        code: "network",
        message: err instanceof Error ? err.message : String(err),
      } as AiStreamEvent);
      notifyPending();
    })
    .finally(() => {
      done = true;
      notifyPending();
    });

  try {
    while (!done || queue.length > 0) {
      if (signal?.aborted) {
        break;
      }

      if (queue.length === 0) {
        await new Promise<void>((resolve) => {
          pendingResolve = resolve;
        });
        continue;
      }

      const next = queue.shift();
      if (next) {
        yield next;

        // Stop on terminal events
        if (next.type === "done" || next.type === "error") {
          return;
        }
      }
    }
  } finally {
    // Clear the channel handler to prevent memory leaks
    channel.onmessage = () => {};
    if (!signal?.aborted) {
      await invokePromise;
    }
  }
}

// ============================================================================
// Thread Management Functions
// ============================================================================

/**
 * List AI chat threads with cursor-based pagination and optional search.
 */
export async function listAiThreads(
  req?: ListThreadsRequest,
): Promise<ThreadPage> {
  return invokeTauri<ThreadPage>("list_threads", {
    cursor: req?.cursor,
    limit: req?.limit ?? 20,
    search: req?.search,
  });
}

/**
 * Get a single chat thread by ID.
 */
export async function getAiThread(
  threadId: string,
): Promise<AiThread | null> {
  return invokeTauri<AiThread | null>("get_thread", { threadId });
}

/**
 * Get all messages for a chat thread.
 */
export async function getAiThreadMessages(
  threadId: string,
): Promise<ChatMessage[]> {
  return invokeTauri<ChatMessage[]>("get_thread_messages", { threadId });
}

/**
 * Update a chat thread's title and/or pinned status.
 */
export async function updateAiThread(
  request: UpdateThreadRequest,
): Promise<AiThread> {
  return invokeTauri<AiThread>("update_thread", { request });
}

/**
 * Delete a chat thread and all its messages.
 */
export async function deleteAiThread(threadId: string): Promise<void> {
  return invokeTauri<void>("delete_thread", { threadId });
}

/**
 * Add a tag to a thread.
 */
export async function addAiThreadTag(
  threadId: string,
  tag: string,
): Promise<void> {
  return invokeTauri<void>("add_thread_tag", { threadId, tag });
}

/**
 * Remove a tag from a thread.
 */
export async function removeAiThreadTag(
  threadId: string,
  tag: string,
): Promise<void> {
  return invokeTauri<void>("remove_thread_tag", { threadId, tag });
}

/**
 * Get all tags for a thread.
 */
export async function getAiThreadTags(threadId: string): Promise<string[]> {
  return invokeTauri<string[]>("get_thread_tags", { threadId });
}

// ============================================================================
// Provider Management Functions
// ============================================================================

/**
 * Response from get_ai_settings backend command.
 */
interface BackendSimpleSettings {
  provider_id: string;
  model: string;
  has_api_key: boolean;
  providers: BackendSimpleProviderSetting[];
  capabilities: Record<string, CapabilityInfo>;
}

interface BackendSimpleProviderSetting {
  id: string;
  name: string;
  description: string;
  provider_type: string;
  icon: string;
  default_model: string;
  enabled: boolean;
  supports_custom_url: boolean;
  url: string | null;
  documentation_url: string | null;
  env_key: string | null;
  models: BackendSimpleModelInfo[];
}

interface BackendSimpleModelInfo {
  id: string;
  capabilities: ModelCapabilities;
}

/**
 * Get all AI providers merged with user settings.
 * Returns catalog data merged with user overrides and computed hasApiKey flag.
 */
export async function getAiProviders(): Promise<AiProvidersResponse> {
  const settings = await invokeTauri<BackendSimpleSettings>("get_ai_settings");

  // Transform backend response to frontend format
  const providers: AiProviderSetting[] = settings.providers.map((p) => ({
    id: p.id,
    name: p.name,
    description: p.description,
    url: p.url,
    priority: 0, // Not tracked in backend yet
    enabled: p.enabled,
    logoFilename: null, // Using icon string instead
    capabilities: null, // Capabilities are per-model in backend
    requiresApiKey: p.provider_type === "api",
    hasApiKey: false, // Will be checked per provider
    isDefault: p.id === settings.provider_id,
    // Additional fields for MergedProvider construction
    icon: p.icon,
    type: p.provider_type === "api" ? "api" : "local",
    models: p.models.map((m) => ({
      id: m.id,
      name: m.id,
      capabilities: m.capabilities,
      isCatalog: true,
    })),
    defaultModel: p.default_model,
    documentationUrl: p.documentation_url,
    supportsCustomUrl: p.supports_custom_url,
    connectionFields: [], // Not exposed yet
    favoriteModels: [],
    selectedModel: undefined,
    modelCapabilityOverrides: {},
    toolsAllowlist: undefined,
  }));

  return { providers };
}

/**
 * Update settings for a specific AI provider.
 */
export async function updateAiProviderSettings(
  request: UpdateProviderSettingsRequest,
): Promise<void> {
  return invokeTauri<void>("update_provider_settings", {
    providerId: request.providerId,
    enabled: request.enabled,
    apiKey: request.apiKey,
  });
}

/**
 * Set or clear the default AI provider.
 */
export async function setDefaultAiProvider(
  request: SetDefaultProviderRequest,
): Promise<void> {
  return invokeTauri<void>("set_default_provider", {
    providerId: request.providerId,
  });
}

/**
 * List available models from a provider.
 * Fetches models from the provider's API using backend-stored secrets.
 */
export async function listAiModels(
  providerId: string,
): Promise<ListModelsResponse> {
  return invokeTauri<ListModelsResponse>("list_ai_models", { providerId });
}

// ============================================================================
// Tool Result Functions
// ============================================================================

/**
 * Update a tool result in the database.
 * Used to persist state like submission status after user actions.
 */
export async function updateToolResult(
  request: UpdateToolResultRequest,
): Promise<void> {
  return invokeTauri<void>("update_tool_result", {
    threadId: request.threadId,
    toolCallId: request.toolCallId,
    resultPatch: request.resultPatch,
  });
}

// ============================================================================
// Secret Management Functions
// ============================================================================

/**
 * Set a secret value in the secure store.
 * Used for storing API keys for AI providers.
 */
export async function setSecret(
  key: string,
  value: string,
): Promise<void> {
  return invokeTauri<void>("set_secret", { key, value });
}

/**
 * Get a secret value from the secure store.
 * Returns null if the secret doesn't exist.
 */
export async function getSecret(
  key: string,
): Promise<string | null> {
  return invokeTauri<string | null>("get_secret", { key });
}

/**
 * Delete a secret value from the secure store.
 */
export async function deleteSecret(
  key: string,
): Promise<void> {
  return invokeTauri<void>("delete_secret", { key });
}

// ============================================================================
// NDJSON Parser for HTTP streaming (future web support)
// ============================================================================

/**
 * Parse an NDJSON stream into AI events.
 * Useful for web mode or when streaming over HTTP.
 *
 * @param response - Fetch response with NDJSON body
 * @yields Parsed AiStreamEvent objects
 */
export async function* parseNdjsonStream(
  response: Response,
): AsyncGenerator<AiStreamEvent, void, undefined> {
  if (!response.body) {
    throw new Error("Response body is null");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        // Process any remaining buffer content
        if (buffer.trim()) {
          try {
            const event = JSON.parse(buffer.trim()) as AiStreamEvent;
            yield event;
          } catch (parseError) {
            // Silently skip final incomplete JSON
          }
        }
        break;
      }

      // Decode chunk and add to buffer
      buffer += decoder.decode(value, { stream: true });

      // Split by newlines and process complete lines
      const lines = buffer.split("\n");

      // Keep the last incomplete line in the buffer
      buffer = lines.pop() ?? "";

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;

        try {
          const event = JSON.parse(trimmed) as AiStreamEvent;
          yield event;
        } catch {
          // Skip malformed lines
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
