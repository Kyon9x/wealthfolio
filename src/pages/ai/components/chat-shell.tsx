/**
 * ChatShell - Main chat interface with message list and input.
 * Provides a clean UI for AI chat conversations with streaming support.
 */

import { useState, useCallback, useEffect } from "react";

import {
  Button,
  Icons,
  Page,
  PageContent,
  PageHeader,
  Badge,
  ScrollArea,
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/ui";
import { cn } from "@/lib/utils";
import { generateId } from "@/lib/id";

import {
  useAiChat,
  useAiThreads,
  useToggleAiThreadPin,
  useDeleteAiThread,
  useAiThreadMessages,
  flattenThreadPages,
} from "../hooks";
import { MessageList } from "./message-list";
import { MessageInput } from "./message-input";
import { ThreadList } from "./thread-list";
import type { ExternalMessage, ExternalMessagePart } from "../types";

interface ChatShellProps {
  className?: string;
}

/** Debounce delay for search input (ms) */
const SEARCH_DEBOUNCE_MS = 300;

/**
 * Main chat shell component with thread sidebar and message panel.
 */
export function ChatShell({ className }: ChatShellProps) {
  const [searchValue, setSearchValue] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(searchValue);
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [searchValue]);

  // Fetch threads
  const {
    data: threadsData,
    fetchNextPage,
    hasNextPage,
    isLoading,
  } = useAiThreads(debouncedSearch);
  const threads = flattenThreadPages(threadsData?.pages);

  // Fetch messages for selected thread
  const { data: messages } = useAiThreadMessages(selectedThreadId);

  // Chat mutations
  const togglePin = useToggleAiThreadPin();
  const deleteThread = useDeleteAiThread();

  // Local message state for streaming
  const [localMessages, setLocalMessages] = useState<ExternalMessage[]>([]);

  // Load messages when thread is selected
  useEffect(() => {
    if (messages) {
      const externalMessages: ExternalMessage[] = messages.map((msg) => {
        const parts: ExternalMessagePart[] = [];
        const toolCallIndexes = new Map<string, number>();

        for (const part of msg.content.parts) {
          switch (part.type) {
            case "text":
              parts.push({ type: "text", content: part.content });
              break;
            case "reasoning":
              parts.push({ type: "reasoning", content: part.content });
              break;
            case "toolCall":
              toolCallIndexes.set(part.toolCallId, parts.length);
              parts.push({
                type: "toolCall",
                toolCallId: part.toolCallId,
                name: part.name,
                arguments: part.arguments,
              });
              break;
            case "toolResult": {
              const idx = toolCallIndexes.get(part.toolCallId);
              if (idx !== undefined) {
                const tcPart = parts[idx];
                if (tcPart?.type === "toolCall") {
                  tcPart.result = part.success
                    ? part.meta
                      ? { data: part.data, meta: part.meta }
                      : part.data
                    : { error: part.error };
                  tcPart.meta = part.meta;
                }
              }
              break;
            }
          }
        }

        return {
          id: msg.id,
          role: msg.role,
          parts,
          createdAt: new Date(msg.createdAt),
        };
      });
      setLocalMessages(externalMessages);
    } else {
      setLocalMessages([]);
    }
  }, [messages]);

  // Handle thread selection
  const handleSelectThread = useCallback((threadId: string | null) => {
    setSelectedThreadId(threadId);
    setSidebarOpen(false);
  }, []);

  // Handle new thread
  const handleNewThread = useCallback(() => {
    setSelectedThreadId(null);
    setLocalMessages([]);
    setSidebarOpen(false);
  }, []);

  // Handle thread deletion
  const handleDeleteThread = useCallback(
    (threadId: string) => {
      deleteThread.mutate(threadId);
      if (selectedThreadId === threadId) {
        handleNewThread();
      }
    },
    [deleteThread, selectedThreadId, handleNewThread],
  );

  // Handle thread pin toggle
  const handleTogglePin = useCallback(
    (threadId: string, isPinned: boolean) => {
      togglePin.mutate({ id: threadId, isPinned: !isPinned });
    },
    [togglePin],
  );

  return (
    <Page className={cn("h-full", className)}>
      <PageHeader
        heading="AI Assistant"
        actions={
          <Badge variant="secondary" className="gap-1">
            <Icons.Sparkles className="size-3" />
            Beta
          </Badge>
        }
      />
      <PageContent className="p-0">
        <div className="flex h-full">
          {/* Desktop Sidebar */}
          <aside
            className={cn(
              "hidden md:flex border-r flex-col transition-all duration-200",
              sidebarCollapsed ? "w-0 overflow-hidden opacity-0" : "w-64 opacity-100",
            )}
          >
            <div className="flex flex-col h-full">
              <div className="p-3 border-b flex items-center justify-between">
                <span className="font-semibold text-sm">Conversations</span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
                >
                  <Icons.ArrowLeft className="size-4" />
                </Button>
              </div>
              <ScrollArea className="flex-1">
                <ThreadList
                  threads={threads}
                  selectedThreadId={selectedThreadId}
                  isLoading={isLoading}
                  onSelectThread={handleSelectThread}
                  onNewThread={handleNewThread}
                  onDeleteThread={handleDeleteThread}
                  onTogglePin={handleTogglePin}
                  searchValue={searchValue}
                  onSearchChange={setSearchValue}
                  hasNextPage={hasNextPage}
                  onLoadMore={() => fetchNextPage()}
                />
              </ScrollArea>
            </div>
          </aside>

          {/* Mobile Sidebar */}
          <Sheet open={sidebarOpen} onOpenChange={setSidebarOpen}>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="md:hidden absolute left-4 top-4 z-10"
              >
                <Icons.Menu className="size-4" />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-80 p-0">
              <SheetHeader className="p-4 border-b space-y-0">
                <SheetTitle>Conversations</SheetTitle>
              </SheetHeader>
              <ScrollArea className="flex-1 h-[calc(100vh-120px)]">
                <ThreadList
                  threads={threads}
                  selectedThreadId={selectedThreadId}
                  isLoading={isLoading}
                  onSelectThread={handleSelectThread}
                  onNewThread={handleNewThread}
                  onDeleteThread={handleDeleteThread}
                  onTogglePin={handleTogglePin}
                  searchValue={searchValue}
                  onSearchChange={setSearchValue}
                  hasNextPage={hasNextPage}
                  onLoadMore={() => fetchNextPage()}
                />
              </ScrollArea>
            </SheetContent>
          </Sheet>

          {/* Desktop sidebar toggle button (when collapsed) */}
          {sidebarCollapsed && (
            <Button
              variant="ghost"
              size="icon"
              className="hidden md:flex absolute left-0 top-1/2 -translate-y-1/2 z-10 rounded-r-none rounded-l-md"
              onClick={() => setSidebarCollapsed(false)}
            >
              <Icons.ArrowRight className="size-4" />
            </Button>
          )}

          {/* Main Chat Area */}
          <div className="flex-1 flex flex-col min-w-0">
            <ChatContent
              messages={localMessages}
              selectedThreadId={selectedThreadId}
              onNewThread={handleNewThread}
              onMessagesChange={setLocalMessages}
            />
          </div>
        </div>
      </PageContent>
    </Page>
  );
}

