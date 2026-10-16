import { createConnectQueryKey, useMutation, useTransport } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { useUpdateState } from "@/entities/release";
import { workspaceMutationKeys } from "@/shared/api";
import { UpdateService, type UpdateSettingsResult, type UpdateState } from "@/shared/proto";

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  const transport = useTransport();
  const state = useUpdateState();
  const store = (next: UpdateState | undefined) => {
    if (next) {
      queryClient.setQueryData(
        createConnectQueryKey({
          schema: UpdateService.method.getUpdateState,
          input: {},
          transport,
          cardinality: "finite",
        }),
        next,
      );
    }
  };
  const check = useMutation(UpdateService.method.checkForUpdates, {
    mutationKey: workspaceMutationKeys.checkForUpdates(),
    onSuccess: store,
  });
  const automatic = useMutation(UpdateService.method.setAutomaticChecks, {
    mutationKey: workspaceMutationKeys.updateSettings(),
    onSuccess: (result: UpdateSettingsResult) => store(result.state),
  });

  return {
    state,
    check: () => check.mutateAsync({}),
    checking: check.isPending,
    setAutomatic: (checkAutomatically: boolean) => automatic.mutateAsync({ checkAutomatically }),
    saving: automatic.isPending,
  };
}
