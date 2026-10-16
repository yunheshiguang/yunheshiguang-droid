import { create } from "@bufbuild/protobuf";
import { callUnaryMethod, useQuery, useTransport } from "@connectrpc/connect-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { toTaskRun, useTaskHistoryActions } from "@/entities/task";
import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import {
  BatchActionKind,
  BatchActionRequestSchema,
  type BatchActionResult,
  BatchService,
  type ScriptEngineInfo,
  ScriptService,
} from "@/shared/proto";

const NO_ENGINES: readonly ScriptEngineInfo[] = [];

export function useScriptEngines(): readonly ScriptEngineInfo[] {
  const { data } = useQuery(
    ScriptService.method.listScriptEngines,
    {},
    { staleTime: Number.POSITIVE_INFINITY },
  );

  return data?.engines ?? NO_ENGINES;
}

export type ScriptRunner = {
  run: (scriptId: string, deviceIds: readonly string[]) => Promise<BatchActionResult>;
  running: boolean;
  result: BatchActionResult | undefined;
  error: unknown;
};

function scriptRequest(scriptId: string, deviceIds: readonly string[]) {
  return create(BatchActionRequestSchema, {
    kind: BatchActionKind.RUN_SCRIPT,
    deviceIds: [...deviceIds],
    scriptId,
  });
}

export function useRunScript(): ScriptRunner {
  const { record } = useTaskHistoryActions();
  const transport = useTransport();
  const queryClient = useQueryClient();
  const { mutateAsync, isPending, data, error } = useMutation({
    mutationKey: workspaceMutationKeys.batchAction(),
    mutationFn: ({ scriptId, deviceIds }: { scriptId: string; deviceIds: readonly string[] }) =>
      callUnaryMethod(
        transport,
        BatchService.method.runBatchAction,
        scriptRequest(scriptId, deviceIds),
      ),
    onSuccess: (result, { scriptId, deviceIds }) => {
      const run = toTaskRun(result, scriptRequest(scriptId, deviceIds));

      if (run) {
        record(run);
      }
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKey() });
    },
  });

  return {
    run: (scriptId, deviceIds) => mutateAsync({ scriptId, deviceIds }),
    running: isPending,
    result: data,
    error,
  };
}
