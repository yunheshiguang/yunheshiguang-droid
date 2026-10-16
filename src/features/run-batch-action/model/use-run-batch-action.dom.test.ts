import { create, fromJson, type JsonValue } from "@bufbuild/protobuf";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { useTaskHistory } from "@/entities/task";
import { snapshotKey } from "@/shared/api";
import {
  AdbErrorKind,
  BatchActionKind,
  BatchActionRequestSchema,
  BatchActionResultSchema,
  BatchService,
  TaskStatus,
  WorkspaceSnapshotSchema,
} from "@/shared/proto";
import {
  createWorkspaceTestHarness,
  installFakeServices,
  type WorkspaceTestHarness,
} from "@/shared/testing";

import { useRunBatchAction } from "./use-run-batch-action";

const partialFailure = {
  accepted: true,
  task: {
    id: "batch-screenshot-1",
    title: { key: "workspace.task.title.screenshot" },
    status: "TASK_STATUS_FAILED",
    actionKind: "BATCH_ACTION_KIND_SCREENSHOT",
    concurrencyLimit: 4,
  },
  deviceResults: [
    { deviceId: "serial-a", success: true },
    {
      deviceId: "serial-b",
      success: false,
      errorKind: "ADB_ERROR_KIND_TIMEOUT",
      message: "timed out",
    },
  ],
};

const screenshot = () =>
  create(BatchActionRequestSchema, {
    kind: BatchActionKind.SCREENSHOT,
    deviceIds: ["serial-a", "serial-b"],
  });

let pendingBatches: Array<(result: unknown) => void> = [];
let harness: WorkspaceTestHarness;

function renderRunner() {
  return renderHook(() => ({ runner: useRunBatchAction(), tasks: useTaskHistory() }), {
    wrapper: harness.wrapper,
  });
}

async function finishBatch(result: unknown) {
  await waitFor(() => expect(pendingBatches.length).toBeGreaterThan(0));
  await act(async () => pendingBatches.shift()?.(result));
}

beforeEach(() => {
  pendingBatches = [];
  harness = createWorkspaceTestHarness();
  installFakeServices((router) =>
    router.service(BatchService, {
      runBatchAction: () =>
        new Promise((resolve) =>
          pendingBatches.push((result) =>
            resolve(fromJson(BatchActionResultSchema, result as JsonValue)),
          ),
        ),
    }),
  );
});

describe("useRunBatchAction", () => {
  it("reports a running batch, then every device's result, and records the task", async () => {
    harness.queryClient.setQueryData(snapshotKey(), create(WorkspaceSnapshotSchema));
    const { result } = renderRunner();

    act(() => result.current.runner.run(screenshot()));
    await waitFor(() => expect(result.current.runner.running).toBe(true));
    expect(result.current.runner.view.status).toBe("running");

    await finishBatch(partialFailure);
    await waitFor(() => expect(result.current.runner.view.status).toBe("finished"));

    const { view } = result.current.runner;
    expect(view.status === "finished" ? view.run.deviceResults : []).toMatchObject([
      { deviceId: "serial-a", success: true, errorKind: AdbErrorKind.UNSPECIFIED, message: "" },
      {
        deviceId: "serial-b",
        success: false,
        errorKind: AdbErrorKind.TIMEOUT,
        message: "timed out",
      },
    ]);
    expect(result.current.runner.running).toBe(false);
    expect(result.current.tasks.map((task) => [task.id, task.status])).toEqual([
      ["batch-screenshot-1", TaskStatus.FAILED],
    ]);
    expect(harness.queryClient.getQueryState(snapshotKey())?.isInvalidated).toBe(true);
  });

  it("keeps the running state and the result across a remount", async () => {
    const first = renderRunner();

    act(() =>
      first.result.current.runner.run(
        create(BatchActionRequestSchema, {
          kind: BatchActionKind.RECOVER_CONNECTION,
          deviceIds: ["serial-a"],
        }),
      ),
    );
    await waitFor(() => expect(first.result.current.runner.running).toBe(true));
    first.unmount();

    const second = renderRunner();
    expect(second.result.current.runner.running).toBe(true);

    await finishBatch(partialFailure);
    await waitFor(() => expect(second.result.current.runner.view.status).toBe("finished"));

    expect(second.result.current.runner.running).toBe(false);
    expect(second.result.current.tasks.map((task) => task.id)).toEqual(["batch-screenshot-1"]);
  });

  it("still shows the finished run after the command cache drops the attempt", async () => {
    const { result } = renderRunner();

    act(() => result.current.runner.run(screenshot()));
    await finishBatch(partialFailure);
    await waitFor(() => expect(result.current.runner.view.status).toBe("finished"));

    act(() => harness.queryClient.getMutationCache().clear());

    await waitFor(() => expect(result.current.runner.view.status).toBe("finished"));
    const { view } = result.current.runner;
    expect(view.status === "finished" ? view.run.task.id : undefined).toBe("batch-screenshot-1");
  });

  it("starts every test with an empty task history", () => {
    expect(renderRunner().result.current.tasks).toEqual([]);
  });
});