interface ChatContentProps {
  messages: ExternalMessage[];
  selectedThreadId: string | null;
  onNewThread: () => void;
  onMessagesChange: (messages: ExternalMessage[]) => void;
}

function ChatContent({ messages, selectedThreadId, onMessagesChange }: ChatContentProps) {
  const { sendMessage, cancelStream, streamState } = useAiChat({
    onThreadIdCreated: () => {
      // Thread was created, will be handled by parent
    },
    onStreamComplete: () => {
      // Stream complete
    },
  });

  const handleSendMessage = useCallback(
    async (content: string) => {
      // Add user message
      const userMessage: ExternalMessage = {
        id: generateId(),
        role: "user",
        parts: [{ type: "text", content }],
        createdAt: new Date(),
      };
      onMessagesChange([...messages, userMessage]);

      // Create placeholder assistant message
      const assistantMessage: ExternalMessage = {
        id: generateId(),
        role: "assistant",
        parts: [],
        createdAt: new Date(),
      };
      onMessagesChange([...messages, userMessage, assistantMessage]);

      // Send message and stream response
      await sendMessage(content, selectedThreadId ?? undefined);
    },
    [messages, onMessagesChange, sendMessage, selectedThreadId],
  );

  return (
    <div className="flex flex-col h-full">
      {/* Messages */}
      <ScrollArea className="flex-1">
        <div className="max-w-3xl mx-auto px-4 py-6">
          {messages.length === 0 ? (
            <WelcomeView onPromptClick={handleSendMessage} />
          ) : (
            <MessageList messages={messages} streamState={streamState} />
          )}
        </div>
      </ScrollArea>

      {/* Input */}
      <div className="border-t p-4">
        <MessageInput
          onSend={handleSendMessage}
          onCancel={streamState.isStreaming ? cancelStream : undefined}
          isStreaming={streamState.isStreaming}
        />
        <p className="text-muted-foreground text-xs text-center mt-2">
          Responses may be inaccurate. Not financial advice.
        </p>
      </div>
    </div>
  );
}

interface WelcomeViewProps {
  onPromptClick: (prompt: string) => void;
}

function WelcomeView({ onPromptClick }: WelcomeViewProps) {
  const suggestions = [
    {
      icon: "TrendingUp",
      text: "How is my portfolio performing this year?",
    },
    {
      icon: "BarChart",
      text: "What are my top performing holdings?",
    },
    {
      icon: "FileText",
      text: "Show my dividend income summary",
    },
    {
      icon: "PieChart",
      text: "Analyze my asset allocation",
    },
  ];

  return (
    <div className="flex flex-col items-center justify-center h-full py-12">
      <div className="text-center space-y-2 mb-8">
        <Icons.Sparkles className="size-12 mx-auto text-primary mb-4" />
        <h2 className="text-2xl font-semibold">Hello there!</h2>
        <p className="text-muted-foreground">
          How can I help you today?
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 w-full max-w-2xl">
        {suggestions.map((suggestion, index) => {
          const Icon = Icons[suggestion.icon as keyof typeof Icons];
          // Skip rendering if icon doesn't exist to prevent errors
          if (!Icon) return null;

          return (
            <Button
              key={`suggestion-${index}`}
              variant="outline"
              className="justify-start text-left h-auto py-3 px-4"
              onClick={() => onPromptClick(suggestion.text)}
            >
              <Icon className="size-4 shrink-0 opacity-60 mr-2" />
              <span className="text-sm">{suggestion.text}</span>
            </Button>
          );
        })}
      </div>
    </div>
  );
}
