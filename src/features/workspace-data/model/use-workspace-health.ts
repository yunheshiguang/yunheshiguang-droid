import { useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import { TelemetryService, type WorkspaceHealth } from "@/shared/proto";

const HEALTH_POLL_INTERVAL_MS = 5_000;

export type WorkspaceHealthStatus = {
  health?: WorkspaceHealth;
  loading: boolean;
  error: string | null;
};

export function useWorkspaceHealth(): WorkspaceHealthStatus {
  const { data, isPending, error } = useQuery(
    TelemetryService.method.getWorkspaceHealth,
    {},
    { refetchInterval: HEALTH_POLL_INTERVAL_MS },
  );

  return { health: data, loading: isPending, error: error == null ? null : errorMessage(error) };
}
