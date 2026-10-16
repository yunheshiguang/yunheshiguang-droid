import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { WorkspaceService } from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import {
  SNAPSHOT_POLL_INTERVAL_MS,
  useWorkspaceDevices,
  useWorkspaceSnapshotStatus,
} from "./use-workspace-snapshot";

let snapshotCalls = 0;
let connected: string[] = [];
let heldReply: Promise<void> | null = null;

function holdReplies(): () => void {
  let release = () => {};
  heldReply = new Promise((resolve) => {
    release = () => {
      heldReply = null;
      resolve();
    };
  });

  return release;
}

function renderSnapshot() {
  const { wrapper } = createWorkspaceTestHarness();

  return renderHook(
    () => ({ devices: useWorkspaceDevices(), status: useWorkspaceSnapshotStatus() }),
    { wrapper },
  );
}

function setWindowHidden(hidden: boolean) {
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    get: () => (hidden ? "hidden" : "visible"),
  });
  document.dispatchEvent(new Event("visibilitychange"));
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  snapshotCalls = 0;
  connected = [];
  heldReply = null;
  installFakeServices((router) =>
    router.service(WorkspaceService, {
      getWorkspaceSnapshot: async () => {
        snapshotCalls += 1;
        await heldReply;
        return { schemaVersion: "2026.06.p0", devices: connected.map((id) => ({ id })) };
      },
    }),
  );
});

afterEach(() => {
  setWindowHidden(false);
  vi.useRealTimers();
});

describe("useWorkspaceSnapshotStatus", () => {
  it("picks up a newly plugged device without a manual refresh", async () => {
    const { result } = renderSnapshot();
    await waitFor(() => expect(result.current.status.loading).toBe(false));
    expect(result.current.devices).toHaveLength(0);

    connected = ["serial-a"];
    await act(() => vi.advanceTimersByTimeAsync(SNAPSHOT_POLL_INTERVAL_MS));

    await waitFor(() => expect(result.current.devices).toHaveLength(1));
    expect(result.current.status.refreshing).toBe(false);
  });

  it("stops polling while the window is in the background", async () => {
    const { result } = renderSnapshot();
    await waitFor(() => expect(result.current.status.loading).toBe(false));
    const callsBeforeHiding = snapshotCalls;

    act(() => setWindowHidden(true));
    await act(() => vi.advanceTimersByTimeAsync(SNAPSHOT_POLL_INTERVAL_MS * 3));

    expect(snapshotCalls).toBe(callsBeforeHiding);
  });

  it("reports a refresh the user asked for, not a background poll", async () => {
    const { result } = renderSnapshot();
    await waitFor(() => expect(result.current.status.loading).toBe(false));

    const releasePoll = holdReplies();
    const callsBeforePoll = snapshotCalls;
    await act(() => vi.advanceTimersByTimeAsync(SNAPSHOT_POLL_INTERVAL_MS));
    expect(snapshotCalls).toBe(callsBeforePoll + 1);
    expect(result.current.status.refreshing).toBe(false);
    await act(async () => releasePoll());

    const releaseRefresh = holdReplies();
    let refreshed: Promise<void> = Promise.resolve();
    act(() => {
      refreshed = result.current.status.refresh();
    });
    await waitFor(() => expect(result.current.status.refreshing).toBe(true));

    await act(async () => {
      releaseRefresh();
      await refreshed;
    });
    await waitFor(() => expect(result.current.status.refreshing).toBe(false));
  });
});
