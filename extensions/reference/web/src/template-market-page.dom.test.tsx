import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  BatchActionKind,
  createExtensionTestHarness,
  installFakeServices,
  TemplateService,
} from "yunheshiguang/sdk/testing";

import extension from "./index";
import { TemplateMarketPage } from "./template-market-page";

const catalog = {
  sources: [
    { id: "core.starter", label: { key: "workspace.templates.starter.label" } },
    { id: "reference.market", label: { key: "reference.market.label" } },
  ],
  templates: [
    {
      id: "capture-screens",
      sourceId: "core.starter",
      name: { key: "workspace.templates.captureScreens.name" },
      steps: [{ kind: BatchActionKind.SCREENSHOT }],
    },
    {
      id: "nightly-health-check",
      sourceId: "reference.market",
      name: { key: "reference.market.nightlyHealthCheck.name" },
      description: { key: "reference.market.nightlyHealthCheck.description" },
      steps: [{ kind: BatchActionKind.RECOVER_CONNECTION }, { kind: BatchActionKind.SCREENSHOT }],
    },
  ],
};

describe("reference extension", () => {
  it("puts each page behind its own capability", () => {
    expect(extension.id).toBe("reference");
    expect(extension.modules?.map((module) => [module.id, module.requires])).toEqual([
      ["template-market", ["reference.market"]],
      ["enterprise", ["reference.enterprise"]],
    ]);
  });

  it("lists only the market's templates on the market page", async () => {
    installFakeServices((router) =>
      router.service(TemplateService, { listFlowTemplates: () => catalog }),
    );
    const { wrapper } = createExtensionTestHarness(extension);

    render(<TemplateMarketPage />, { wrapper });

    expect(await screen.findByText("Nightly health check")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Template market" })).toBeInTheDocument();
    expect(screen.queryByText("Screenshot each device")).not.toBeInTheDocument();
  });
});
