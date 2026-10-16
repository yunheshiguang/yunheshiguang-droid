import { createConnectQueryKey } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { discoveryKey, snapshotKey, WORKSPACE_EVENTS } from "@/shared/api";
import { CloudService } from "@/shared/proto";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useCloudPhonesSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.cloudPhonesChanged, () => {
    for (const queryKey of [
      createConnectQueryKey({ schema: CloudService, cardinality: undefined }),
      discoveryKey(),
      snapshotKey(),
    ]) {
      void queryClient.invalidateQueries({ queryKey });
    }
  });
}
