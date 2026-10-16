import { useQueryClient } from "@tanstack/react-query";

import { snapshotKey, WORKSPACE_EVENTS } from "@/shared/api";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useDeviceProfileSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.deviceProfilesChanged, () => {
    void queryClient.invalidateQueries({ queryKey: snapshotKey() });
  });
}
