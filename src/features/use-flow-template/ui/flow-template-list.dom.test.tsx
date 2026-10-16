import { fromJson } from "@bufbuild/protobuf";
import { Code, ConnectError, type ServiceImpl } from "@connectrpc/connect";
import { createConnectQueryKey } from "@connectrpc/connect-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { taskMessages } from "@/entities/task";
import {
  type FlowTemplateCatalogJson,
  FlowTemplateCatalogSchema,
  TemplateService,
} from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";
import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

import { useFlowTemplateMessages } from "../messages";
import { FlowTemplateList } from "./flow-template-list";

const catalog: FlowTemplateCatalogJson = {
  sources: [
    { id: "core.starter", label: { key: "workspace.templates.starter.label" } },
    { id: "market", label: { key: "market.label", fallback: "Template market" } },
  ],
  templates: [
    {
      id: "recover-and-capture",
      sourceId: "core.starter",
      name: { key: "workspace.templates.recoverAndCapture.name" },
      description: { key: "workspace.templates.recoverAndCapture.description" },
      steps: [
        { kind: "BATCH_ACTION_KIND_RECOVER_CONNECTION" },
        { kind: "BATCH_ACTION_KIND_SCREENSHOT" },
      ],
    },
    {
      id: "nightly",
      sourceId: "market",
      name: { key: "market.nightly", fallback: "Nightly check" },
      steps: [{ kind: "BATCH_ACTION_KIND_LAUNCH_APP", value: "com.example" }],
    },
  ],
};

const marketMessages: WorkspaceModuleMessages = {
  "zh-CN": { market: { label: "模板市场", nightly: "夜间巡检" } },
  "en-US": { market: { label: "Template market", nightly: "Nightly check" } },
};

function renderList(
  props: Parameters<typeof FlowTemplateList>[0] = {},
  templates: Partial<ServiceImpl<typeof TemplateService>> = {},
) {
  const documents = createFakeDocuments();
  installFakeServices(documents.routes, (router) =>
    router.service(TemplateService, {
      listFlowTemplates: () => fromJson(FlowTemplateCatalogSchema, catalog),
      ...templates,
    }),
  );
  const { wrapper, queryClient } = createWorkspaceTestHarness(
    taskMessages,
    useFlowTemplateMessages,
    marketMessages,
  );
  queryClient.setQueryDefaults(
    createConnectQueryKey({
      schema: TemplateService.method.listFlowTemplates,
      cardinality: "finite",
    }),
    { retry: false },
  );

  render(<FlowTemplateList {...props} />, { wrapper });
  return documents;
}

describe("FlowTemplateList", () => {
  it("adds a template once however quickly Use is pressed again", async () => {
    const documents = renderList();

    await userEvent.dblClick(
      await screen.findByRole("button", {
        name: "Create a flow from the “Nightly check” template",
      }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent("Created flow “Nightly check”");
    expect(documents.bodies("automation-flows")).toHaveLength(1);
  });

  it("groups templates by source and turns one into a flow of its own", async () => {
    const documents = renderList();

    const starter = await screen.findByRole("region", { name: "Starter templates" });
    expect(within(starter).getByText("Reconnect, then screenshot")).toBeInTheDocument();
    expect(within(starter).getByText("Recover connection → Screenshot")).toBeInTheDocument();
    expect(
      within(screen.getByRole("region", { name: "Template market" })).getByText("Nightly check"),
    ).toBeInTheDocument();

    await userEvent.click(
      screen.getByRole("button", {
        name: "Create a flow from the “Reconnect, then screenshot” template",
      }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Created flow “Reconnect, then screenshot”",
    );
    expect(documents.bodies("automation-flows")).toEqual([
      {
        name: "Reconnect, then screenshot",
        steps: [
          expect.objectContaining({ kind: "BATCH_ACTION_KIND_RECOVER_CONNECTION", value: "" }),
          expect.objectContaining({ kind: "BATCH_ACTION_KIND_SCREENSHOT", value: "" }),
        ],
      },
    ]);
  });

  it("lists a single source when asked to", async () => {
    renderList({ sourceId: "market" });

    expect(await screen.findByText("Nightly check")).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Starter templates" })).not.toBeInTheDocument();
  });

  it("says so when the catalog cannot be read", async () => {
    renderList(
      {},
      { listFlowTemplates: () => Promise.reject(new ConnectError("offline", Code.Unavailable)) },
    );

    expect(await screen.findByText("Could not load templates")).toBeInTheDocument();
    expect(screen.getByText("offline")).toBeInTheDocument();
  });
});
