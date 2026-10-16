import { create } from "@bufbuild/protobuf";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { DeviceRecord } from "@/entities/device";
import { type TaskRun, taskMessages } from "@/entities/task";
import {
  AdbDeviceActionResultSchema,
  AdbErrorKind,
  BatchActionKind,
  BatchActionRequestSchema,
  ConnectionHealth,
  ConnectionType,
  DeviceHintSchema,
  DeviceInputRouteSchema,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  PiiClass,
  SearchableTextSchema,
  TaskItemSchema,
  TaskStatus,
  UiTextSchema,
  VisibilityMode,
} from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { runBatchActionMessages } from "../messages";
import { BatchRunResult } from "./batch-run-result";

function device(id: string, redacted: string): DeviceRecord {
  return create(DeviceRecordSchema, {
    id,
    name: create(SearchableTextSchema, {
      displayValue: id,
      canonicalValue: id.toLowerCase(),
      aliases: [],
      redactedValue: redacted,
      piiClass: PiiClass.DEVICE_IDENTIFIER,
      visibility: VisibilityMode.SEARCHABLE,
      searchable: true,
    }),
    status: DeviceStatus.ONLINE,
    connection: ConnectionType.USB,
    tags: [],
    health: ConnectionHealth.READY,
    diagnostic: DiagnosticCode.NONE,
    selectedRank: 1,
    groupId: "",
  });
}

const partialFailure: TaskRun = {
  task: create(TaskItemSchema, {
    id: "batch-screenshot-1",
    title: create(UiTextSchema, {
      key: "workspace.task.title.screenshot",
      params: {},
      fallback: "",
    }),
    status: TaskStatus.FAILED,
    progress: 0,
    actionKind: BatchActionKind.SCREENSHOT,
    concurrencyLimit: 4,
    retryCount: 0,
  }),
  deviceResults: [
    create(AdbDeviceActionResultSchema, {
      deviceId: "SERIAL-A",
      success: true,
      errorKind: AdbErrorKind.UNSPECIFIED,
      message: "",
    }),
    create(AdbDeviceActionResultSchema, {
      deviceId: "SERIAL-B",
      success: false,
      errorKind: AdbErrorKind.TIMEOUT,
      message: "timed out",
    }),
    create(AdbDeviceActionResultSchema, {
      deviceId: "SERIAL-GONE",
      success: false,
      errorKind: AdbErrorKind.IO,
      message: "broken pipe",
    }),
  ],
  diagnostics: [],
  deviceHints: [],
  inputRoutes: [],
  scriptOutputs: [],
};

const devices = [device("SERIAL-A", "SE*****-A"), device("SERIAL-B", "SE*****-B")];

function renderView(view: Parameters<typeof BatchRunResult>[0]["view"]) {
  const { wrapper } = createWorkspaceTestHarness(runBatchActionMessages);

  return render(<BatchRunResult view={view} devices={devices} />, { wrapper });
}

