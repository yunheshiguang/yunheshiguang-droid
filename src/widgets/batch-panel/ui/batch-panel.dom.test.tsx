import { create, fromJson, type JsonValue, toJson } from "@bufbuild/protobuf";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { DeviceRecord } from "@/entities/device";
import { phraseMessages } from "@/entities/phrase";
import { taskMessages } from "@/entities/task";
import { inputGuidanceMessages } from "@/features/input-guidance";
import { runBatchActionMessages } from "@/features/run-batch-action";
import { type PathPickRequest, setDialogGateway } from "@/shared/api";
import {
  BatchActionRequestSchema,
  BatchActionResultSchema,
  BatchService,
  ConnectionHealth,
  ConnectionType,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  InputChannelsSchema,
  InputKind,
  InputService,
} from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { batchPanelMessages } from "../messages";
import { BatchPanel } from "./batch-panel";

const phone: DeviceRecord = create(DeviceRecordSchema, {
  id: "serial-a",
  status: DeviceStatus.ONLINE,
  connection: ConnectionType.USB,
  tags: [],
  health: ConnectionHealth.READY,
  diagnostic: DiagnosticCode.NONE,
  selectedRank: 1,
  groupId: "",
});

let pending: Array<{ payload: { request: JsonValue }; finish: (result: JsonValue) => void }> = [];

function serveBatches() {
  installFakeServices((router) =>
    router.service(BatchService, {
      runBatchAction: (request) =>
        new Promise((resolve) =>
          pending.push({
            payload: { request: toJson(BatchActionRequestSchema, request) },
            finish: (result) => resolve(fromJson(BatchActionResultSchema, result)),
          }),
        ),
    }),
  );
}

function renderPanel(selectedDevices: readonly DeviceRecord[]) {
  const { wrapper } = createWorkspaceTestHarness(
    taskMessages,
    phraseMessages,
    inputGuidanceMessages,
    runBatchActionMessages,
    batchPanelMessages,
  );

  return render(<BatchPanel selectedDevices={selectedDevices} devices={[phone]} />, { wrapper });
}

beforeEach(() => {
  pending = [];
  serveBatches();
});

describe("BatchPanel", () => {
  it("cannot run without a selected device", () => {
    renderPanel([]);

    expect(screen.getByRole("button", { name: "Run" })).toBeDisabled();
    expect(screen.getByText("Select devices on the device wall first")).toBeInTheDocument();
  });

  it("sends only the chosen action's input and locks the button until the batch returns", async () => {
    const user = userEvent.setup();
    renderPanel([phone]);

    expect(screen.getByText("1 device")).toBeInTheDocument();

    await user.type(screen.getByPlaceholderText("Local output directory"), "/tmp/shots");
    await user.click(screen.getByRole("button", { name: "Launch app" }));
    await user.type(screen.getByPlaceholderText("Enter package name"), "com.example.app");
    await user.click(screen.getByRole("button", { name: "Run" }));

    await waitFor(() => expect(pending).toHaveLength(1));
    expect(pending[0].payload).toMatchObject({
      request: {
        kind: "BATCH_ACTION_KIND_LAUNCH_APP",
        deviceIds: ["serial-a"],
        packageName: "com.example.app",
      },
    });
    expect(pending[0].payload?.request).not.toHaveProperty("text");
    expect(pending[0].payload?.request).not.toHaveProperty("path");
    expect(screen.getByRole("button", { name: "Running…" })).toBeDisabled();

    pending[0].finish({
      accepted: true,
      task: {
        id: "batch-launch-app-1",
        title: { key: "workspace.task.title.launchApp" },
        status: "TASK_STATUS_DONE",
        detail: { key: "workspace.task.detail.completed", params: { count: "1" } },
      },
      deviceResults: [{ deviceId: "serial-a", success: true }],
    });

    expect(await screen.findByRole("button", { name: "Run" })).toBeEnabled();
    expect(screen.getByRole("heading", { name: "Launch app" })).toBeInTheDocument();
  });

  it("lets the system dialog pick an APK and sends the picked path", async () => {
    const pickPath = vi.fn(async (_request: PathPickRequest) => "/Users/ops/builds/app.apk");
    setDialogGateway({ pickPath });
    const user = userEvent.setup();
    renderPanel([phone]);

    await user.click(screen.getByRole("button", { name: "Install APK" }));
    await user.click(screen.getByRole("button", { name: "Browse…" }));

    expect(pickPath).toHaveBeenCalledWith({ target: "file", extensions: ["apk"] });
    expect(await screen.findByDisplayValue("/Users/ops/builds/app.apk")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Run" }));

    await waitFor(() => expect(pending).toHaveLength(1));
    expect(pending[0].payload).toMatchObject({
      request: { kind: "BATCH_ACTION_KIND_INSTALL_APK", path: "/Users/ops/builds/app.apk" },
    });
  });

  it("fills typed text from the text library and shows who will deliver it", async () => {
    const documents = createFakeDocuments();
    documents.save(
      "phrases",
      "welcome",
      JSON.stringify({ version: 1, title: "Welcome", body: "你好，欢迎！", tags: [] }),
    );
    installFakeServices(documents.routes);
    installFakeServices((router) =>
      router.service(InputService, {
        listInputChannels: () =>
          create(InputChannelsSchema, {
            channels: [
              {
                id: "core.yadb",
                label: { key: "workspace.input.yadb.label" },
                kinds: [InputKind.TEXT, InputKind.CLIPBOARD],
              },
            ],
          }),
      }),
    );
    const user = userEvent.setup();
    renderPanel([phone]);

    await user.click(screen.getByRole("button", { name: "Write clipboard" }));
    expect(await screen.findByRole("region", { name: "Input channels" })).toHaveTextContent(
      "YADB (built-in)",
    );
    await user.selectOptions(await screen.findByLabelText("Insert a text snippet"), "welcome");
    expect(screen.getByPlaceholderText("Enter text to send")).toHaveValue("你好，欢迎！");

    await user.click(screen.getByRole("button", { name: "Run" }));
    await waitFor(() => expect(pending).toHaveLength(1));
    expect(pending[0].payload).toMatchObject({
      request: { kind: "BATCH_ACTION_KIND_CLIPBOARD_WRITE", text: "你好，欢迎！" },
    });
  });

  it("asks once more before an action that cannot be undone", async () => {
    const user = userEvent.setup();
    renderPanel([phone]);

    await user.click(screen.getByRole("button", { name: "Uninstall app" }));
    await user.type(screen.getByPlaceholderText("Enter package name"), "com.example.app");
    await user.click(screen.getByRole("button", { name: "Run" }));

    expect(screen.getByRole("alertdialog")).toHaveTextContent(
      "“Uninstall app” will run on 1 device. This cannot be undone",
    );
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(pending).toHaveLength(0);

    await user.click(screen.getByRole("button", { name: "Run" }));
    await user.click(screen.getByRole("button", { name: "Run anyway" }));

    await waitFor(() => expect(pending).toHaveLength(1));
    expect(pending[0].payload).toMatchObject({
      request: { kind: "BATCH_ACTION_KIND_UNINSTALL_APP", packageName: "com.example.app" },
    });
  });

  it("keeps what was typed when the dialog is cancelled", async () => {
    setDialogGateway({ pickPath: async () => null });
    const user = userEvent.setup();
    renderPanel([phone]);

    const directory = screen.getByPlaceholderText("Local output directory");
    await user.type(directory, "/tmp/shots");
    await user.click(screen.getByRole("button", { name: "Browse…" }));

    expect(directory).toHaveValue("/tmp/shots");
  });
});
