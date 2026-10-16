import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { WorkspaceExtension, WorkspaceModuleDefinition } from "@/shared/workspace-module";

import { collectAppMessages } from "./app-messages";
import { checkoutExtensions } from "./full-assembly";
import { collectWorkspaceExtensions, installedWorkspaceExtensions } from "./workspace-extensions";
import { createAppWorkspaceRegistry } from "./workspace-registry";

const Page = () => null;

function extensionModule(
  id: string,
  extra: Partial<WorkspaceModuleDefinition> = {},
): WorkspaceModuleDefinition {
  return {
    id,
    component: Page,
    navigation: {
      groupId: "workspace",
      labelKey: `modules.${id}.label`,
      descriptionKey: `modules.${id}.description`,
      icon: Page,
    },
    ...extra,
  };
}

const enterprise: WorkspaceExtension = {
  id: "acme",
  modules: [
    extensionModule("acme-console", { requires: ["acme.console"] }),
    extensionModule("acme-overview", { replaces: ["overview"] }),
  ],
  messages: { "en-US": { acme: { source: "Acme templates" } } },
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("workspace extensions", () => {
  it("installs none in the open-source build", () => {
    expect(installedWorkspaceExtensions).toEqual([]);
    expect(createAppWorkspaceRegistry().has("acme-console")).toBe(false);
  });

  it("drops manifests that break the contract and keeps the first of two with one id", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const impostor: WorkspaceExtension = { id: "acme" };

    const collected = collectWorkspaceExtensions({
      "@acme/web": { default: enterprise, enterprise },
      "@acme/copy": { default: impostor },
      "@broken/web": { default: { id: "", modules: [] }, other: { id: "x", modules: {} } },
    });

    expect(collected).toEqual([enterprise]);
    expect(warn).toHaveBeenCalledWith("[extension] @broken/web exports no extension pack");
    expect(warn).toHaveBeenCalledWith("[extension] acme is installed twice; keeping the first");
  });

  it("gates extension modules by capability and lets them replace core modules", () => {
    const locked = createAppWorkspaceRegistry({ extensions: [enterprise] });
    const granted = createAppWorkspaceRegistry({
      extensions: [enterprise],
      capabilities: ["acme.console"],
    });

    expect(locked.has("acme-console")).toBe(false);
    expect(granted.has("acme-console")).toBe(true);
    expect(granted.has("acme-overview")).toBe(true);
    expect(granted.has("overview")).toBe(false);
  });

  it("holds every extension package in the checkout to the copy checks", () => {
    const onDisk = existsSync("extensions")
      ? readdirSync("extensions")
          .map((name) => `extensions/${name}/web`)
          .filter((directory) => existsSync(path.join(directory, "package.json")))
      : [];

    expect(
      checkoutExtensions()
        .map(({ directory }) => directory)
        .sort(),
    ).toEqual(onDisk.sort());
  });

  it("adds the extension's own copy to the overlay", () => {
    const registry = createAppWorkspaceRegistry({ extensions: [enterprise] });

    expect(collectAppMessages(registry, [enterprise])["en-US"]).toMatchObject({
      acme: { source: "Acme templates" },
    });
  });
});
