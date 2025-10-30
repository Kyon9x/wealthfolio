import { logger } from '@/adapters';
import { importActivities } from '@/commands/activity-import';
import { toast } from '@/components/ui/use-toast';
import { QueryKeys } from '@/lib/query-keys';
import { useMutation, useQueryClient } from '@tanstack/react-query';

export function useActivityImportMutations({
  onSuccess,
  onError,
}: {
  onSuccess?: (activities: unknown[]) => void;
  onError?: (error: string) => void;
} = {}) {
  const queryClient = useQueryClient();

  const confirmImportMutation = useMutation({
    mutationFn: importActivities,
    onSuccess: async (result: any) => {
      // Invalidate activity data cache to trigger refetch
      await queryClient.invalidateQueries({
        queryKey: [QueryKeys.ACTIVITY_DATA],
      });

      // Call the provided onSuccess callback if it exists
      if (onSuccess) {
        // Ensure we pass an array of activities to the callback
        const activities = Array.isArray(result) ? result : [result];
        onSuccess(activities);
        toast({
          title: 'Import successful',
          description: 'Activities have been imported successfully.',
        });
      }
    },
    onError: (error: unknown) => {
      logger.error(`Error confirming import: ${String(error)}`);

      // Call the provided onError callback if it exists
      if (onError) {
        const errMsg =
          error && typeof error === 'object' && 'message' in error
            ? String((error as { message?: unknown }).message)
            : 'An error occurred during import';
        onError(errMsg);
      } else {
        const errorMessage =
          error && typeof error === "object" && "message" in error
            ? String((error as { message?: unknown }).message)
            : "An error occurred during import";

        toast({
          title: "Import failed",
          description:
            errorMessage || "Please try again or report an issue if the problem persists.",
          variant: "destructive",
        });
      }
    },
  });

  return {
    confirmImportMutation,
  };
}
