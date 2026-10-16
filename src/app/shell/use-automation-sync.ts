import { createConnectQueryKey } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { WORKSPACE_EVENTS } from "@/shared/api";
import { AutomationService } from "@/shared/proto";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useAutomationSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.automationChanged, () => {
    void queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({ schema: AutomationService, cardinality: undefined }),
    });
  });
}
