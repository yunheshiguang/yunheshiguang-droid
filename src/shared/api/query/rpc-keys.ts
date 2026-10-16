import { createConnectQueryKey } from "@connectrpc/connect-query";

import { DiscoveryService, WorkspaceService } from "@/shared/proto";

export function snapshotKey() {
  return createConnectQueryKey({
    schema: WorkspaceService.method.getWorkspaceSnapshot,
    cardinality: "finite",
  });
}

export function discoveryKey() {
  return createConnectQueryKey({ schema: DiscoveryService, cardinality: undefined });
}
