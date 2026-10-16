import type { DescMessage, DescMethodUnary, MessageInitShape } from "@bufbuild/protobuf";
import { useMutation } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import {
  type ConnectionResult,
  type ConnectionResultSchema,
  ConnectionService,
} from "@/shared/proto";

export type ConnectionCommand<TRequest> = {
  run: (request: TRequest) => void;
  pending: boolean;
  result?: ConnectionResult;
  error: unknown;
};

function useConnectionCommand<I extends DescMessage>(
  mutationKey: readonly unknown[],
  method: DescMethodUnary<I, typeof ConnectionResultSchema>,
): ConnectionCommand<MessageInitShape<I>> {
  const queryClient = useQueryClient();
  const { mutate, isPending, data, error } = useMutation(method, {
    mutationKey,
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKey() });
    },
  });

  return { run: mutate, pending: isPending, result: data, error };
}

export function useConnectDevice() {
  return useConnectionCommand(
    workspaceMutationKeys.connectDevice(),
    ConnectionService.method.connectDevice,
  );
}

export function usePairDevice() {
  return useConnectionCommand(
    workspaceMutationKeys.pairDevice(),
    ConnectionService.method.pairDevice,
  );
}

export function useDisconnectDevice() {
  return useConnectionCommand(
    workspaceMutationKeys.disconnectDevice(),
    ConnectionService.method.disconnectDevice,
  );
}
