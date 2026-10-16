import { fromJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { assistantMessages } from "@/entities/assistant";
import { taskMessages } from "@/entities/task";
import { AssistantService, type DraftFlowResultJson, DraftFlowResultSchema } from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { editAutomationFlowMessages } from "../messages";
import { FlowEditor } from "./flow-editor";

const ready = {
  providers: [{ id: "core.openai-compatible", ready: true }],
  settings: { baseUrl: "https://api.openai.com/v1", model: "gpt" },
};

const drafted: DraftFlowResultJson = {
  accepted: true,
  draft: {
    flow: {
      name: "Morning check",
      steps: [
        { id: "s1", kind: "BATCH_ACTION_KIND_LAUNCH_APP", value: "com.android.settings" },
        { id: "s2", kind: "BATCH_ACTION_KIND_SCREENSHOT", value: "" },
      ],
    },
    issues: [{ stepId: "s2", message: { key: "workspace.validation.needSavePath" } }],
    providerId: "core.openai-compatible",
    model: "gpt-test",
  },
};

type DraftFlow = ServiceImpl<typeof AssistantService>["draftFlow"];

const text = (value: string) => ({ event: { case: "text" as const, value } });

const result = (json: DraftFlowResultJson) => ({
  event: { case: "result" as const, value: fromJson(DraftFlowResultSchema, json) },
});

function renderEditor(draftFlow: DraftFlow, documents = createFakeDocuments()) {
  installFakeServices(documents.routes, (router) =>
    router.service(AssistantService, { getAssistant: () => ready, draftFlow }),
  );
  const { wrapper } = createWorkspaceTestHarness(
    taskMessages,
    editAutomationFlowMessages,
    assistantMessages,
  );

  render(<FlowEditor />, { wrapper });
}

describe("FlowEditor with the AI assistant", () => {
  it("counts the steps the AI has written while it is still writing", async () => {
    renderEditor(async function* () {
      yield text('{"name":"Morning","steps":[{"kind":"BATCH_ACTION_KIND_LAUNCH_APP","value":"a"},');
      yield text('{"kind":"BATCH_ACTION_KIND_SCREENSH');
      await new Promise<never>(() => {});
    });

    await userEvent.type(await screen.findByLabelText("What to do"), "Open settings");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));

    expect(await screen.findByText("2 steps drafted so far…")).toBeInTheDocument();
  });

  it("opens the drafted flow in the editor and lists what needs attention", async () => {
    const prompts: string[] = [];
    renderEditor(async function* (request) {
      prompts.push(request.prompt);
      yield result(drafted);
    });

    await userEvent.type(
      await screen.findByLabelText("What to do"),
      "Open settings and take a screenshot",
    );
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));

    expect(prompts).toEqual(["Open settings and take a screenshot"]);
    expect(await screen.findByDisplayValue("Morning check")).toBeInTheDocument();
    expect(screen.getByDisplayValue("com.android.settings")).toBeInTheDocument();
    expect(screen.getByText(/Drafted 2 steps/)).toHaveTextContent("Drafted by gpt-test");
    expect(screen.getByRole("list", { name: "Needs attention" })).toHaveTextContent(
      "Step 2: Choose where to save",
    );
  });

  it("stops drafting when asked and tells the assistant to stop", async () => {
    let stopped = false;
    renderEditor(async function* (_request, context) {
      yield text('{"name":"Morning","steps":[');
      await new Promise<void>((resolve) => {
        context.signal.addEventListener("abort", () => {
          stopped = true;
          resolve();
        });
      });
    });

    await userEvent.type(await screen.findByLabelText("What to do"), "Open settings");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));
    await userEvent.click(await screen.findByRole("button", { name: "Stop" }));

    expect(await screen.findByRole("button", { name: "Draft" })).toBeEnabled();
    expect(stopped).toBe(true);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("leaves a flow opened while the AI was writing alone", async () => {
    const documents = createFakeDocuments();
    documents.save(
      "automation-flows",
      "evening",
      JSON.stringify({
        name: "Evening",
        steps: [{ id: "a", kind: "BATCH_ACTION_KIND_LAUNCH_APP", value: "com.android.settings" }],
      }),
    );
    let release = () => {};
    const released = new Promise<void>((resolve) => {
      release = resolve;
    });
    renderEditor(async function* () {
      await released;
      yield result(drafted);
    }, documents);

    await userEvent.type(await screen.findByLabelText("What to do"), "Open settings");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));
    await userEvent.click(await screen.findByRole("button", { name: "Evening" }));
    release();

    expect(await screen.findByText(/Drafted 2 steps/)).toBeInTheDocument();
    expect(screen.getByDisplayValue("Evening")).toBeInTheDocument();
    expect(screen.queryByDisplayValue("Morning check")).not.toBeInTheDocument();
  });

  it("says why the model could not draft", async () => {
    renderEditor(async function* () {
      yield result({
        accepted: false,
        detail: { key: "workspace.assistant.requestFailed", params: { message: "HTTP 401" } },
      });
    });

    await userEvent.type(await screen.findByLabelText("What to do"), "anything");
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The AI service request failed: HTTP 401",
    );
  });
});
