import { create } from "@bufbuild/protobuf";
import { createConnectQueryKey } from "@connectrpc/connect-query";
import { MutationObserver } from "@tanstack/react-query";
import { describe, expect, it } from "vitest";

import { GovernanceService, GovernanceStateSchema, WorkspaceSnapshotSchema } from "@/shared/proto";

import { workspaceMutationKeys } from "./keys";
import { snapshotKey } from "./rpc-keys";

const governance = createConnectQueryKey({
  schema: GovernanceService.method.getGovernanceState,
  input: {},
  cardinality: "finite",
});

import { createWorkspaceQueryClient } from "./query-client";

describe("createWorkspaceQueryClient", () => {
  it.each([
    ["succeeds", () => Promise.resolve("done")],
    ["fails", () => Promise.reject(new Error("adb exited"))],
  ])("marks governance data stale when any command %s", async (_label, mutationFn) => {
    const queryClient = createWorkspaceQueryClient();
    queryClient.setQueryData(governance, create(GovernanceStateSchema));
    queryClient.setQueryData(snapshotKey(), create(WorkspaceSnapshotSchema));

    const observer = new MutationObserver(queryClient, {
      mutationKey: workspaceMutationKeys.connectDevice(),
      mutationFn,
    });
    await observer.mutate().catch(() => undefined);

    expect(queryClient.getQueryState(governance)?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(snapshotKey())?.isInvalidated).toBe(false);
  });
});
