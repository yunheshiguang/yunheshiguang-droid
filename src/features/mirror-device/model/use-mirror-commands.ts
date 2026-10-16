import { createConnectQueryKey, useMutation, useTransport } from "@connectrpc/connect-query";
import { useIsMutating, useQueryClient } from "@tanstack/react-query";

import { workspaceMutationKeys } from "@/shared/api";
import { type ScrcpyConfig, ScrcpyService, type ScrcpyState } from "@/shared/proto";

export type MirrorCommands = {
  launch: (serial: string) => void;
  stop: (sessionId: string) => void;
  apply: (config: ScrcpyConfig) => Promise<ScrcpyState>;
  install: () => void;
  busy: boolean;
  installing: boolean;
};

export function useMirrorCommands(): MirrorCommands {
  const queryClient = useQueryClient();
  const transport = useTransport();
  const onSuccess = (state: ScrcpyState) =>
    queryClient.setQueryData(
      createConnectQueryKey({
        schema: ScrcpyService.method.getScrcpyState,
        input: {},
        transport,
        cardinality: "finite",
      }),
      state,
    );
  const mutationKey = workspaceMutationKeys.scrcpy();
  const installKey = workspaceMutationKeys.installScrcpy();

  const launch = useMutation(ScrcpyService.method.launchScrcpySession, { mutationKey, onSuccess });
  const stop = useMutation(ScrcpyService.method.stopScrcpySession, { mutationKey, onSuccess });
  const apply = useMutation(ScrcpyService.method.updateScrcpyConfig, { mutationKey, onSuccess });
  const install = useMutation(ScrcpyService.method.installScrcpy, {
    mutationKey: installKey,
    onSuccess,
  });

  return {
    launch: (serial) => launch.mutate({ serial }),
    stop: (sessionId) => stop.mutate({ sessionId }),
    apply: (config) => apply.mutateAsync({ config }),
    install: () => install.mutate({}),
    busy: useIsMutating({ mutationKey }) > 0,
    installing: useIsMutating({ mutationKey: installKey }) > 0,
  };
}
