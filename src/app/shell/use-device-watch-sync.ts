import { useQueryClient } from "@tanstack/react-query";

import { discoveryKey, snapshotKey, WORKSPACE_EVENTS } from "@/shared/api";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useDeviceWatchSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.devicesChanged, () => {
    for (const queryKey of [snapshotKey(), discoveryKey()]) {
      void queryClient.invalidateQueries({ queryKey });
    }
  });
}
