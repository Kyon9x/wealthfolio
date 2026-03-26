/**
 * MessageList - Displays a list of chat messages with support for streaming.
 */

import { memo, useState } from "react";

import { Icons } from "@/ui";
import { cn } from "@/lib/utils";
import type { ExternalMessage, ExternalMessagePart } from "../types";

interface MessageListProps {
  messages: ExternalMessage[];
  streamState?: {
    isStreaming: boolean;
    assistantMessage: string;
    reasoningMessage: string;
    toolCalls: unknown[];
    error: string | null;
  };
}

export function MessageList({ messages }: MessageListProps) {
  return (
    <div className="space-y-6">
      {messages.map((message) => (
        <Message key={message.id} message={message} />
      ))}
    </div>
  );
}

interface MessageProps {
  message: ExternalMessage;
}

const Message = memo(function Message({ message }: MessageProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={cn(
        "flex w-full",
        isUser ? "justify-end" : "justify-start",
      )}
    >
      <div
        className={cn(
          "max-w-[85%] rounded-3xl px-4 py-2.5",
          isUser
            ? "bg-primary text-primary-foreground rounded-tr-sm"
            : "bg-muted rounded-tl-sm",
        )}
      >
        <MessageContent message={message} />
      </div>
    </div>
  );
});

interface MessageContentProps {
  message: ExternalMessage;
}

function MessageContent({ message }: MessageContentProps) {
  return (
    <div className="text-sm space-y-2">
      {message.parts.map((part, index) => (
        <MessagePart key={index} part={part} />
      ))}
    </div>
  );
}

interface MessagePartProps {
  part: ExternalMessagePart;
}

function MessagePart({ part }: MessagePartProps) {
  switch (part.type) {
    case "text":
      return <TextContent content={part.content ?? ""} />;
    case "reasoning":
      return <ReasoningContent content={part.content ?? ""} />;
    case "toolCall":
      return <ToolCallContent part={part as ToolCallPart} />;
    default:
      return null;
  }
}

interface TextContentProps {
  content: string;
}

function TextContent({ content }: TextContentProps) {
  // Safe rendering by treating content as plain text with line breaks
  const lines = content.split("\n");

  return (
    <div className="space-y-1">
      {lines.map((line, i) => (
        <p key={i} className="leading-relaxed">
          {line || "\u00A0"}
        </p>
      ))}
    </div>
  );
}

interface ReasoningContentProps {
  content: string;
}

function ReasoningContent({ content }: ReasoningContentProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <details className="group">
      <summary
        className="flex cursor-pointer items-center gap-2 text-muted-foreground text-xs font-medium hover:text-foreground transition-colors list-none"
        onClick={(e) => {
          e.preventDefault();
          setIsExpanded(!isExpanded);
        }}
      >
        <Icons.HelpCircle className="size-4" />
        <span>Reasoning</span>
        <Icons.ChevronDown
          className={cn(
            "size-4 transition-transform",
            isExpanded && "rotate-180",
          )}
        />
      </summary>
      {isExpanded && (
        <div className="mt-2 text-muted-foreground text-xs whitespace-pre-wrap bg-background/50 rounded p-2">
          {content}
        </div>
      )}
    </details>
  );
}

interface ToolCallPart extends ExternalMessagePart {
  type: "toolCall";
  toolCallId: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: unknown;
  meta?: Record<string, unknown>;
}

interface ToolCallContentProps {
  part: ToolCallPart;
}

function ToolCallContent({ part }: ToolCallContentProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const result = part.result as { data?: unknown; error?: string } | undefined;
  const hasError = result?.error !== undefined;
  const isComplete = part.result !== undefined;

  return (
    <div
      className={cn(
        "rounded-lg border overflow-hidden",
        hasError ? "border-destructive/50" : "border-border",
      )}
    >
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-muted/50 transition-colors"
      >
        <div className="flex items-center gap-2">
          {hasError ? (
            <Icons.XCircle className="size-4 text-destructive" />
          ) : isComplete ? (
            <Icons.Check className="size-4 text-green-500" />
          ) : (
            <Icons.Spinner className="size-4 animate-spin" />
          )}
          <span className="text-sm font-medium">{part.name}</span>
        </div>
        <Icons.ChevronDown
          className={cn(
            "size-4 transition-transform text-muted-foreground",
            isExpanded && "rotate-180",
          )}
        />
      </button>

      {isExpanded && (
        <div className="border-t p-3 space-y-3">
          {/* Arguments */}
          <div>
            <p className="text-xs font-medium text-muted-foreground mb-1">Arguments:</p>
            <pre className="text-xs bg-background rounded p-2 overflow-x-auto">
              {JSON.stringify(part.arguments, null, 2)}
            </pre>
          </div>

          {/* Result */}
          {part.result !== undefined && (
            <div>
              <p className="text-xs font-medium text-muted-foreground mb-1">Result:</p>
              {hasError ? (
                <p className="text-xs text-destructive">
                  {result.error}
                </p>
              ) : (
                <pre className="text-xs bg-background rounded p-2 overflow-x-auto">
                  {typeof result === "string"
                    ? result
                    : JSON.stringify(result, null, 2)}
                </pre>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
