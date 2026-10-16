import { useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import { ScrcpyService, ScrcpySessionStatus, type ScrcpyState } from "@/shared/proto";

const SCRCPY_SESSION_POLL_INTERVAL_MS = 3_000;

export type ScrcpyStateStatus = {
  state?: ScrcpyState;
  loading: boolean;
  error: string | null;
};

export function useScrcpyState(): ScrcpyStateStatus {
  const { data, isPending, error } = useQuery(
    ScrcpyService.method.getScrcpyState,
    {},
    {
      refetchInterval: (query) =>
        query.state.data?.sessions.some((session) => session.status === ScrcpySessionStatus.RUNNING)
          ? SCRCPY_SESSION_POLL_INTERVAL_MS
          : false,
    },
  );

  return { state: data, loading: isPending, error: error == null ? null : errorMessage(error) };
}
