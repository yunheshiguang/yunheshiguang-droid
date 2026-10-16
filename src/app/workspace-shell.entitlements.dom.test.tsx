import { render, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { setCommandGateway } from "@/shared/api";
import { GovernanceService, WorkspaceService } from "@/shared/proto";
import { createFakeCommandGateway, installFakeServices } from "@/shared/testing";

import { WorkspaceShell } from "./workspace-shell";

vi.mock("./registry/discover-workspace-modules", async () => {
  const { createElement } = await import("react");
  const { FileText, House } = await import("lucide-react");
  const { workspaceNavigationGroupIds } = await import("@/shared/config");
  const copy = (id: string, label: string) => ({
    "en-US": { modules: { [id]: { label, description: `${label} module` } } },
    "zh-CN": { modules: { [id]: { label, description: `${label} 模块` } } },
  });

  return {
    discoverWorkspaceModules: () => [
      {
        id: "overview",
        component: () => createElement("p", null, "Base page"),
        navigation: {
          groupId: workspaceNavigationGroupIds.workspace,
          order: 10,
          labelKey: "modules.overview.label",
          descriptionKey: "modules.overview.description",
          icon: House,
        },
        messages: copy("overview", "Base"),
      },
      {
        id: "proReport",
        requires: ["pro.report"],
        component: () => createElement("p", null, "Pro page"),
        navigation: {
          groupId: workspaceNavigationGroupIds.workspace,
          order: 20,
          labelKey: "modules.proReport.label",
          descriptionKey: "modules.proReport.description",
          icon: FileText,
        },
        messages: copy("proReport", "Pro report"),
      },
    ],
  };
});

function renderShell(grantedCapabilities: string[]) {
  const governance = vi.fn(() => ({ grantedCapabilities }));
  setCommandGateway(createFakeCommandGateway({}));
  installFakeServices((router) => {
    router.service(WorkspaceService, {
      getWorkspaceSnapshot: () => ({ schemaVersion: "2026.06.p0" }),
    });
    router.service(GovernanceService, { getGovernanceState: governance });
  });
  render(<WorkspaceShell />);

  return governance;
}

describe("WorkspaceShell entitlements", () => {
  it("mounts a module once the backend grants the capability it requires", async () => {
    renderShell(["pro.report"]);

    const navigation = await screen.findByRole("navigation", { name: "Modules" });

    expect(await within(navigation).findByRole("button", { name: /Pro report/ })).toBeVisible();
    expect(within(navigation).getByRole("button", { name: /Base/ })).toBeVisible();
  });

  it("keeps the module out while the capability is not granted", async () => {
    const governance = renderShell([]);

    const navigation = await screen.findByRole("navigation", { name: "Modules" });
    await waitFor(() => expect(governance).toHaveBeenCalled());
    await screen.findByText("Base page");

    expect(within(navigation).queryByRole("button", { name: /Pro report/ })).toBeNull();
  });
});
