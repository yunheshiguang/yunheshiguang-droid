import { create } from "@bufbuild/protobuf";
import { useMutation } from "@connectrpc/connect-query";
import { useMutationState, useQueryClient } from "@tanstack/react-query";

import { toTaskRun, useLatestTaskRun, useTaskHistoryActions } from "@/entities/task";
import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import { type BatchActionRequest, BatchActionRequestSchema, BatchService } from "@/shared/proto";

import { type BatchAttempt, type BatchRunView, resolveBatchRunView } from "./batch-run-view";

export type BatchActionRunner = {
  run: (request: BatchActionRequest) => void;
  running: boolean;
  view: BatchRunView;
};

export function useRunBatchAction(): BatchActionRunner {
  const queryClient = useQueryClient();
  const { record } = useTaskHistoryActions();
  const latestRun = useLatestTaskRun();
  const mutationKey = workspaceMutationKeys.batchAction();
  const { mutate } = useMutation(BatchService.method.runBatchAction, {
    mutationKey,
    onSuccess: (result, request) => {
      const run = toTaskRun(result, create(BatchActionRequestSchema, request));

      if (run) {
        record(run);
      }
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKey() });
    },
  });
  const attempts = useMutationState({
    filters: { mutationKey },
    select: (mutation): BatchAttempt => ({
      status: mutation.state.status,
      error: mutation.state.error,
    }),
  });

  return {
    run: mutate,
    running: attempts.some((attempt) => attempt.status === "pending"),
    view: resolveBatchRunView(attempts.at(-1), latestRun),
  };
}