describe("BatchRunResult", () => {
  it("summarizes the batch and lists failures first under masked names", () => {
    renderView({ status: "finished", run: partialFailure });

    expect(screen.getByRole("heading", { name: "Screenshot" })).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("1 of 3 devices succeeded");
    expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual([
      "FailedSE*****-BTimed out · timed out",
      "FailedDisconnected deviceRead or write failed · broken pipe",
      "SucceededSE*****-A",
    ]);
    expect(screen.queryByText("SERIAL-B")).not.toBeInTheDocument();
  });

  it("explains a failure the way the device's vendor adapter does", () => {
    renderView({
      status: "finished",
      run: {
        ...partialFailure,
        deviceHints: [
          create(DeviceHintSchema, {
            deviceId: "SERIAL-B",
            vendorId: "core.xiaomi",
            hint: create(UiTextSchema, {
              key: "workspace.vendors.xiaomi.explain.installViaUsb",
              params: {},
              fallback: "",
            }),
          }),
        ],
      },
    });

    const [failedWithHint, failedWithoutHint] = screen.getAllByRole("listitem");
    expect(failedWithHint).toHaveTextContent(
      "Xiaomi refused the install. Turn on “Install via USB” in Developer options",
    );
    expect(failedWithoutHint).not.toHaveTextContent("Xiaomi");
  });

  it("names the input channel that served each device, and why it failed", () => {
    renderView({
      status: "finished",
      run: {
        ...partialFailure,
        inputRoutes: [
          create(DeviceInputRouteSchema, {
            deviceId: "SERIAL-A",
            channelId: "core.yadb",
            channel: create(UiTextSchema, {
              key: "workspace.input.yadb.label",
              params: {},
              fallback: "",
            }),
          }),
          create(DeviceInputRouteSchema, {
            deviceId: "SERIAL-B",
            channelId: "core.yadb",
            channel: create(UiTextSchema, {
              key: "workspace.input.yadb.label",
              params: {},
              fallback: "",
            }),
            failure: create(UiTextSchema, {
              key: "workspace.input.yadb.unconfirmed",
              params: {},
              fallback: "",
            }),
          }),
        ],
      },
    });

    const [failed, , succeeded] = screen.getAllByRole("listitem");
    expect(failed).toHaveTextContent(
      "Via YADB (built-in) · YADB could not write the clipboard, so no text was entered",
    );
    expect(succeeded).toHaveTextContent("Via YADB (built-in)");
  });

  it("uses the singular for a batch sent to one device", () => {
    renderView({
      status: "finished",
      run: { ...partialFailure, deviceResults: partialFailure.deviceResults.slice(1, 2) },
    });

    expect(screen.getByRole("status")).toHaveTextContent("0 of 1 device succeeded");
  });

  it("says so when the request never reached the backend", () => {
    renderView({ status: "unreachable", error: "ipc channel closed" });

    expect(screen.getByRole("alert")).toHaveTextContent("Request failed");
    expect(screen.getByRole("alert")).toHaveTextContent("ipc channel closed");
  });

  it("announces a running batch", () => {
    renderView({ status: "running" });

    expect(screen.getByRole("status")).toHaveTextContent("Running…");
  });
});

describe("BatchRunResult replay", () => {
  it("sends the same request again, and only where replaying is offered", async () => {
    const request = create(BatchActionRequestSchema, {
      kind: BatchActionKind.SCREENSHOT,
      deviceIds: ["SERIAL-A", "SERIAL-B"],
      path: "/tmp",
    });
    const onReplay = vi.fn();
    const run = { ...partialFailure, request };
    const { wrapper } = createWorkspaceTestHarness(runBatchActionMessages, taskMessages);
    const { rerender } = render(
      <BatchRunResult view={{ status: "finished", run }} devices={devices} onReplay={onReplay} />,
      { wrapper },
    );

    await userEvent.click(screen.getByRole("button", { name: "Run again" }));
    expect(onReplay).toHaveBeenCalledWith(request);

    rerender(<BatchRunResult view={{ status: "finished", run }} devices={devices} />);
    expect(screen.queryByRole("button", { name: "Run again" })).not.toBeInTheDocument();
  });

  it("asks before sending an action that cannot be undone again", async () => {
    const request = create(BatchActionRequestSchema, {
      kind: BatchActionKind.UNINSTALL_APP,
      deviceIds: ["SERIAL-A", "SERIAL-B"],
      packageName: "com.example.app",
    });
    const onReplay = vi.fn();
    const { wrapper } = createWorkspaceTestHarness(runBatchActionMessages, taskMessages);
    render(
      <BatchRunResult
        view={{ status: "finished", run: { ...partialFailure, request } }}
        devices={devices}
        onReplay={onReplay}
      />,
      { wrapper },
    );

    await userEvent.click(screen.getByRole("button", { name: "Run again" }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm).toHaveTextContent("Run “Uninstall app” again on 2 devices?");
    expect(onReplay).not.toHaveBeenCalled();

    await userEvent.click(within(confirm).getByRole("button", { name: "Run again" }));
    expect(onReplay).toHaveBeenCalledWith(request);
  });
});
