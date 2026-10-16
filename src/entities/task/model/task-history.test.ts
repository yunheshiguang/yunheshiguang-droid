import { create, type MessageInitShape } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import {
  AdbErrorKind,
  BatchActionKind,
  BatchActionResultSchema,
  TaskItemSchema,
  TaskStatus,
  UiTextSchema,
} from "@/shared/proto";

import {
  createTaskHistoryStore,
  mergeTasks,
  recordRun,
  type TaskRun,
  toTaskRun,
} from "./task-history";
import type { TaskItem } from "./types";

function task(id: string, overrides: MessageInitShape<typeof TaskItemSchema> = {}): TaskItem {
  return create(TaskItemSchema, {
    id,
    status: TaskStatus.DONE,
    progress: 100,
    actionKind: BatchActionKind.SCREENSHOT,
    concurrencyLimit: 4,
    retryCount: 0,
    ...overrides,
  });
}

function run(id: string, overrides: MessageInitShape<typeof TaskItemSchema> = {}): TaskRun {
  return {
    task: task(id, overrides),
    deviceResults: [],
    diagnostics: [],
    deviceHints: [],
    inputRoutes: [],
    scriptOutputs: [],
  };
}

const ids = (runs: readonly TaskRun[]) => runs.map((item) => item.task.id);

describe("recordRun", () => {
  it("puts the newest run first", () => {
    expect(ids(recordRun([run("a")], run("b")))).toEqual(["b", "a"]);
  });

  it("updates a known task in place instead of duplicating it", () => {
    const updated = recordRun([run("b"), run("a", { status: TaskStatus.RUNNING })], run("a"));

    expect(updated.map((item) => [item.task.id, item.task.status])).toEqual([
      ["b", TaskStatus.DONE],
      ["a", TaskStatus.DONE],
    ]);
  });

  it("drops the oldest runs beyond the limit", () => {
    const history = ["c", "b", "a"].reduce<readonly TaskRun[]>(
      (runs, id) => recordRun(runs, run(id), 2),
      [],
    );

    expect(ids(history)).toEqual(["a", "b"]);
  });
});

describe("toTaskRun", () => {
  it("keeps the per-device results together with the task", () => {
    const deviceResults = [
      {
        deviceId: "serial-a",
        success: false,
        errorKind: AdbErrorKind.TIMEOUT,
        message: "timed out",
      },
    ];

    const deviceHints = [
      {
        deviceId: "serial-a",
        vendorId: "core.xiaomi",
        hint: create(UiTextSchema, { key: "hint" }),
      },
    ];

    const inputRoutes = [{ deviceId: "serial-a", channelId: "core.yadb" }];
    const scriptOutputs = [
      { deviceId: "serial-a", engineId: "core.shell", exitCode: 1, output: "x", truncated: false },
    ];

    expect(
      toTaskRun(
        create(BatchActionResultSchema, {
          accepted: true,
          task: task("a"),
          deviceResults,
          deviceHints,
          inputRoutes,
          scriptOutputs,
        }),
      ),
    ).toMatchObject({
      task: task("a"),
      diagnostics: [],
      deviceResults,
      deviceHints,
      inputRoutes,
      scriptOutputs,
    });
  });

  it("records nothing for a result that carries no task", () => {
    expect(
      toTaskRun(
        create(BatchActionResultSchema, {
          accepted: false,
          diagnostics: [],
          deviceResults: [],
          deviceHints: [],
          inputRoutes: [],
          scriptOutputs: [],
        }),
      ),
    ).toBeUndefined();
  });
});

describe("mergeTasks", () => {
  it("keeps the local list untouched while the backend sends no tasks", () => {
    const local = [task("a")];

    expect(mergeTasks(local, [])).toBe(local);
  });

  it("prefers the backend version of a task and appends backend-only tasks", () => {
    const merged = mergeTasks(
      [task("a", { status: TaskStatus.RUNNING }), task("b")],
      [task("a", { status: TaskStatus.FAILED }), task("remote")],
    );

    expect(merged.map((item) => [item.id, item.status])).toEqual([
      ["a", TaskStatus.FAILED],
      ["b", TaskStatus.DONE],
      ["remote", TaskStatus.DONE],
    ]);
  });
});

describe("task history store", () => {
  it("records and clears runs through its actions", () => {
    const store = createTaskHistoryStore();

    store.getState().actions.record(run("a"));
    store.getState().actions.record(run("b"));
    expect(ids(store.getState().runs)).toEqual(["b", "a"]);

    store.getState().actions.clear();
    expect(store.getState().runs).toEqual([]);
  });
});
