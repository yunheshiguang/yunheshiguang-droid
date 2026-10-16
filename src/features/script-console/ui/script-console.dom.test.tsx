import { create, fromJson, type JsonObject, type JsonValue, toJson } from "@bufbuild/protobuf";
import type { ConnectRouter, ServiceImpl } from "@connectrpc/connect";
import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { assistantMessages } from "@/entities/assistant";
import type { DeviceRecord } from "@/entities/device";
import { scriptMessages } from "@/entities/script";
import { setCommandGateway } from "@/shared/api";
import {
  AssistantService,
  BatchActionRequestSchema,
  BatchActionResultSchema,
  BatchService,
  ConnectionHealth,
  ConnectionType,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  DocumentService,
  DraftScriptResultSchema,
  PiiClass,
  ScriptEnginesSchema,
  ScriptLanguage,
  ScriptService,
  SearchableTextSchema,
  VisibilityMode,
} from "@/shared/proto";
import {
  createFakeCommandGateway,
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";
import { ThemeProvider } from "@/shared/theme";
import type { CodeEditorProps } from "@/shared/ui/code-editor";

import { scriptConsoleMessages } from "../messages";
import { ScriptConsole } from "./script-console";
import { ScriptEngineList } from "./script-engine-list";

vi.mock("@/shared/ui", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/shared/ui")>()),
  CodeEditor: ({ value, onChange, label }: CodeEditorProps) => (
    <textarea aria-label={label} value={value} onChange={(event) => onChange(event.target.value)} />
  ),
}));

