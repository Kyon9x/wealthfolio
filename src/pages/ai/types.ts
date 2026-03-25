// AI Chat Types for WealthVN
// Extended types for the AI chat UI

import type {
  ChatMessage,
  ChatMessageContent,
  ChatMessagePart,
  AiThread,
  AiThread as Thread,
} from "@/adapters/ai";

// Re-export adapter types
export type { ChatMessage, ChatMessageContent, ChatMessagePart, AiThread };

/** Alias for thread to match naming conventions */
export type { Thread };

/**
 * External message format for use with assistant-ui runtime.
 * Preserves part ordering for inline tool UI rendering.
 */
export interface ExternalMessagePart {
  type: "text" | "reasoning" | "toolCall";
  content?: string;
  toolCallId?: string;
  name?: string;
  arguments?: Record<string, unknown>;
  result?: unknown;
  meta?: Record<string, unknown>;
}

/**
 * Message stored in external state (matches DB format loosely).
 * Uses ordered parts array to preserve interleaved text/tool positions.
 */
export interface ExternalMessage {
  id: string;
  role: "user" | "assistant";
  parts: ExternalMessagePart[];
  createdAt: Date;
}

/**
 * Chat model configuration for requests.
 */
export interface ChatModelConfig {
  /** Provider ID (e.g., "openai", "anthropic"). */
  provider?: string;
  /** Model ID (e.g., "gpt-4o", "claude-3-sonnet"). */
  model?: string;
  /** Override thinking/reasoning capability for this request. */
  thinking?: boolean;
}

/**
 * Thread list data for assistant-ui thread list adapter.
 */
export interface ThreadListItemData {
  status: "regular";
  id: string;
  remoteId?: string;
  externalId?: string;
  title?: string;
}

/** Paginated response for AI threads. */
export interface ThreadPage {
  threads: Thread[];
  nextCursor: string | null;
  hasMore: boolean;
}

/**
 * Chat error for display in the UI.
 */
export interface ChatError {
  code: string;
  message: string;
  retryable: boolean;
}
