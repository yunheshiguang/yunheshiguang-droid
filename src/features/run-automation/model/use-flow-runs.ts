import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import type { RunOptionValues, RunTargetInit } from "@/entities/flow-run";
import { workspaceMutationKeys } from "@/shared/api";
import { AutomationService, type FlowRun, type FlowRunResult, RunStatus } from "@/shared/proto";

const RUNNING_POLL_INTERVAL_MS = 2_000;

const NO_RUNS: readonly FlowRun[] = [];

const listFlowRuns = AutomationService.method.listFlowRuns;

export function useFlowRuns(): readonly FlowRun[] {
  const { data } = useQuery(
    listFlowRuns,
    {},
    {
      refetchInterval: (query) =>
        query.state.data?.runs.some((run) => run.status === RunStatus.RUNNING)
          ? RUNNING_POLL_INTERVAL_MS
          : false,
    },
  );

  return data?.runs ?? NO_RUNS;
}

export type FlowRunActions = {
  start: (
    flowId: string,
    target: RunTargetInit,
    options: RunOptionValues,
  ) => Promise<FlowRunResult>;
  stop: (runId: string) => Promise<FlowRunResult>;
  starting: boolean;
};

export function useFlowRunActions(): FlowRunActions {
  const queryClient = useQueryClient();
  const options = {
    mutationKey: workspaceMutationKeys.flowRun(),
    onSettled: () =>
      queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: listFlowRuns, cardinality: "finite" }),
      }),
  };
  const start = useMutation(AutomationService.method.startFlowRun, options);
  const stop = useMutation(AutomationService.method.stopFlowRun, options);

  return {
    start: (flowId, target, runOptions) =>
      start.mutateAsync({ flowId, target, options: runOptions }),
    stop: (runId) => stop.mutateAsync({ runId }),
    starting: start.isPending,
  };
}
