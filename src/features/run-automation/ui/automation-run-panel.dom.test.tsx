import { create, fromJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";
import { act, render, renderHook, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { type AutomationFlow, useAutomationFlowActions } from "@/entities/automation";
import { flowRunMessages } from "@/entities/flow-run";
import { taskMessages } from "@/entities/task";
import {
  AutomationService,
  BatchActionKind,
  type DeviceGroup,
  DeviceGroupSchema,
  type FlowRunJson,
  FlowRunResultSchema,
  FlowRunsSchema,
  type StartFlowRunRequest,
  type StopFlowRunRequest,
  UiTextSchema,
} from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { runAutomationMessages } from "../messages";
import { AutomationRunPanel } from "./automation-run-panel";

const flow: AutomationFlow = {
  id: "flow-1",
  name: "Install and launch",
  steps: [
    { id: "install", kind: BatchActionKind.INSTALL_APK, value: "/tmp/app.apk" },
    { id: "launch", kind: BatchActionKind.LAUNCH_APP, value: "com.example" },
    { id: "shot", kind: BatchActionKind.SCREENSHOT, value: "/tmp/shots" },
  ],
};

const groups: DeviceGroup[] = [
  create(DeviceGroupSchema, {
    id: "shelf-a",
    name: create(UiTextSchema, { key: "", params: {}, fallback: "Shelf A" }),
    total: 2,
    online: 2,
  }),
];

const inProgress: FlowRunJson = {
  id: "run-1",
  flowId: "flow-1",
  flowName: "Install and launch",
  trigger: "RUN_TRIGGER_MANUAL",
  actor: "main",
  status: "RUN_STATUS_RUNNING",
  deviceIds: ["serial-a"],
  iteration: 1,
  iterations: 2,
  steps: [
    {
      stepId: "install",
      kind: "BATCH_ACTION_KIND_INSTALL_APK",
      status: "STEP_STATUS_DONE",
      attempts: 2,
      result: {
        accepted: true,
        task: { id: "t", status: "TASK_STATUS_DONE" },
        deviceResults: [{ deviceId: "serial-a", success: true }],
      },
    },
    { stepId: "launch", kind: "BATCH_ACTION_KIND_LAUNCH_APP", status: "STEP_STATUS_RUNNING" },
    { stepId: "shot", kind: "BATCH_ACTION_KIND_SCREENSHOT", status: "STEP_STATUS_PENDING" },
  ],
};

type Automation = Partial<ServiceImpl<typeof AutomationService>>;

async function renderPanel(
  selectedDeviceIds: readonly string[],
  automation: Automation = {},
  saved: AutomationFlow = flow,
) {
  installFakeServices(createFakeDocuments().routes, (router) =>
    router.service(AutomationService, { listFlowRuns: () => ({ runs: [] }), ...automation }),
  );
  const { wrapper } = createWorkspaceTestHarness(
    taskMessages,
    flowRunMessages,
    runAutomationMessages,
  );
  const { result } = renderHook(() => useAutomationFlowActions(), { wrapper });
  await act(() => result.current.save(saved));

  render(
    <AutomationRunPanel selectedDeviceIds={selectedDeviceIds} groups={groups} tags={["night"]} />,
    { wrapper },
  );
}

describe("AutomationRunPanel", () => {
  it("asks the backend to run the flow and follows the run it reports", async () => {
    let runs: FlowRunJson[] = [];
    const start = vi.fn((_request: StartFlowRunRequest) => {
      runs = [inProgress];
      return fromJson(FlowRunResultSchema, { accepted: true, run: inProgress });
    });
    const stop = vi.fn((_request: StopFlowRunRequest) => ({ accepted: true }));
    await renderPanel(["serial-a"], {
      listFlowRuns: () => fromJson(FlowRunsSchema, { runs }),
      startFlowRun: start,
      stopFlowRun: stop,
    });

    expect(await screen.findByText("1 device selected on the device wall")).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText("Retries for failed devices"));
    await userEvent.type(screen.getByLabelText("Retries for failed devices"), "2");
    await userEvent.click(screen.getByRole("button", { name: "Run" }));

    expect(start.mock.calls[0]?.[0]).toMatchObject({
      flowId: "flow-1",
      target: { target: { case: "devices", value: { deviceIds: ["serial-a"] } } },
      options: { repeat: 1, retryAttempts: 2 },
    });
    expect(await screen.findByRole("status")).toHaveTextContent("“Install and launch”, run 1 of 2");
    const [install, launch, shot] = within(
      screen.getByRole("region", { name: "Run progress" }),
    ).getAllByRole("listitem");
    expect(install).toHaveTextContent("Step 1 · Install APK");
    expect(install).toHaveTextContent("Done");
    expect(install).toHaveTextContent("1 of 1 device succeeded · 2 attempts");
    expect(launch).toHaveTextContent("Running");
    expect(shot).toHaveTextContent("Waiting");
    expect(screen.getByRole("button", { name: "Run" })).toBeDisabled();

    await userEvent.click(screen.getByRole("button", { name: "Stop after this step" }));
    expect(stop.mock.calls[0]?.[0]).toMatchObject({ runId: "run-1" });
  });

  it("runs on a group and shows why the backend refused", async () => {
    const start = vi.fn((_request: StartFlowRunRequest) => ({
      accepted: false,
      detail: { key: "workspace.automation.noDevices" },
    }));
    await renderPanel([], { startFlowRun: start });

    await userEvent.selectOptions(await screen.findByLabelText("Run on"), "group");
    expect(screen.getByRole("button", { name: "Run" })).toBeDisabled();
    await userEvent.selectOptions(screen.getByLabelText("Group"), "shelf-a");
    await userEvent.click(screen.getByRole("button", { name: "Run" }));

    expect(start.mock.calls[0]?.[0]).toMatchObject({
      target: { target: { case: "groupId", value: "shelf-a" } },
    });
    expect(await screen.findByRole("alert")).toHaveTextContent("No device in the target is online");
  });

  it("asks before running a flow with steps that cannot be undone", async () => {
    const start = vi.fn((_request: StartFlowRunRequest) => ({ accepted: true }));
    await renderPanel(
      ["serial-a"],
      { startFlowRun: start },
      {
        ...flow,
        steps: [
          ...flow.steps,
          { id: "wipe", kind: BatchActionKind.CLEAR_APP_DATA, value: "com.example" },
          { id: "remove", kind: BatchActionKind.UNINSTALL_APP, value: "com.example" },
        ],
      },
    );

    await userEvent.click(await screen.findByRole("button", { name: "Run" }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm).toHaveTextContent("Run “Install and launch”?");
    expect(confirm).toHaveTextContent("Clear app data and Uninstall app");
    expect(start).not.toHaveBeenCalled();

    await userEvent.click(within(confirm).getByRole("button", { name: "Run" }));
    expect(start).toHaveBeenCalledTimes(1);
  });

  it("waits for devices before it lets a flow run on them", async () => {
    await renderPanel([]);

    expect(await screen.findByText("Select devices on the device wall first")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Run" })).toBeDisabled();
  });
});
