import type { MessageInitShape } from "@bufbuild/protobuf";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import {
  AssistantService,
  type AssistantStateSchema,
  StructuredOutput,
  type UpdateAssistantRequest,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { assistantSettingsMessages } from "../messages";
import { AssistantSettingsPanel } from "./assistant-settings-panel";

type State = MessageInitShape<typeof AssistantStateSchema>;

const unconfigured = {
  providers: [
    {
      id: "core.openai-compatible",
      label: { key: "workspace.assistant.openai.label" },
      configurable: true,
      ready: false,
      problem: { key: "workspace.assistant.invalid.modelMissing" },
    },
  ],
  settings: {
    baseUrl: "https://api.openai.com/v1",
    structuredOutput: StructuredOutput.JSON_SCHEMA,
  },
} satisfies State;

const configured: State = {
  providers: [{ ...unconfigured.providers[0], ready: true, problem: undefined }],
  settings: {
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    apiKeySet: true,
    structuredOutput: StructuredOutput.JSON_OBJECT,
  },
};

describe("AssistantSettingsPanel", () => {
  it("fills a preset, sends the key once, and never shows it again", async () => {
    let current: State = unconfigured;
    const update = vi.fn((_request: UpdateAssistantRequest) => {
      current = configured;
      return { accepted: true, state: configured };
    });
    installFakeServices((router) =>
      router.service(AssistantService, { getAssistant: () => current, updateAssistant: update }),
    );
    const { wrapper } = createWorkspaceTestHarness(assistantSettingsMessages);
    render(<AssistantSettingsPanel />, { wrapper });

    expect(await screen.findByText("Enter a model name")).toBeInTheDocument();
    expect(screen.getByText("Not ready")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "DeepSeek" }));
    expect(screen.getByLabelText("Base URL")).toHaveValue("https://api.deepseek.com/v1");
    expect(screen.getByLabelText("Model")).toHaveValue("deepseek-chat");
    await userEvent.type(screen.getByLabelText("API key"), "sk-secret");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(update.mock.calls[0]?.[0]).toMatchObject({
      apiKey: "sk-secret",
      settings: {
        baseUrl: "https://api.deepseek.com/v1",
        model: "deepseek-chat",
        structuredOutput: StructuredOutput.JSON_OBJECT,
      },
    });
    expect(await screen.findByText("Ready")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Saved");
    const key = screen.getByLabelText("API key");
    expect(key).toHaveValue("");
    expect(key).toHaveAttribute("placeholder", "Saved. Leave empty to keep it");

    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(update.mock.calls[1]?.[0]?.apiKey).toBeUndefined();
  });
});
