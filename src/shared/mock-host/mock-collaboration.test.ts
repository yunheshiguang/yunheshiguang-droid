import { toJsonString } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import {
  AssistantResultSchema,
  AssistantService,
  AuditExportFormat,
  AuditService,
  CloudService,
  ConflictPolicy,
  ConnectionService,
  DocumentService,
  type DraftFlowResult,
  GovernedAction,
  PackageService,
} from "@/shared/proto";

import { mockBackendHarness } from "./mock-harness";

describe("the mock backend's collaboration commands", () => {
  it("keeps a filterable, pageable, exportable audit history with the operator", async () => {
    const call = mockBackendHarness();
    const connection = call.client(ConnectionService);
    for (const host of ["10.0.0.1", "10.0.0.2", "10.0.0.3"]) {
      await connection.connectDevice({ endpoint: { host, port: 5555 } });
    }
    await connection.pairDevice({
      endpoint: { host: "10.0.0.9", port: 37_000 },
      pairingCode: "123456",
    });

    const auditing = call.client(AuditService);
    const first = await auditing.queryAudit({
      query: { actions: [GovernedAction.CONNECT_DEVICE], limit: 2 },
    });
    const rest = await auditing.queryAudit({
      query: { actions: [GovernedAction.CONNECT_DEVICE], limit: 2, beforeId: first.nextBeforeId },
    });
    expect(first.entries).toHaveLength(2);
    expect(first.entries[0].operator?.id).toBe("local:browser");
    expect(rest.entries).toHaveLength(1);
    expect(rest.nextBeforeId).toBe("");

    const exported = await auditing.exportAudit({
      query: {},
      format: AuditExportFormat.CSV,
      path: "/tmp/audit.csv",
    });
    expect(exported).toMatchObject({ accepted: true, exported: 4 });
    const latest = await auditing.queryAudit({ query: {} });
    expect(latest.entries[0].action).toBe(GovernedAction.EXPORT_AUDIT);
  });

  it("drafts only once the assistant is configured, and never hands the key back", async () => {
    const call = mockBackendHarness();
    const assistant = call.client(AssistantService);
    const draft = async (prompt: string) => {
      let outcome: DraftFlowResult | undefined;
      for await (const { event } of assistant.draftFlow({ prompt })) {
        if (event.case === "result") {
          outcome = event.value;
        }
      }
      return outcome;
    };
    const refused = await draft("open settings");
    expect(refused?.detail?.key).toBe("workspace.assistant.invalid.modelMissing");

    const saved = await assistant.updateAssistant({
      settings: { baseUrl: "https://api.deepseek.com/v1", model: "deepseek-chat" },
      apiKey: "sk-secret",
    });
    expect(saved.state?.settings?.apiKeySet).toBe(true);
    expect(toJsonString(AssistantResultSchema, saved)).not.toContain("sk-secret");
    const state = await assistant.getAssistant({});
    expect(state.providers[0].ready).toBe(true);

    const drafted = await draft("open settings");
    expect(drafted?.draft?.flow?.steps).toHaveLength(2);
    expect(drafted?.draft?.issues[0].message?.key).toBe("workspace.validation.needSavePath");
  });

  it("exports flows with their scripts and installs them back beside the originals", async () => {
    const call = mockBackendHarness();
    const documents = call.client(DocumentService);
    await documents.saveDocument({
      collection: "scripts",
      document: { id: "battery", body: JSON.stringify({ name: "Battery", body: "dumpsys" }) },
    });
    await documents.saveDocument({
      collection: "automation-flows",
      document: {
        id: "nightly",
        body: JSON.stringify({
          name: "Nightly",
          steps: [{ id: "s1", kind: "BATCH_ACTION_KIND_RUN_SCRIPT", value: "battery" }],
        }),
      },
    });
    const packages = call.client(PackageService);
    const path = "/tmp/nightly.json";
    const origin = { origin: { case: "path" as const, value: path } };

    const exported = await packages.exportPackage({
      manifest: { id: "com.example.nightly", name: "Nightly", version: "1.0.0" },
      flowIds: ["nightly"],
      path,
    });
    expect(exported).toMatchObject({ accepted: true, flows: 1, scripts: 1 });

    const inspected = await packages.inspectPackage({ origin });
    expect(inspected.preview?.flows[0]).toMatchObject({ id: "nightly", exists: true });

    const installed = await packages.installPackage({
      origin,
      onConflict: ConflictPolicy.KEEP_BOTH,
    });
    expect(installed).toMatchObject({ accepted: true, flows: 1, scripts: 1 });
    const flows = await documents.listDocuments({ collection: "automation-flows" });
    expect(flows.documents.map((document) => document.id)).toEqual(["nightly", "nightly-2"]);
  });

  it("registers cloud phones by address once and removes them", async () => {
    const call = mockBackendHarness();
    const cloud = call.client(CloudService);
    const registered = await cloud.registerCloudPhone({
      name: "Cloud 01",
      endpoint: { host: "10.20.0.11", port: 5555 },
    });
    const again = await cloud.registerCloudPhone({
      name: "Cloud 01 again",
      endpoint: { host: "10.20.0.11", port: 5555 },
    });
    const started = await cloud.startCloudPhone({
      providerId: "core.registered",
      phoneId: registered.phone?.id,
    });

    expect(registered.accepted).toBe(true);
    expect(again.detail?.key).toBe("workspace.cloud.alreadyRegistered");
    expect(started.detail?.key).toBe("workspace.cloud.unsupported");

    await cloud.removeCloudPhone({ providerId: "core.registered", phoneId: registered.phone?.id });
    const listed = await cloud.listCloudPhones({});
    expect(listed.phones).toEqual([]);
    expect(listed.providers[0].canRegister).toBe(true);
  });
});
