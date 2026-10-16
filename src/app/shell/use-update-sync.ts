import { createConnectQueryKey } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { WORKSPACE_EVENTS } from "@/shared/api";
import { UpdateService } from "@/shared/proto";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useUpdateSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.updateAvailable, () => {
    void queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({ schema: UpdateService, cardinality: undefined }),
    });
  });
}
