import { describe, expect, it } from "vitest";

import type {} from "@/shared/proto";
import {
  ApiGatewayService,
  AutomationService,
  BatchActionKind,
  BatchService,
  DeviceStatus,
  DocumentService,
  GovernanceService,
  GovernedAction,
  RunStatus,
  StepStatus,
  TaskStatus,
  WorkspaceService,
} from "@/shared/proto";

import { mockBackendHarness } from "./mock-harness";

async function onlineDevice(call: ReturnType<typeof mockBackendHarness>): Promise<string> {
  const snapshot = await call.client(WorkspaceService).getWorkspaceSnapshot({});

  return snapshot.devices.find((device) => device.status === DeviceStatus.ONLINE)?.id ?? "";
}

async function saveDocument(
  call: ReturnType<typeof mockBackendHarness>,
  collection: string,
  id: string,
  body: object,
) {
  await call
    .client(DocumentService)
    .saveDocument({ collection, document: { id, body: JSON.stringify(body) } });
}

describe("mock automation, scripts and local API", () => {
  it("runs library scripts on devices and reports each device's output", async () => {
    const call = mockBackendHarness();
    const device = await onlineDevice(call);
    await saveDocument(call, "scripts", "model", {
      name: "Model",
      language: "SCRIPT_LANGUAGE_SHELL",
      body: "getprop ro.product.model",
    });

    const ran = await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.RUN_SCRIPT,
      deviceIds: [device],
      scriptId: "model",
    });
    const missing = await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.RUN_SCRIPT,
      deviceIds: [device],
      scriptId: "absent",
    });

    expect(ran.task?.status).toBe(TaskStatus.DONE);
    expect(ran.scriptOutputs?.[0]).toMatchObject({ deviceId: device, exitCode: 0 });
    expect(ran.scriptOutputs?.[0]?.output).toContain("getprop ro.product.model");
    expect(missing.task?.detail?.key).toBe("workspace.scripts.missing");
  });

  it("runs a stored flow in the background and records it", async () => {
    const call = mockBackendHarness();
    const device = await onlineDevice(call);
    await saveDocument(call, "automation-flows", "morning", {
      name: "Morning",
      steps: [{ id: "s1", kind: "BATCH_ACTION_KIND_LAUNCH_APP", value: "com.example" }],
    });

    const automation = call.client(AutomationService);
    const target = { target: { case: "devices" as const, value: { deviceIds: [device] } } };
    const started = await automation.startFlowRun({
      flowId: "morning",
      target,
      options: { repeat: 2 },
    });
    const refused = await automation.startFlowRun({ flowId: "absent", target });
    await new Promise((resolve) => setTimeout(resolve, 5));
    const { runs } = await automation.listFlowRuns({});
    const governance = await call.client(GovernanceService).getGovernanceState({});

    expect(started.accepted).toBe(true);
    expect(refused.detail?.key).toBe("workspace.automation.flowMissing");
    expect(runs[0]).toMatchObject({
      status: RunStatus.SUCCEEDED,
      iteration: 2,
      steps: [expect.objectContaining({ status: StepStatus.DONE })],
      target,
      options: { repeat: 2 },
      operator: { id: "local:browser" },
    });
    expect(governance.auditTrail[0]?.action).toBe(GovernedAction.RUN_AUTOMATION);
  });

  it("keeps schedules and issues local API tokens", async () => {
    const call = mockBackendHarness();
    const automation = call.client(AutomationService);
    const saved = await automation.saveSchedule({
      schedule: { id: "s-1", name: "Morning", flowId: "morning", cron: "0 9 * * *", enabled: true },
    });
    const badCron = await automation.saveSchedule({
      schedule: { id: "s-2", name: "Broken", flowId: "morning", cron: "daily" },
    });
    const { schedules } = await automation.listSchedules({});
    const gateway = call.client(ApiGatewayService);
    const enabled = await gateway.updateApiGateway({ config: { enabled: true, port: 0 } });
    const token = await gateway.createApiToken({ name: "CI" });

    expect(saved.accepted).toBe(true);
    expect(badCron.detail?.key).toBe("workspace.schedules.invalid.cron");
    expect(schedules.map((state) => state.schedule?.id)).toEqual(["s-1"]);
    expect(schedules[0]?.schedule?.owner?.id).toBe("local:browser");
    expect(enabled.state?.address).toBe("http://127.0.0.1:47831");
    expect(token.secret).toMatch(/^yhsg_/);
    expect(token.state?.tokens.map((item) => item.name)).toEqual(["CI"]);
  });
});
