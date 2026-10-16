import type { MessageInitShape } from "@bufbuild/protobuf";
import { createConnectQueryKey, useMutation, useTransport } from "@connectrpc/connect-query";
import { useIsMutating, useQueryClient } from "@tanstack/react-query";

import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import {
  type AndroidPlatformToolsConfigSchema,
  type AndroidPlatformToolsState,
  PlatformToolsService,
  WorkspaceService,
  type WorkspaceSnapshot,
} from "@/shared/proto";

export type PlatformToolsConfig = MessageInitShape<typeof AndroidPlatformToolsConfigSchema>;

export type PlatformToolsCommands = {
  apply: (config: PlatformToolsConfig) => Promise<AndroidPlatformToolsState>;
  install: (licenseAccepted: boolean) => Promise<AndroidPlatformToolsState>;
  applying: boolean;
  installing: boolean;
};

export function usePlatformToolsCommands(): PlatformToolsCommands {
  const queryClient = useQueryClient();
  const transport = useTransport();
  const onSuccess = (state: AndroidPlatformToolsState) => {
    queryClient.setQueryData<WorkspaceSnapshot>(
      createConnectQueryKey({
        schema: WorkspaceService.method.getWorkspaceSnapshot,
        input: {},
        transport,
        cardinality: "finite",
      }),
      (snapshot) => (snapshot ? { ...snapshot, androidPlatformTools: state } : snapshot),
    );
    void queryClient.invalidateQueries({ queryKey: snapshotKey() });
  };
  const applyKey = workspaceMutationKeys.updatePlatformTools();
  const installKey = workspaceMutationKeys.installPlatformTools();
  const apply = useMutation(PlatformToolsService.method.updateAndroidPlatformToolsConfig, {
    mutationKey: applyKey,
    onSuccess,
  });
  const install = useMutation(PlatformToolsService.method.installAndroidPlatformTools, {
    mutationKey: installKey,
    onSuccess,
  });
  const applying = useIsMutating({ mutationKey: applyKey }) > 0;
  const installing = useIsMutating({ mutationKey: installKey }) > 0;

  return {
    apply: (config) => apply.mutateAsync({ config }),
    install: (licenseAccepted) => install.mutateAsync({ licenseAccepted }),
    applying,
    installing,
  };
}
