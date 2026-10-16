import { toJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import {
  type AssistantProviderInfoJson,
  type AssistantService,
  type AssistantSettingsJson,
  AssistantSettingsSchema,
  type AssistantStateJson,
  AssistantStateSchema,
  DraftFlowResultSchema,
  DraftScriptResultSchema,
  type FlowJson,
  type ScriptJson,
  ScriptLanguage,
  type UiTextJson,
} from "@/shared/proto";

import { fromMockJson, type MockContext } from "./mock-context";

const PROVIDER_ID = "core.openai-compatible";

function* pieces(answer: unknown) {
  const text = JSON.stringify(answer);
  const half = Math.ceil(text.length / 2);

  yield { event: { case: "text" as const, value: text.slice(0, half) } };
  yield { event: { case: "text" as const, value: text.slice(half) } };
}

export function createMockAssistant(context: MockContext): ServiceImpl<typeof AssistantService> {
  let settings: AssistantSettingsJson = {
    baseUrl: "https://api.openai.com/v1",
    model: "",
    structuredOutput: "STRUCTURED_OUTPUT_JSON_SCHEMA",
  };
  let keySet = false;

  const problem = (): UiTextJson | undefined => {
    if (!/^https?:\/\//.test(settings.baseUrl ?? "")) {
      return { key: "workspace.assistant.invalid.baseUrl" };
    }
    if (!settings.model?.trim()) {
      return { key: "workspace.assistant.invalid.modelMissing" };
    }
    if (!keySet && !/\/\/(localhost|127\.0\.0\.1)/.test(settings.baseUrl ?? "")) {
      return { key: "workspace.assistant.invalid.keyMissing" };
    }
    return undefined;
  };
  const provider = (): AssistantProviderInfoJson => ({
    id: PROVIDER_ID,
    label: { key: "workspace.assistant.openai.label" },
    description: { key: "workspace.assistant.openai.description" },
    configurable: true,
    ready: problem() === undefined,
    problem: problem(),
  });
  const state = (): AssistantStateJson => ({
    providers: [provider()],
    settings: { ...settings, apiKeySet: keySet },
  });
  const refusal = (prompt?: string): UiTextJson | undefined =>
    prompt?.trim() ? problem() : { key: "workspace.assistant.invalid.promptRequired" };
  const used = (key: string) =>
    context.audit("GOVERNED_ACTION_USE_ASSISTANT", true, [], {
      key,
      params: { model: settings.model ?? "" },
    });

  const current = () => fromMockJson(AssistantStateSchema, state());
  return {
    getAssistant: () => context.settle(current()),
    updateAssistant: (request) => {
      if (!/^https?:\/\//.test(request.settings?.baseUrl ?? "")) {
        return context.settle({
          accepted: false,
          detail: { key: "workspace.assistant.invalid.baseUrl" },
          state: current(),
        });
      }
      settings = {
        ...(request.settings ? toJson(AssistantSettingsSchema, request.settings) : {}),
        apiKeySet: false,
      };
      if (request.apiKey !== undefined) {
        keySet = request.apiKey.trim() !== "";
      }
      context.audit("GOVERNED_ACTION_MANAGE_ASSISTANT", true, [], {
        key: "workspace.assistant.audit.updated",
      });
      return context.settle({ accepted: true, state: current() });
    },
    draftFlow: async function* ({ prompt }) {
      await context.settle(undefined);
      const refused = refusal(prompt);

      if (refused) {
        yield {
          event: {
            case: "result",
            value: fromMockJson(DraftFlowResultSchema, { accepted: false, detail: refused }),
          },
        };
        return;
      }
      const capture = crypto.randomUUID();
      const flow = {
        name: prompt.trim().slice(0, 40),
        steps: [
          {
            id: crypto.randomUUID(),
            kind: "BATCH_ACTION_KIND_LAUNCH_APP",
            value: "com.android.settings",
          },
          { id: capture, kind: "BATCH_ACTION_KIND_SCREENSHOT", value: "" },
        ],
      } satisfies FlowJson;
      yield* pieces(flow);
      used("workspace.assistant.audit.flow");
      yield {
        event: {
          case: "result",
          value: fromMockJson(DraftFlowResultSchema, {
            accepted: true,
            draft: {
              flow,
              issues: [{ stepId: capture, message: { key: "workspace.validation.needSavePath" } }],
              providerId: PROVIDER_ID,
              model: settings.model,
            },
          }),
        },
      };
    },
    draftScript: async function* (request) {
      await context.settle(undefined);
      const refused = refusal(request.prompt);

      if (refused) {
        yield {
          event: {
            case: "result",
            value: fromMockJson(DraftScriptResultSchema, { accepted: false, detail: refused }),
          },
        };
        return;
      }
      const javascript = request.language === ScriptLanguage.JAVASCRIPT;
      const script = {
        name: request.prompt.trim().slice(0, 40),
        language: javascript ? "SCRIPT_LANGUAGE_JAVASCRIPT" : "SCRIPT_LANGUAGE_SHELL",
        body: javascript ? 'app.launch("com.android.settings");' : "dumpsys battery | grep level",
      } satisfies ScriptJson;
      yield* pieces({ name: script.name, body: script.body });
      used("workspace.assistant.audit.script");
      yield {
        event: {
          case: "result",
          value: fromMockJson(DraftScriptResultSchema, {
            accepted: true,
            draft: {
              script,
              explanation: javascript ? "Opens Settings." : "Prints the battery level.",
              providerId: PROVIDER_ID,
              model: settings.model,
            },
          }),
        },
      };
    },
  };
}
