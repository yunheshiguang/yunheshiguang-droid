import { useQuery } from "@connectrpc/connect-query";
import { useIsMutating, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import { errorMessage } from "@/shared/lib/error-message";
import {
  type ConnectionPlan,
  type DeviceGroup,
  type DeviceRecord,
  type DiagnosticSignal,
  WorkspaceService,
  type WorkspaceSnapshot,
} from "@/shared/proto";

const NO_ITEMS: readonly never[] = [];

const SNAPSHOT_STALE_TIME_MS = 10_000;

export const SNAPSHOT_POLL_INTERVAL_MS = 30_000;

const getWorkspaceSnapshot = WorkspaceService.method.getWorkspaceSnapshot;

const snapshotOptions = {
  staleTime: SNAPSHOT_STALE_TIME_MS,
  refetchInterval: SNAPSHOT_POLL_INTERVAL_MS,
};

export function useWorkspaceSnapshot<T>(select: (snapshot: WorkspaceSnapshot) => T): T | undefined {
  return useQuery(getWorkspaceSnapshot, {}, { ...snapshotOptions, select }).data;
}

const selectDevices = (snapshot: WorkspaceSnapshot) => snapshot.devices;
const selectGroups = (snapshot: WorkspaceSnapshot) => snapshot.groups;
const selectDiagnostics = (snapshot: WorkspaceSnapshot) => snapshot.diagnostics;
const selectConnectionPlans = (snapshot: WorkspaceSnapshot) => snapshot.connectionPlans;

export function useWorkspaceDevices(): readonly DeviceRecord[] {
  return useWorkspaceSnapshot(selectDevices) ?? NO_ITEMS;
}

export function useWorkspaceGroups(): readonly DeviceGroup[] {
  return useWorkspaceSnapshot(selectGroups) ?? NO_ITEMS;
}

export function useWorkspaceDiagnostics(): readonly DiagnosticSignal[] {
  return useWorkspaceSnapshot(selectDiagnostics) ?? NO_ITEMS;
}

export function useWorkspaceConnectionPlans(): readonly ConnectionPlan[] {
  return useWorkspaceSnapshot(selectConnectionPlans) ?? NO_ITEMS;
}

export type WorkspaceSnapshotStatus = {
  loading: boolean;
  refreshing: boolean;
  error: string | null;
  refresh: () => Promise<void>;
};

export function useWorkspaceSnapshotStatus(): WorkspaceSnapshotStatus {
  const queryClient = useQueryClient();
  const { isPending, error } = useQuery(getWorkspaceSnapshot, {}, snapshotOptions);
  const refreshKey = workspaceMutationKeys.refreshSnapshot();
  const { mutateAsync } = useMutation({
    mutationKey: refreshKey,
    mutationFn: () => queryClient.refetchQueries({ queryKey: snapshotKey() }),
  });
  const refreshing = useIsMutating({ mutationKey: refreshKey }) > 0;
  const refresh = useCallback(async () => {
    await mutateAsync();
  }, [mutateAsync]);

  return {
    loading: isPending,
    refreshing,
    error: error == null ? null : errorMessage(error),
    refresh,
  };
}