const phone: DeviceRecord = create(DeviceRecordSchema, {
  id: "serial-a",
  name: create(SearchableTextSchema, {
    displayValue: "serial-a",
    canonicalValue: "serial-a",
    aliases: [],
    redactedValue: "Pixel 8",
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

function scriptResult(scriptId: string) {
  return {
    accepted: true,
    task: {
      id: "batch-run-script-1",
      status: "TASK_STATUS_DONE",
      title: { key: "workspace.task.title.runScript" },
      detail: { key: "workspace.task.detail.completed", params: { count: "1" } },
      actionKind: "BATCH_ACTION_KIND_RUN_SCRIPT",
    },
    deviceResults: [{ deviceId: "serial-a", success: true }],
    scriptOutputs: [
      {
        deviceId: "serial-a",
        engineId: "core.shell",
        engine: { key: "workspace.scripts.shell.label" },
        exitCode: 0,
        output: `ran ${scriptId.length > 0 ? "script" : ""}\nhi`,
      },
    ],
  };
}

function serveReadyAssistant(draftScript: ServiceImpl<typeof AssistantService>["draftScript"]) {
  installFakeServices((router) =>
    router.service(AssistantService, {
      getAssistant: () => ({
        providers: [{ id: "core.openai-compatible", ready: true }],
        settings: { baseUrl: "https://api.openai.com/v1", model: "gpt" },
      }),
      draftScript,
    }),
  );
}

function renderConsole(
  handlers: Parameters<typeof createFakeCommandGateway>[0],
  documents: (router: ConnectRouter) => void = createFakeDocuments().routes,
) {
  installFakeServices(documents);
  setCommandGateway(createFakeCommandGateway(handlers));
  const { wrapper } = createWorkspaceTestHarness(
    scriptMessages,
    scriptConsoleMessages,
    assistantMessages,
  );

  render(
    <ThemeProvider>
      <ScriptConsole selectedDevices={[phone]} devices={[phone]} />
      <ScriptEngineList />
    </ThemeProvider>,
    { wrapper },
  );
}

describe("ScriptConsole", () => {
  it("saves a new script, runs it on the selected devices, and shows each device's output", async () => {
    const run = vi.fn((payload: { request: JsonObject }) =>
      scriptResult(String(payload.request.scriptId ?? "")),
    );
    installFakeServices((router) =>
      router.service(ScriptService, {
        listScriptEngines: () =>
          create(ScriptEnginesSchema, {
            engines: [
              {
                id: "core.shell",
                label: { key: "workspace.scripts.shell.label" },
                languages: [ScriptLanguage.SHELL],
                capturesOutput: true,
              },
            ],
          }),
      }),
    );
    installFakeServices((router) =>
      router.service(BatchService, {
        runBatchAction: (request) =>
          fromJson(
            BatchActionResultSchema,
            run({ request: toJson(BatchActionRequestSchema, request) as JsonObject }) as JsonValue,
          ),
      }),
    );
    renderConsole({});

    await userEvent.click(screen.getByRole("button", { name: "Shell" }));
    const editor = screen.getByRole("textbox", { name: "Script" });
    expect(editor).toHaveValue("getprop ro.product.model\n");
    await userEvent.clear(editor);
    await userEvent.type(editor, "echo hi");
    expect(screen.getByText("Unsaved changes. Running saves them first")).toBeVisible();

    await userEvent.click(screen.getByRole("button", { name: "Run" }));

    const request = run.mock.calls[0]?.[0]?.request ?? {};
    expect(request).toMatchObject({
      kind: "BATCH_ACTION_KIND_RUN_SCRIPT",
      deviceIds: ["serial-a"],
    });
    expect(request.scriptId).toMatch(/^[0-9a-f-]{36}$/);

    const results = await screen.findByRole("region", { name: "Run results" });
    expect(within(results).getByText("Exit code 0")).toBeVisible();
    expect(within(results).getByText("Run by Device shell (built-in)")).toBeVisible();
    expect(within(results).getByRole("figure", { name: "Output from Pixel 8" })).toHaveTextContent(
      "hi",
    );
    expect(await screen.findByRole("button", { name: /New script/ })).toBeVisible();
    expect(screen.queryByText("Unsaved changes. Running saves them first")).toBeNull();
    expect(await screen.findByText("Returns output and exit code", { exact: false })).toBeVisible();
  });

  it("runs an edited script once however quickly Run is pressed again", async () => {
    let runs = 0;
    installFakeServices((router) =>
      router.service(BatchService, {
        runBatchAction: (request) => {
          runs += 1;
          return fromJson(BatchActionResultSchema, scriptResult(request.scriptId) as JsonValue);
        },
      }),
    );
    let release = () => {};
    const saving = new Promise<void>((resolve) => {
      release = resolve;
    });
    const documents = createFakeDocuments();
    renderConsole({}, (router) =>
      router.service(DocumentService, {
        ...documents.service,
        saveDocument: async (request, context) => {
          await saving;
          return documents.service.saveDocument(request, context);
        },
      }),
    );

    await userEvent.click(screen.getByRole("button", { name: "Shell" }));
    const runButton = screen.getByRole("button", { name: "Run" });
    await userEvent.click(runButton);
    await userEvent.click(runButton);
    release();

    await screen.findByRole("region", { name: "Run results" });
    expect(runs).toBe(1);
  });

  it("shows the script while the AI is still writing it", async () => {
    serveReadyAssistant(async function* () {
      yield { event: { case: "text", value: '{"name":"Battery","body":"dumpsys ' } };
      yield { event: { case: "text", value: "battery | grep le" } };
      await new Promise<never>(() => {});
    });
    renderConsole({});

    await userEvent.type(await screen.findByLabelText("Draft a script with AI"), "battery");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));

    expect(await screen.findByLabelText("Script being drafted by AI")).toHaveTextContent(
      "dumpsys battery | grep le",
    );
  });

  it("counts a script as saved once its name has been tidied up", async () => {
    renderConsole({});

    await userEvent.click(screen.getByRole("button", { name: "Shell" }));
    await userEvent.clear(screen.getByLabelText("Script name"));
    await userEvent.type(screen.getByLabelText("Script name"), "  Battery  ");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("button", { name: /Battery/ })).toBeVisible();
    expect(screen.getByLabelText("Script name")).toHaveValue("Battery");
    expect(screen.queryByText("Unsaved changes. Running saves them first")).toBeNull();
  });

  it("puts an AI draft only into the script it was written for", async () => {
    let finish: (value: JsonValue) => void = () => {};
    serveReadyAssistant(async function* () {
      const answer = await new Promise<JsonValue>((resolve) => {
        finish = resolve;
      });
      yield { event: { case: "result", value: fromJson(DraftScriptResultSchema, answer) } };
    });
    renderConsole({});

    await userEvent.click(screen.getByRole("button", { name: "Shell" }));
    await userEvent.clear(screen.getByLabelText("Script name"));
    await userEvent.type(screen.getByLabelText("Script name"), "First");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    const first = await screen.findByRole("button", { name: /First/ });

    await userEvent.click(screen.getByRole("button", { name: "JavaScript" }));
    await userEvent.type(await screen.findByLabelText("Edit this script with AI"), "battery");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));
    await userEvent.click(first);
    await act(async () => {
      finish({
        accepted: true,
        draft: {
          script: {
            name: "Battery",
            language: "SCRIPT_LANGUAGE_JAVASCRIPT",
            body: "toast(battery());",
          },
          explanation: "",
          model: "",
        },
      });
    });

    expect(screen.getByLabelText("Script name")).toHaveValue("First");
    expect(screen.getByRole("textbox", { name: "Script" })).toHaveValue(
      "getprop ro.product.model\n",
    );
  });
});
