import { describe, expect, it } from "vitest";

import { assessSchemaCompatibility, workspaceCommandNames } from "@/shared/api";
import {
  AuditOutcome,
  BatchActionKind,
  BatchService,
  ConnectionOutcome,
  ConnectionService,
  ConnectionType,
  DiscoveryService,
  DocumentService,
  GovernanceService,
  GovernedAction,
  GroupTone,
  InputService,
  ProfileService,
  TaskStatus,
  TelemetryService,
  TemplateService,
  VendorService,
  WorkspaceService,
} from "@/shared/proto";

import { createMockBackend } from "./mock-backend";
import { mockBackendHarness } from "./mock-harness";

describe("createMockBackend", () => {
  it("answers every command the frontend declares", () => {
    const { handlers } = createMockBackend({ latencyMs: 0 });

    expect(Object.keys(handlers).sort()).toEqual([...workspaceCommandNames].sort());
  });

  it("serves a snapshot the frontend accepts", async () => {
    const call = mockBackendHarness();
    const snapshot = await call.client(WorkspaceService).getWorkspaceSnapshot({});

    expect(assessSchemaCompatibility(snapshot.schemaVersion)).toBe("compatible");
    expect(snapshot.devices.map((device) => device.connection)).toContain(ConnectionType.WIFI);
  });

  it("attaches and detaches network devices and audits both", async () => {
    const call = mockBackendHarness();
    const endpoint = { host: "10.0.0.8", port: 5555 };

    const connection = call.client(ConnectionService);
    const connected = await connection.connectDevice({ endpoint });
    const again = await connection.connectDevice({ endpoint });
    const listed = await call.client(WorkspaceService).getWorkspaceSnapshot({});
    const detached = await connection.disconnectDevice({ deviceId: "10.0.0.8:5555" });
    const missing = await connection.disconnectDevice({ deviceId: "10.0.0.8:5555" });
    const refused = await connection.disconnectDevice({ deviceId: "R58M20ABCDE" });

    expect(connected.outcome).toBe(ConnectionOutcome.CONNECTED);
    expect(again.outcome).toBe(ConnectionOutcome.ALREADY_CONNECTED);
    expect(listed.devices.some((device) => device.id === "10.0.0.8:5555")).toBe(true);
    expect(detached.outcome).toBe(ConnectionOutcome.DISCONNECTED);
    expect(missing.outcome).toBe(ConnectionOutcome.NOT_CONNECTED);
    expect(refused.outcome).toBe(ConnectionOutcome.INVALID_REQUEST);

    const governance = await call.client(GovernanceService).getGovernanceState({});
    expect(governance.auditTrail.map((entry) => [entry.action, entry.outcome])).toEqual([
      [GovernedAction.DISCONNECT_DEVICE, AuditOutcome.FAILED],
      [GovernedAction.DISCONNECT_DEVICE, AuditOutcome.SUCCEEDED],
      [GovernedAction.CONNECT_DEVICE, AuditOutcome.SUCCEEDED],
      [GovernedAction.CONNECT_DEVICE, AuditOutcome.SUCCEEDED],
    ]);
    expect(governance.auditTrail[0].targets).toEqual(["***5555"]);
  });

  it("keeps documents per collection and refuses what the backend store would refuse", async () => {
    const call = mockBackendHarness();
    const documents = call.client(DocumentService);
    const save = (collection: string, id: string, body: string) =>
      documents.saveDocument({ collection, document: { id, body } });

    await save("automation-flows", "b", "{}");
    await save("automation-flows", "a", "[]");
    await expect(save("automation-flows", "c", "{")).rejects.toThrow();
    await expect(save("../escape", "c", "{}")).rejects.toThrow();
    await documents.removeDocument({ collection: "automation-flows", id: "b" });

    const flows = await documents.listDocuments({ collection: "automation-flows" });
    const other = await documents.listDocuments({ collection: "other" });

    expect(flows.documents).toMatchObject([{ id: "a", body: "[]" }]);
    expect(other.documents).toEqual([]);
  });

  it("offers the starter templates the open-source backend ships", async () => {
    const call = mockBackendHarness();
    const catalog = await call.client(TemplateService).listFlowTemplates({});

    expect(catalog.sources.map((source) => source.id)).toEqual(["core.starter"]);
    expect(
      catalog.templates.map((template) => [template.id, template.steps.map((step) => step.kind)]),
    ).toEqual([
      ["install-and-launch", [BatchActionKind.INSTALL_APK, BatchActionKind.LAUNCH_APP]],
      ["capture-screens", [BatchActionKind.SCREENSHOT]],
      ["recover-and-capture", [BatchActionKind.RECOVER_CONNECTION, BatchActionKind.SCREENSHOT]],
    ]);
  });

  it("serves the core connection plans and hides discovered phones once they are attached", async () => {
    const call = mockBackendHarness();
    const snapshot = await call.client(WorkspaceService).getWorkspaceSnapshot({});
    const discovery = call.client(DiscoveryService);
    const before = await discovery.discoverDevices({});
    const ready = before.endpoints.find((endpoint) => !endpoint.requiresPairing)?.endpoint;
    await call.client(ConnectionService).connectDevice({ endpoint: ready });
    const after = await discovery.discoverDevices({});

    expect(snapshot.connectionPlans.map((plan) => plan.id)).toEqual([
      "core.usb",
      "core.wireless",
      "core.otg",
    ]);
    expect(before.endpoints).toHaveLength(2);
    expect(after.endpoints.map((endpoint) => endpoint.requiresPairing)).toEqual([true]);
  });

  it("recognizes online devices by vendor and records health as commands run", async () => {
    const call = mockBackendHarness();
    const vendors = await call.client(VendorService).identifyDevices({
      deviceIds: ["R58M20ABCDE", "RZ8N70XYZ12", "192.168.1.20:5555", "0A1B2C3D4E"],
    });

    await call.client(WorkspaceService).getWorkspaceSnapshot({});
    await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.SCREENSHOT,
      deviceIds: ["0A1B2C3D4E"],
      path: "/tmp",
    });
    const health = await call.client(TelemetryService).getWorkspaceHealth({});

    expect(vendors.devices).toMatchObject([
      { deviceId: "RZ8N70XYZ12", vendorId: "core.samsung" },
      { deviceId: "192.168.1.20:5555", vendorId: "core.xiaomi" },
    ]);
    expect(health.probes[0]).toMatchObject({ devices: 4, online: 3, offline: 1 });
    expect(health.operations).toEqual([
      expect.objectContaining({
        action: GovernedAction.RUN_BATCH_ACTION,
        batchAction: BatchActionKind.SCREENSHOT,
        runs: 1,
        failures: 1,
      }),
    ]);
  });

  it("fails a batch step on devices that are not online", async () => {
    const call = mockBackendHarness();

    const done = await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.SCREENSHOT,
      deviceIds: ["R58M20ABCDE"],
      path: "/tmp",
    });
    const failed = await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.SCREENSHOT,
      deviceIds: ["R58M20ABCDE", "0A1B2C3D4E"],
      path: "/tmp",
    });

    expect(done.task?.status).toBe(TaskStatus.DONE);
    expect(failed.task?.status).toBe(TaskStatus.FAILED);
    expect(failed.deviceResults?.map((result) => result.success)).toEqual([true, false]);
  });

  it("keeps device profiles and groups in the snapshot, like the backend", async () => {
    const call = mockBackendHarness();

    const profiles = call.client(ProfileService);
    const refused = await profiles.updateDeviceProfiles({
      deviceIds: ["R58M20ABCDE"],
      patch: { groupId: "missing" },
    });
    await profiles.saveDeviceGroup({
      group: { id: "shelf-a", name: "  Shelf   A ", tone: GroupTone.SKY },
    });
    await profiles.updateDeviceProfiles({
      deviceIds: ["RZ8N70XYZ12", "R58M20ABCDE"],
      patch: { number: 7, groupId: "shelf-a", addTags: ["TikTok"] },
    });
    const snapshot = await call.client(WorkspaceService).getWorkspaceSnapshot({});
    const numbered = Object.fromEntries(
      snapshot.devices.map((device) => [device.id, device.labels?.number]),
    );

    expect(refused.detail?.key).toBe("workspace.organize.invalid.groupMissing");
    expect(numbered).toMatchObject({ RZ8N70XYZ12: 7, R58M20ABCDE: 8 });
    expect(snapshot.groups.map((group) => [group.id, group.total, group.online])).toEqual([
      ["shelf-a", 2, 2],
      ["ungrouped", 2, 1],
    ]);

    await profiles.removeDeviceGroup({ id: "shelf-a" });
    const after = await call.client(WorkspaceService).getWorkspaceSnapshot({});
    expect(after.groups.map((group) => group.id)).toEqual(["ungrouped"]);
    expect(after.devices.find((device) => device.id === "RZ8N70XYZ12")?.labels?.tags).toEqual([
      "TikTok",
    ]);
  });

  it("routes typing and clipboard writes through the core input channel", async () => {
    const call = mockBackendHarness();

    const channels = await call.client(InputService).listInputChannels({});
    const typed = await call.client(BatchService).runBatchAction({
      kind: BatchActionKind.INPUT_TEXT,
      deviceIds: ["R58M20ABCDE"],
      text: "你好",
    });

    expect(channels.channels.map((channel) => channel.id)).toEqual(["core.yadb"]);
    expect(typed.inputRoutes).toEqual([
      expect.objectContaining({ deviceId: "R58M20ABCDE", channelId: "core.yadb" }),
    ]);
  });
});
