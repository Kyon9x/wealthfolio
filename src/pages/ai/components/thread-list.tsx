/**
 * ThreadList - Sidebar component showing list of AI chat threads.
 */

import { memo, useEffect, useRef, useState } from "react";

import {
  Icons,
  Button,
  Input,
  Skeleton,
} from "@/ui";
import { cn } from "@/lib/utils";
import type { AiThread } from "@/adapters/ai";

interface ThreadListProps {
  threads: AiThread[];
  selectedThreadId: string | null;
  isLoading: boolean;
  onSelectThread: (threadId: string | null) => void;
  onNewThread: () => void;
  onDeleteThread: (threadId: string) => void;
  onTogglePin: (threadId: string, isPinned: boolean) => void;
  searchValue: string;
  onSearchChange: (value: string) => void;
  hasNextPage?: boolean;
  onLoadMore?: () => void;
}

export function ThreadList({
  threads,
  selectedThreadId,
  isLoading,
  onSelectThread,
  onNewThread,
  onDeleteThread,
  onTogglePin,
  searchValue,
  onSearchChange,
  hasNextPage,
  onLoadMore,
}: ThreadListProps) {
  // Separate pinned and unpinned threads
  const pinnedThreads = threads.filter((t) => t.isPinned);
  const unpinnedThreads = threads.filter((t) => !t.isPinned);

  // Infinite scroll trigger
  const loadMoreRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!hasNextPage || !onLoadMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          onLoadMore();
        }
      },
      { rootMargin: "100px" },
    );

    const element = loadMoreRef.current;
    if (element) {
      observer.observe(element);
    }

    return () => observer.disconnect();
  }, [hasNextPage, onLoadMore]);

  return (
    <div className="flex flex-col h-full">
      {/* New Thread Button */}
      <Button
        variant="ghost"
        className="m-2 justify-start"
        onClick={onNewThread}
      >
        <Icons.Plus className="size-4 mr-2" />
        New Thread
      </Button>

      {/* Search Input */}
      <div className="px-2 mb-2">
        <div className="relative">
          <Icons.Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search threads..."
            value={searchValue}
            onChange={(e) => onSearchChange(e.target.value)}
            className="pl-9 h-8 text-sm"
          />
          {searchValue && (
            <button
              type="button"
              onClick={() => onSearchChange("")}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              aria-label="Clear search"
            >
              <Icons.Close className="size-4" />
            </button>
          )}
        </div>
      </div>

      {/* Thread List */}
      <div className="flex-1 overflow-y-auto px-2">
        {isLoading ? (
          <ThreadListSkeleton />
        ) : threads.length === 0 ? (
          <EmptyState search={searchValue} />
        ) : (
          <>
            {pinnedThreads.length > 0 && (
              <ThreadGroup
                title="Pinned"
                threads={pinnedThreads}
                selectedThreadId={selectedThreadId}
                onSelectThread={onSelectThread}
                onDeleteThread={onDeleteThread}
                onTogglePin={onTogglePin}
              />
            )}
            {unpinnedThreads.length > 0 && (
              <ThreadGroup
                title={pinnedThreads.length > 0 ? "Recent" : undefined}
                threads={unpinnedThreads}
                selectedThreadId={selectedThreadId}
                onSelectThread={onSelectThread}
                onDeleteThread={onDeleteThread}
                onTogglePin={onTogglePin}
              />
            )}
            <div ref={loadMoreRef} className="h-1" />
          </>
        )}
      </div>
    </div>
  );
}

interface ThreadGroupProps {
  title?: string;
  threads: AiThread[];
  selectedThreadId: string | null;
  onSelectThread: (threadId: string) => void;
  onDeleteThread: (threadId: string) => void;
  onTogglePin: (threadId: string, isPinned: boolean) => void;
}

function ThreadGroup({
  title,
  threads,
  selectedThreadId,
  onSelectThread,
  onDeleteThread,
  onTogglePin,
}: ThreadGroupProps) {
  return (
    <div className="mb-4">
      {title && (
        <div className="flex items-center gap-1.5 px-2 py-1 text-xs font-medium text-muted-foreground mb-1">
          <Icons.Star className="size-3" />
          {title}
        </div>
      )}
      <div className="space-y-0.5">
        {threads.map((thread) => (
          <ThreadListItem
            key={thread.id}
            thread={thread}
            isSelected={selectedThreadId === thread.id}
            onSelect={() => onSelectThread(thread.id)}
            onDelete={() => onDeleteThread(thread.id)}
            onTogglePin={() => onTogglePin(thread.id, thread.isPinned)}
          />
        ))}
      </div>
    </div>
  );
}

interface ThreadListItemProps {
  thread: AiThread;
  isSelected: boolean;
  onSelect: () => void;
  onDelete: () => void;
  onTogglePin: () => void;
}

const ThreadListItem = memo(function ThreadListItem({
  thread,
  isSelected,
  onSelect,
  onDelete,
  onTogglePin,
}: ThreadListItemProps) {
  const [showActions, setShowActions] = useState(false);

  return (
    <div
      className={cn(
        "group relative rounded-lg transition-colors",
        isSelected ? "bg-muted" : "hover:bg-muted/50",
      )}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      <button
        type="button"
        className="w-full px-3 py-2 text-left"
        onClick={onSelect}
      >
        <span className="block text-sm font-medium truncate">
          {thread.title || "New Chat"}
        </span>
        <span className="block text-xs text-muted-foreground truncate">
          {new Date(thread.updatedAt).toLocaleDateString()}
        </span>
      </button>

      {showActions && (
        <div className="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5 bg-background rounded-md border shadow-sm">
          <button
            type="button"
            className="h-6 w-6 p-0 flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              onTogglePin();
            }}
            title={thread.isPinned ? "Unpin" : "Pin"}
          >
            <Icons.Star className={cn("size-3.5", thread.isPinned && "fill-current")} />
          </button>

          <button
            type="button"
            className="h-6 w-6 p-0 flex items-center justify-center text-destructive hover:text-destructive/80 transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              onDelete();
            }}
            title="Delete"
          >
            <Icons.Trash className="size-3.5" />
          </button>
        </div>
      )}
    </div>
  );
});

function ThreadListSkeleton() {
  return (
    <>
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="p-3 space-y-2">
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-3 w-1/2" />
        </div>
      ))}
    </>
  );
}

interface EmptyStateProps {
  search: string;
}

function EmptyState({ search }: EmptyStateProps) {
  return (
    <div className="text-center py-8 px-4">
      <Icons.HelpCircle className="size-8 mx-auto text-muted-foreground/50 mb-3" />
      <p className="text-sm text-muted-foreground">
        {search
          ? "No threads match your search."
          : "No conversations yet."}
      </p>
    </div>
  );
}
