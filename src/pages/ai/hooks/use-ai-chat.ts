/**
 * useAiChat hook for streaming AI chat responses.
 * Provides a simplified interface for sending messages and handling streaming responses.
 */

import { useCallback, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  streamAiChat,
  type AiSendMessageRequest,
  type AiChatModelConfig,
  type ChatMessage,
  parseErrorCode,
} from "@/adapters/ai";

/** Streaming state for the chat hook */
export interface ChatStreamState {
  isStreaming: boolean;
  error: string | null;
  assistantMessage: string;
  reasoningMessage: string;
  toolCalls: ToolCallState[];
}

/** Individual tool call state */
export interface ToolCallState {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: unknown;
  isComplete: boolean;
}

/** Hook result */
export interface UseAiChatResult {
  /** Send a message and stream the response */
  sendMessage: (content: string, threadId?: string) => Promise<void>;
  /** Cancel the current stream */
  cancelStream: () => void;
  /** Current streaming state */
  streamState: ChatStreamState;
}

/** Hook options */
export interface UseAiChatOptions {
  /** Model configuration (provider and model selection) */
  config?: AiChatModelConfig;
  /** Callback when a new thread is created */
  onThreadIdCreated?: (threadId: string) => void;
  /** Callback when thread title is updated */
  onThreadTitleUpdated?: (threadId: string, title: string) => void;
  /** Callback when stream completes */
  onStreamComplete?: (message: ChatMessage) => void;
}

/**
 * Hook for streaming AI chat responses.
 * Handles message sending, streaming updates, and error handling.
 */
export function useAiChat(options: UseAiChatOptions = {}): UseAiChatResult {
  const { config, onThreadIdCreated, onThreadTitleUpdated, onStreamComplete } = options;
  const queryClient = useQueryClient();

  const abortControllerRef = useRef<AbortController | null>(null);
  const threadIdRef = useRef<string | null>(null);

  const [streamState, setStreamState] = useState<ChatStreamState>({
    isStreaming: false,
    error: null,
    assistantMessage: "",
    reasoningMessage: "",
    toolCalls: [],
  });

  /** Reset stream state for a new message */
  const resetStreamState = useCallback(() => {
    setStreamState({
      isStreaming: true,
      error: null,
      assistantMessage: "",
      reasoningMessage: "",
      toolCalls: [],
    });
  }, []);

  /** Cancel the current stream */
  const cancelStream = useCallback(() => {
    abortControllerRef.current?.abort();
    setStreamState((prev) => ({ ...prev, isStreaming: false }));
  }, []);

  /** Send a message and stream the response */
  const sendMessage = useCallback(
    async (content: string, existingThreadId?: string) => {
      // Cancel any existing stream
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      // Create new abort controller
      abortControllerRef.current = new AbortController();
      const { signal } = abortControllerRef.current;

      // Reset state for new stream
      resetStreamState();

      // Build request
      const request: AiSendMessageRequest = {
        content,
        threadId: existingThreadId ?? threadIdRef.current ?? undefined,
        config,
      };

      try {
        for await (const event of streamAiChat(request, signal)) {
          if (signal.aborted) break;

          switch (event.type) {
            case "system":
              // Capture thread ID from system event
              if (event.threadId) {
                threadIdRef.current = event.threadId;
                onThreadIdCreated?.(event.threadId);
              }
              break;

            case "threadTitleUpdated":
              if (event.threadId && event.title) {
                onThreadTitleUpdated?.(event.threadId, event.title);
              }
              break;

            case "textDelta":
              setStreamState((prev) => ({
                ...prev,
                assistantMessage: prev.assistantMessage + event.delta,
                reasoningMessage: "", // Clear reasoning when text starts
              }));
              break;

            case "reasoningDelta":
              setStreamState((prev) => ({
                ...prev,
                reasoningMessage: prev.reasoningMessage + event.delta,
              }));
              break;

            case "toolCall": {
              const newToolCall: ToolCallState = {
                id: event.toolCall.id,
                name: event.toolCall.name,
                arguments: event.toolCall.arguments,
                isComplete: false,
              };
              setStreamState((prev) => ({
                ...prev,
                toolCalls: [...prev.toolCalls, newToolCall],
              }));
              break;
            }

            case "toolResult": {
              setStreamState((prev) => ({
                ...prev,
                toolCalls: prev.toolCalls.map((tc) =>
                  tc.id === event.result.toolCallId
                    ? { ...tc, result: event.result.data, isComplete: true }
                    : tc,
                ),
              }));
              break;
            }

            case "done":
              setStreamState((prev) => ({ ...prev, isStreaming: false }));
              onStreamComplete?.(event.message);
              // Invalidate threads cache to refresh from DB
              queryClient.invalidateQueries({ queryKey: ["ai_threads"] });
              break;

            case "error":
              const parsedError = parseErrorCode(event.code, event.message);
              setStreamState((prev) => ({
                ...prev,
                isStreaming: false,
                error: parsedError.message,
              }));
              break;
          }
        }
      } catch (error) {
        if (!signal.aborted) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          const parsedError = parseErrorCode("network", errorMessage);
          setStreamState((prev) => ({
            ...prev,
            isStreaming: false,
            error: parsedError.message,
          }));
        }
      }
    },
    [config, onThreadIdCreated, onThreadTitleUpdated, onStreamComplete, queryClient, resetStreamState],
  );

  return {
    sendMessage,
    cancelStream,
    streamState,
  };
}
