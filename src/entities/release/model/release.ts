import { useQuery } from "@connectrpc/connect-query";

import { windowGateway } from "@/shared/api";
import {
  type AvailableUpdate,
  UpdateService,
  type UpdateState,
  UpdateStatus,
} from "@/shared/proto";

export function shortCommit(commit: string): string {
  return commit.slice(0, 7);
}

export function openReleaseLink(url: string): void {
  void windowGateway().openExternal(url);
}

export function availableUpdate(state: UpdateState | undefined): AvailableUpdate | undefined {
  return state?.status === UpdateStatus.AVAILABLE ? state.update : undefined;
}

export function useUpdateState(): UpdateState | undefined {
  return useQuery(UpdateService.method.getUpdateState, {}).data;
}
