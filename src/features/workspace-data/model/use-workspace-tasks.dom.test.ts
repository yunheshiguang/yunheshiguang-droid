import { create, fromJson, type JsonObject } from "@bufbuild/protobuf";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { type TaskRun, useTaskHistoryActions } from "@/entities/task";
import {
  BatchActionKind,
  TaskItemSchema,
  TaskStatus,
  WorkspaceService,
  WorkspaceSnapshotSchema,
} from "@/shared/proto";
import {
  createWorkspaceTestHarness,
  installFakeServices,
  type WorkspaceTestHarness,
} from "@/shared/testing";

import { useWorkspaceDevices, useWorkspaceSnapshotStatus } from "./use-workspace-snapshot";
import { useWorkspaceTasks } from "./use-workspace-tasks";

function localRun(id: string): TaskRun {
  return {
    task: create(TaskItemSchema, {
      id,
      status: TaskStatus.DONE,
      progress: 100,
      actionKind: BatchActionKind.SCREENSHOT,
      concurrencyLimit: 4,
      retryCount: 0,
    }),
    deviceResults: [],
    deviceHints: [],
    inputRoutes: [],
    scriptOutputs: [],
    diagnostics: [],
  };
}

let nextSnapshot: JsonObject = {};
let harness: WorkspaceTestHarness;

function renderTasks() {
  return renderHook(
    () => ({
      tasks: useWorkspaceTasks(),
      devices: useWorkspaceDevices(),
      history: useTaskHistoryActions(),
      snapshot: useWorkspaceSnapshotStatus(),
    }),
    { wrapper: harness.wrapper },
  );
}

beforeEach(() => {
  nextSnapshot = { schemaVersion: "2026.06.p0" };
  harness = createWorkspaceTestHarness();
  installFakeServices((router) =>
    router.service(WorkspaceService, {
      getWorkspaceSnapshot: () => fromJson(WorkspaceSnapshotSchema, nextSnapshot),
    }),
  );
});

describe("useWorkspaceTasks", () => {
  it("keeps the local task history when the snapshot is replaced", async () => {
    const { result } = renderTasks();
    await waitFor(() => expect(result.current.snapshot.loading).toBe(false));

    act(() => result.current.history.record(localRun("local-1")));
    nextSnapshot = { ...nextSnapshot, devices: [{ id: "serial-a" }] };
    await act(() => result.current.snapshot.refresh());

    await waitFor(() => expect(result.current.devices).toHaveLength(1));
    expect(result.current.tasks.map((task) => task.id)).toEqual(["local-1"]);
  });

  it("adds tasks the backend starts sending without dropping local ones", async () => {
    const { result } = renderTasks();
    await waitFor(() => expect(result.current.snapshot.loading).toBe(false));

    act(() => result.current.history.record(localRun("local-1")));
    nextSnapshot = { ...nextSnapshot, tasks: [{ id: "remote-1", status: "TASK_STATUS_RUNNING" }] };
    await act(() => result.current.snapshot.refresh());

    await waitFor(() =>
      expect(result.current.tasks.map((task) => [task.id, task.status])).toEqual([
        ["local-1", TaskStatus.DONE],
        ["remote-1", TaskStatus.RUNNING],
      ]),
    );
  });
});
