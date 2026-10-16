import { toJson } from "@bufbuild/protobuf";
import type { ConnectRouter } from "@connectrpc/connect";

import {
  EXPECTED_SCHEMA_VERSION,
  WORKSPACE_COMMANDS,
  type WorkspaceCommandName,
} from "@/shared/api";
import type {
  AndroidPlatformToolsStateJson,
  BatchActionKindJson,
  BatchActionRequestJson,
  BatchActionResultJson,
  ConnectDeviceRequestJson,
  ConnectionOutcomeJson,
  ConnectionResultJson,
  DeviceRecordJson,
  DisconnectDeviceRequestJson,
  GovernedActionJson,
  OrganizeResultJson,
  PairDeviceRequestJson,
  UiTextJson,
  WorkspaceSnapshotJson,
} from "@/shared/proto";
import {
  AndroidPlatformToolsConfigSchema,
  AndroidPlatformToolsStateSchema,
  ApiGatewayService,
  AssistantService,
  AuditService,
  AutomationService,
  BatchActionRequestSchema,
  BatchActionResultSchema,
  BatchService,
  CloudService,
  ConnectDeviceRequestSchema,
  ConnectionResultSchema,
  ConnectionService,
  DeviceDiscoverySchema,
  DeviceVendorsSchema,
  DisconnectDeviceRequestSchema,
  DiscoveryService,
  DocumentService,
  FlowTemplateCatalogSchema,
  GovernanceService,
  InputChannelsSchema,
  InputService,
  PackageService,
  PairDeviceRequestSchema,
  PlatformToolsService,
  ProfileService,
  ScrcpyService,
  ScriptEnginesSchema,
  ScriptService,
  TelemetryService,
  TemplateService,
  UpdateService,
  VendorService,
  WorkspaceHealthSchema,
  WorkspaceService,
  WorkspaceSnapshotSchema,
} from "@/shared/proto";

import {
  initialMockDevices,
  MOCK_ADB_PATH,
  MOCK_FASTBOOT_PATH,
  mockDevice,
  mockPlatformTools,
} from "./fixtures";
import { createMockApiGateway } from "./mock-api";
import { createMockAssistant } from "./mock-assistant";
import { createMockAutomation } from "./mock-automation";
import { createMockCloud } from "./mock-cloud";
import { fromMockJson, type MockCommandHandler, type MockContext } from "./mock-context";
import { createMockDocuments } from "./mock-documents";
import { createMockGovernance } from "./mock-governance";
import { createMockHealth } from "./mock-health";
import { mockInputChannels, mockInputRoutes } from "./mock-input";
import { createMockPackages } from "./mock-packages";
import { createMockProfileService, createMockProfiles } from "./mock-profiles";
import { createMockScrcpy } from "./mock-scrcpy";
import { mockScriptEngines, mockScriptOutputs, readMockScript } from "./mock-scripts";
import { starterTemplateCatalog } from "./mock-templates";
import { mockConnectionPlans, mockDiscovery } from "./mock-transports";
import { createMockUpdates } from "./mock-updates";
import { identifyMockDevices } from "./mock-vendors";

export type MockBackendOptions = {
  latencyMs?: number;
};

export type MockBackend = {
  handlers: Readonly<Record<WorkspaceCommandName, MockCommandHandler>>;
  services: (router: ConnectRouter) => void;
};

const SUCCEEDED_CONNECTIONS = new Set([
  "CONNECTION_OUTCOME_CONNECTED",
  "CONNECTION_OUTCOME_ALREADY_CONNECTED",
  "CONNECTION_OUTCOME_PAIRED",
  "CONNECTION_OUTCOME_DISCONNECTED",
]);

const TASK_TITLE_KEYS = {
  BATCH_ACTION_KIND_UNSPECIFIED: "workspace.task.title.default",
  BATCH_ACTION_KIND_SCREENSHOT: "workspace.task.title.screenshot",
  BATCH_ACTION_KIND_SCREEN_RECORD: "workspace.task.title.screenRecord",
  BATCH_ACTION_KIND_INSTALL_APK: "workspace.task.title.installApk",
  BATCH_ACTION_KIND_UPLOAD_FILE: "workspace.task.title.uploadFile",
  BATCH_ACTION_KIND_INPUT_TEXT: "workspace.task.title.inputText",
  BATCH_ACTION_KIND_LAUNCH_APP: "workspace.task.title.launchApp",
  BATCH_ACTION_KIND_CLIPBOARD_WRITE: "workspace.task.title.clipboardWrite",
  BATCH_ACTION_KIND_RECOVER_CONNECTION: "workspace.task.title.recoverConnection",
  BATCH_ACTION_KIND_RUN_SCRIPT: "workspace.task.title.runScript",
  BATCH_ACTION_KIND_STOP_APP: "workspace.task.title.stopApp",
  BATCH_ACTION_KIND_CLEAR_APP_DATA: "workspace.task.title.clearAppData",
  BATCH_ACTION_KIND_UNINSTALL_APP: "workspace.task.title.uninstallApp",
} as const satisfies Record<BatchActionKindJson, string>;

