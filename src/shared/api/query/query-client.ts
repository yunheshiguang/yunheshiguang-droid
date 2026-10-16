import { createConnectQueryKey } from "@connectrpc/connect-query";
import { MutationCache, QueryClient } from "@tanstack/react-query";

import { AuditService, GovernanceService } from "@/shared/proto";

declare module "@tanstack/react-query" {
  interface Register {
    defaultError: unknown;
  }
}

export function createWorkspaceQueryClient(): QueryClient {
  const queryClient: QueryClient = new QueryClient({
    mutationCache: new MutationCache({
      onSettled: () => {
        for (const schema of [GovernanceService, AuditService]) {
          void queryClient.invalidateQueries({
            queryKey: createConnectQueryKey({ schema, cardinality: undefined }),
          });
        }
      },
    }),
    defaultOptions: {
      queries: { networkMode: "always", retry: 1 },
      mutations: { networkMode: "always", retry: false },
    },
  });

  return queryClient;
}
