/**
 * useAiThreads hook for managing AI chat threads.
 * Provides thread listing, creation, deletion, and pin management.
 */

import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  listAiThreads,
  getAiThread,
  getAiThreadMessages,
  updateAiThread,
  deleteAiThread,
  type ThreadPage,
  type UpdateThreadRequest,
} from "@/adapters/ai";
import type { ChatMessage, AiThread } from "@/adapters/ai";

/** Query key for AI threads list (infinite query) */
export const AI_THREADS_KEY = ["ai_threads"] as const;

/** Default page size for thread pagination */
const DEFAULT_THREADS_LIMIT = 20;

/**
 * Hook to fetch chat threads with infinite pagination and optional search.
 * Threads are sorted by pinned status (pinned first), then by updated_at.
 */
export function useAiThreads(search?: string) {
  const normalizedSearch = search?.trim() || undefined;

  return useInfiniteQuery<ThreadPage, Error>({
    queryKey: normalizedSearch ? [...AI_THREADS_KEY, "search", normalizedSearch] : AI_THREADS_KEY,
    queryFn: ({ pageParam }) =>
      listAiThreads({
        cursor: pageParam as string | undefined,
        limit: DEFAULT_THREADS_LIMIT,
        search: normalizedSearch,
      }),
    getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
    initialPageParam: undefined as string | undefined,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    staleTime: 30000, // 30 seconds before considered stale
  });
}

/**
 * Hook to fetch a single thread by ID.
 */
export function useAiThread(threadId: string | null) {
  return useQuery<AiThread | null, Error>({
    queryKey: ["ai_thread", threadId ?? ""],
    queryFn: () => (threadId ? getAiThread(threadId) : Promise.resolve(null)),
    enabled: !!threadId,
  });
}

/**
 * Hook to fetch messages for a thread.
 */
export function useAiThreadMessages(threadId: string | null) {
  return useQuery<ChatMessage[], Error>({
    queryKey: ["ai_thread_messages", threadId ?? ""],
    queryFn: () => (threadId ? getAiThreadMessages(threadId) : Promise.resolve([])),
    enabled: !!threadId,
  });
}

/**
 * Hook to update a thread's title.
 */
export function useUpdateAiThread() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: UpdateThreadRequest) => updateAiThread(request),
    onSuccess: (updatedThread) => {
      queryClient.invalidateQueries({ queryKey: AI_THREADS_KEY });
      queryClient.setQueryData(["ai_thread", updatedThread.id], updatedThread);
    },
  });
}

/**
 * Hook to toggle a thread's pinned status.
 */
export function useToggleAiThreadPin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, isPinned }: { id: string; isPinned: boolean }) =>
      updateAiThread({ id, isPinned }),
    onSuccess: (updatedThread) => {
      queryClient.invalidateQueries({ queryKey: AI_THREADS_KEY });
      queryClient.setQueryData(["ai_thread", updatedThread.id], updatedThread);
    },
  });
}

/**
 * Hook to delete a thread.
 */
export function useDeleteAiThread() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (threadId: string) => deleteAiThread(threadId),
    onSuccess: (_, threadId) => {
      queryClient.invalidateQueries({ queryKey: AI_THREADS_KEY });
      queryClient.removeQueries({ queryKey: ["ai_thread", threadId] });
    },
  });
}

/**
 * Flatten paginated thread data into a single array.
 * Utility for components that need a flat list of threads.
 */
export function flattenThreadPages(pages: ThreadPage[] | undefined): AiThread[] {
  return pages?.flatMap((page) => page.threads) ?? [];
}
