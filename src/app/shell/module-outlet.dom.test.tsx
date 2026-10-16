import { render, screen } from "@testing-library/react";
import type { ComponentType } from "react";
import { describe, expect, it, vi } from "vitest";

import { createWorkspaceTestHarness } from "@/shared/testing";
import { lazyModulePage, type WorkspaceModuleDefinition } from "@/shared/workspace-module";

import { ModuleOutlet } from "./module-outlet";

type PageModule = { Page: ComponentType };

function moduleLoading(load: () => Promise<PageModule>, id = "probe"): WorkspaceModuleDefinition {
  return {
    id,
    component: lazyModulePage(load, (page) => page.Page),
    navigation: {
      groupId: "workspace",
      labelKey: "probe.label",
      descriptionKey: "probe.description",
      icon: () => null,
    },
  };
}

describe("ModuleOutlet", () => {
  it("loads a module's page only once the module is opened", async () => {
    const load = vi.fn(async (): Promise<PageModule> => ({ Page: () => <p>Module body</p> }));
    const probe = moduleLoading(load);
    const { wrapper } = createWorkspaceTestHarness();

    expect(load).not.toHaveBeenCalled();

    render(<ModuleOutlet module={probe} moduleId={probe.id} />, { wrapper });

    expect(await screen.findByText("Module body")).toBeInTheDocument();
    expect(load).toHaveBeenCalledTimes(1);
  });

  it("opens the next module at the top instead of where the last one was scrolled", async () => {
    const first = moduleLoading(async () => ({ Page: () => <p>First body</p> }), "first");
    const second = moduleLoading(async () => ({ Page: () => <p>Second body</p> }), "second");
    const { wrapper } = createWorkspaceTestHarness();
    const { rerender } = render(<ModuleOutlet module={first} moduleId={first.id} />, { wrapper });

    const scrolled = (await screen.findByText("First body")).parentElement;
    expect(scrolled).not.toBeNull();
    if (!scrolled) {
      return;
    }
    scrolled.scrollTop = 400;
    expect(scrolled.scrollTop).toBe(400);
    rerender(<ModuleOutlet module={second} moduleId={second.id} />);

    const opened = (await screen.findByText("Second body")).parentElement;
    expect(opened?.scrollTop).toBe(0);
  });

  it("keeps a page that fails to load inside the outlet", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const probe = moduleLoading(() => Promise.reject(new Error("chunk missing")));
    const { wrapper } = createWorkspaceTestHarness();

    render(
      <>
        <nav>Sidebar</nav>
        <ModuleOutlet module={probe} moduleId={probe.id} />
      </>,
      { wrapper },
    );

    expect(await screen.findByText("chunk missing")).toBeInTheDocument();
    expect(screen.getByText("This module encountered an error")).toBeInTheDocument();
    expect(screen.getByText("Sidebar")).toBeInTheDocument();
  });
});
