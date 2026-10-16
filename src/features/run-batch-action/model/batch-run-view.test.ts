import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import type { TaskRun } from "@/entities/task";
import { BatchActionKind, TaskItemSchema, TaskStatus } from "@/shared/proto";

import { resolveBatchRunView } from "./batch-run-view";

const finished: TaskRun = {
  task: create(TaskItemSchema, {
    id: "batch-screenshot-1",
    status: TaskStatus.DONE,
    progress: 100,
    actionKind: BatchActionKind.SCREENSHOT,
    concurrencyLimit: 4,
    retryCount: 0,
  }),
  deviceResults: [],
  diagnostics: [],
  deviceHints: [],
  inputRoutes: [],
  scriptOutputs: [],
};

describe("resolveBatchRunView", () => {
  it("shows nothing before the first run", () => {
    expect(resolveBatchRunView(undefined, undefined)).toEqual({ status: "idle" });
  });

  it("shows a running batch even when an older run is in the history", () => {
    expect(resolveBatchRunView({ status: "pending", error: null }, finished)).toEqual({
      status: "running",
    });
  });

  it("reports a request that never reached the backend", () => {
    expect(resolveBatchRunView({ status: "error", error: "ipc closed" }, finished)).toEqual({
      status: "unreachable",
      error: "ipc closed",
    });
  });

  it("falls back to the history once the attempt has been cleaned up", () => {
    expect(resolveBatchRunView(undefined, finished)).toEqual({ status: "finished", run: finished });
    expect(resolveBatchRunView({ status: "success", error: null }, finished)).toEqual({
      status: "finished",
      run: finished,
    });
  });
});
