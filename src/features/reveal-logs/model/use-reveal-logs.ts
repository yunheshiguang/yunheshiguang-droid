import { useMutation } from "@tanstack/react-query";

import { revealLogDirectory, workspaceMutationKeys } from "@/shared/api";

export type RevealLogs = {
  reveal: () => void;
  pending: boolean;
  error: unknown;
};

export function useRevealLogs(): RevealLogs {
  const mutation = useMutation({
    mutationKey: workspaceMutationKeys.revealLogs(),
    mutationFn: revealLogDirectory,
  });

  return {
    reveal: () => mutation.mutate(),
    pending: mutation.isPending,
    error: mutation.error,
  };
}