function connectionResult(
  outcome: ConnectionOutcomeJson,
  deviceId: string,
  key: string,
  params?: Record<string, string>,
): ConnectionResultJson {
  return { outcome, deviceId, detail: { key, params } };
}

export function createMockBackend({ latencyMs = 250 }: MockBackendOptions = {}): MockBackend {
  let devices: DeviceRecordJson[] = initialMockDevices();
  let platformTools: AndroidPlatformToolsStateJson = mockPlatformTools();
  const documents = createMockDocuments();
  const health = createMockHealth(latencyMs);
  const profiles = createMockProfiles();
  let sequence = 0;

  const settle = <T>(value: T): Promise<T> =>
    new Promise((resolve) => setTimeout(() => resolve(value), latencyMs));
  const governance = createMockGovernance(settle);

  const audit = (
    action: GovernedActionJson,
    succeeded: boolean,
    targets: readonly string[],
    detail?: UiTextJson,
  ) => {
    sequence += 1;
    governance.audit(action, succeeded, targets, detail);
  };

  const snapshot = (): WorkspaceSnapshotJson => {
    const labeled = devices.map(profiles.annotate);

    return {
      schemaVersion: EXPECTED_SCHEMA_VERSION,
      locale: "en-US",
      hostPlatform: "browser",
      devices: labeled,
      connectionPlans: mockConnectionPlans(),
      groups: profiles.summary(labeled),
      adbPath: MOCK_ADB_PATH,
      fastbootPath: MOCK_FASTBOOT_PATH,
      androidPlatformTools: platformTools,
    };
  };

  const organized = (
    result: OrganizeResultJson,
    targets: readonly string[],
    detail: UiTextJson,
  ): OrganizeResultJson => {
    audit("GOVERNED_ACTION_ORGANIZE_DEVICES", Boolean(result.accepted), targets, detail);
    health.operation("GOVERNED_ACTION_ORGANIZE_DEVICES", Boolean(result.accepted));
    return result;
  };

  const runBatch = (request: BatchActionRequestJson): BatchActionResultJson => {
    const kind = request.kind ?? "BATCH_ACTION_KIND_UNSPECIFIED";
    const ids = request.deviceIds ?? [];
    const script =
      kind === "BATCH_ACTION_KIND_RUN_SCRIPT"
        ? readMockScript(
            documents
              .list({ collection: "scripts" })
              .documents?.find((document) => document.id === request.scriptId)?.body,
          )
        : undefined;

    if (kind === "BATCH_ACTION_KIND_RUN_SCRIPT" && !script) {
      const detail = { key: "workspace.scripts.missing" };

      return {
        accepted: false,
        task: {
          id: `batch-${kind}-${sequence}`,
          title: { key: TASK_TITLE_KEYS[kind] },
          status: "TASK_STATUS_BLOCKED",
          detail,
          actionKind: kind,
        },
        diagnostics: [
          {
            id: "batch-diagnostic-0",
            code: "DIAGNOSTIC_CODE_ANDROID_SECURITY_LIMIT",
            impact: detail,
          },
        ],
      };
    }

    const deviceResults = ids.map((id) => {
      const online = devices.some(
        (device) => device.id === id && device.status === "DEVICE_STATUS_ONLINE",
      );

      return online
        ? { deviceId: id, success: true }
        : {
            deviceId: id,
            success: false,
            errorKind: "ADB_ERROR_KIND_NON_ZERO_EXIT" as const,
            message: "device offline",
          };
    });
    const done = ids.length > 0 && deviceResults.every((result) => result.success);
    const title = { key: TASK_TITLE_KEYS[kind] };
    const count = String(ids.length);

    audit("GOVERNED_ACTION_RUN_BATCH_ACTION", done, ids, title);

    return {
      accepted: done,
      task: {
        id: `batch-${kind}-${sequence}`,
        title,
        target: { key: "workspace.task.target.count", params: { count } },
        status: done ? "TASK_STATUS_DONE" : "TASK_STATUS_FAILED",
        progress: 100,
        detail: done
          ? { key: "workspace.task.detail.completed", params: { count } }
          : {
              key: "workspace.diagnostic.impact.runtime",
              params: { message: "device offline" },
            },
        actionKind: kind,
        concurrencyLimit: 4,
        retryCount: 0,
      },
      deviceResults,
      inputRoutes: mockInputRoutes(kind, deviceResults),
      scriptOutputs: script ? mockScriptOutputs(script, deviceResults) : [],
    };
  };

  const context: MockContext = {
    latencyMs,
    settle,
    audit,
    measure: (action, succeeded, kind) => health.operation(action, succeeded, kind),
    documents,
    runBatch,
    devices: () => devices.map(profiles.annotate),
  };

  const connect = ({ endpoint }: ConnectDeviceRequestJson): ConnectionResultJson => {
    const host = endpoint?.host?.trim() ?? "";
    const port = endpoint?.port ?? 0;

    if (!host) {
      return connectionResult(
        "CONNECTION_OUTCOME_INVALID_REQUEST",
        "",
        "workspace.connection.result.invalidHost",
      );
    }

    if (port < 1 || port > 65_535) {
      return connectionResult(
        "CONNECTION_OUTCOME_INVALID_REQUEST",
        "",
        "workspace.connection.result.invalidPort",
      );
    }

    const id = `${host}:${port}`;
    const known = devices.some((device) => device.id === id);

    if (!known) {
      devices = [...devices, mockDevice(id, "Android device", "CONNECTION_TYPE_WIFI")];
    }

    audit("GOVERNED_ACTION_CONNECT_DEVICE", true, [id]);

    return known
      ? connectionResult(
          "CONNECTION_OUTCOME_ALREADY_CONNECTED",
          id,
          "workspace.connection.result.alreadyConnected",
          { device: id },
        )
      : connectionResult(
          "CONNECTION_OUTCOME_CONNECTED",
          id,
          "workspace.connection.result.connected",
          { device: id },
        );
  };

  const pair = ({ endpoint, pairingCode }: PairDeviceRequestJson): ConnectionResultJson => {
    const target = `${endpoint?.host ?? ""}:${endpoint?.port ?? 0}`;

    if (!/^\d{6}$/.test(pairingCode?.trim() ?? "")) {
      return connectionResult(
        "CONNECTION_OUTCOME_INVALID_REQUEST",
        "",
        "workspace.connection.result.invalidPairingCode",
      );
    }

    audit("GOVERNED_ACTION_PAIR_DEVICE", true, [target]);

    return connectionResult("CONNECTION_OUTCOME_PAIRED", "", "workspace.connection.result.paired", {
      endpoint: target,
    });
  };

  const disconnect = ({ deviceId = "" }: DisconnectDeviceRequestJson): ConnectionResultJson => {
    if (!/:\d+$/.test(deviceId)) {
      return connectionResult(
        "CONNECTION_OUTCOME_INVALID_REQUEST",
        "",
        "workspace.connection.result.invalidDevice",
      );
    }

    const known = devices.some((device) => device.id === deviceId);

    devices = devices.filter((device) => device.id !== deviceId);
    audit("GOVERNED_ACTION_DISCONNECT_DEVICE", known, [deviceId]);

    return known
      ? connectionResult(
          "CONNECTION_OUTCOME_DISCONNECTED",
          deviceId,
          "workspace.connection.result.disconnected",
          { device: deviceId },
        )
      : connectionResult(
          "CONNECTION_OUTCOME_NOT_CONNECTED",
          deviceId,
          "workspace.connection.result.notConnected",
          { device: deviceId },
        );
  };

  const measured = (action: GovernedActionJson, result: ConnectionResultJson) => {
    if (
      result.outcome !== "CONNECTION_OUTCOME_DENIED" &&
      result.outcome !== "CONNECTION_OUTCOME_INVALID_REQUEST"
    ) {
      health.operation(action, SUCCEEDED_CONNECTIONS.has(result.outcome ?? ""));
    }

    return result;
  };

  const assistant = createMockAssistant(context);
  const handlers = {
    [WORKSPACE_COMMANDS.revealLogDirectory]: () => settle(undefined),
  } satisfies Record<WorkspaceCommandName, MockCommandHandler>;

  const apiGateway = createMockApiGateway(context);
  const automation = createMockAutomation(context);
  const cloud = createMockCloud(context);
  const packages = createMockPackages(context);
  const scrcpy = createMockScrcpy(context);
  const services = (router: ConnectRouter) => {
    router.service(WorkspaceService, {
      getWorkspaceSnapshot: async () => {
        health.probe(devices);
        return fromMockJson(WorkspaceSnapshotSchema, await settle(snapshot()));
      },
    });
    router.service(PlatformToolsService, {
      updateAndroidPlatformToolsConfig: async ({ config }) => {
        platformTools = {
          ...platformTools,
          config: config ? toJson(AndroidPlatformToolsConfigSchema, config) : platformTools.config,
        };
        audit("GOVERNED_ACTION_UPDATE_PLATFORM_TOOLS_CONFIG", true, []);

        return fromMockJson(AndroidPlatformToolsStateSchema, await settle(platformTools));
      },
      installAndroidPlatformTools: async () => {
        const install = {
          status: "ANDROID_PLATFORM_TOOLS_INSTALL_STATUS_INSTALLED" as const,
          platformToolsRoot: "/mock/platform-tools",
          installedAt: new Date().toISOString(),
        };

        platformTools = { ...platformTools, install, lastInstallAttempt: install };
        audit("GOVERNED_ACTION_INSTALL_PLATFORM_TOOLS", true, []);

        return fromMockJson(AndroidPlatformToolsStateSchema, await settle(platformTools));
      },
    });
    const answer = async (action: GovernedActionJson, result: ConnectionResultJson) =>
      fromMockJson(ConnectionResultSchema, await settle(measured(action, result)));
    router.service(ConnectionService, {
      connectDevice: (request) =>
        answer(
          "GOVERNED_ACTION_CONNECT_DEVICE",
          connect(toJson(ConnectDeviceRequestSchema, request)),
        ),
      pairDevice: (request) =>
        answer("GOVERNED_ACTION_PAIR_DEVICE", pair(toJson(PairDeviceRequestSchema, request))),
      disconnectDevice: (request) =>
        answer(
          "GOVERNED_ACTION_DISCONNECT_DEVICE",
          disconnect(toJson(DisconnectDeviceRequestSchema, request)),
        ),
    });
    router.service(DiscoveryService, {
      discoverDevices: async () =>
        fromMockJson(
          DeviceDiscoverySchema,
          await settle(mockDiscovery(new Set(devices.map((device) => device.id ?? "")))),
        ),
    });
    router.service(ProfileService, createMockProfileService(profiles, organized, settle));
    router.service(BatchService, {
      runBatchAction: async (message) => {
        const request = toJson(BatchActionRequestSchema, message);
        const result = runBatch(request);

        health.operation(
          "GOVERNED_ACTION_RUN_BATCH_ACTION",
          result.task?.status === "TASK_STATUS_DONE",
          request.kind,
        );
        return fromMockJson(BatchActionResultSchema, await settle(result));
      },
    });
    router.service(AutomationService, automation);
    router.service(TemplateService, {
      listFlowTemplates: async () =>
        fromMockJson(FlowTemplateCatalogSchema, await settle(starterTemplateCatalog())),
    });
    router.service(ApiGatewayService, apiGateway);
    router.service(AssistantService, assistant);
    router.service(AuditService, governance.auditing);
    router.service(CloudService, cloud);
    router.service(DocumentService, documents.service);
    router.service(GovernanceService, governance.governance);
    router.service(PackageService, packages);
    router.service(ScrcpyService, scrcpy);
    router.service(InputService, {
      listInputChannels: async () =>
        fromMockJson(InputChannelsSchema, await settle(mockInputChannels())),
    });
    router.service(ScriptService, {
      listScriptEngines: async () =>
        fromMockJson(ScriptEnginesSchema, await settle(mockScriptEngines())),
    });
    router.service(VendorService, {
      identifyDevices: async ({ deviceIds }) =>
        fromMockJson(DeviceVendorsSchema, await settle(identifyMockDevices(deviceIds, devices))),
    });
    router.service(TelemetryService, {
      getWorkspaceHealth: async () =>
        fromMockJson(WorkspaceHealthSchema, await settle(health.health())),
    });
    router.service(UpdateService, createMockUpdates(context));
  };

  return { handlers, services };
}
