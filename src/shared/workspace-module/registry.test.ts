import { describe, expect, it } from "vitest";

import { createWorkspaceRegistry } from "./registry";
import type { WorkspaceModuleDefinition } from "./types";

const icon = () => null;
const component = () => null;

function moduleFor(
  id: string,
  overrides: Partial<WorkspaceModuleDefinition> = {},
): WorkspaceModuleDefinition {
  return {
    id,
    component,
    navigation: {
      groupId: "workspace",
      labelKey: `modules.${id}.label`,
      descriptionKey: `modules.${id}.description`,
      icon,
    },
    ...overrides,
  };
}

const groups = [
  { id: "workspace", order: 10, labelKey: "nav.workspace" },
  { id: "risk-surface", order: 20, labelKey: "nav.riskSurface" },
];

describe("createWorkspaceRegistry", () => {
  it("drops candidates that do not satisfy the contract", () => {
    const registry = createWorkspaceRegistry({
      modules: [moduleFor("devices"), { id: "broken" }, null, "devices"],
      groups,
    });

    expect(registry.modules.map((module) => module.id)).toEqual(["devices"]);
  });

  it("keeps the last definition when an id is registered twice", () => {
    const first = moduleFor("devices");
    const second = moduleFor("devices", { stability: "preview" });
    const registry = createWorkspaceRegistry({ modules: [first, second], groups });

    expect(registry.modules).toEqual([second]);
  });

  it("gates modules behind capabilities and fails closed", () => {
    const modules = [moduleFor("devices"), moduleFor("audit", { requires: ["enterprise"] })];

    expect(createWorkspaceRegistry({ modules, groups }).modules.map((module) => module.id)).toEqual(
      ["devices"],
    );

    expect(
      createWorkspaceRegistry({
        modules,
        groups,
        capabilities: ["enterprise"],
      }).modules.map((module) => module.id),
    ).toEqual(["devices", "audit"]);
  });

  it("keeps the built-in module when its same-id override is switched off", () => {
    const builtIn = moduleFor("devices");
    const registry = createWorkspaceRegistry({
      modules: [builtIn, moduleFor("devices", { requires: ["enterprise"] })],
      groups,
    });

    expect(registry.modules).toEqual([builtIn]);
  });

  it("lets a module replace the default implementation it supersedes", () => {
    const registry = createWorkspaceRegistry({
      modules: [moduleFor("devices"), moduleFor("devices-pro", { replaces: ["devices"] })],
      groups,
    });

    expect(registry.modules.map((module) => module.id)).toEqual(["devices-pro"]);
  });

  it("orders groups and entries by their declared order", () => {
    const registry = createWorkspaceRegistry({
      modules: [
        moduleFor("security", {
          navigation: {
            groupId: "risk-surface",
            order: 10,
            labelKey: "modules.security.label",
            descriptionKey: "modules.security.description",
            icon,
          },
        }),
        moduleFor("devices", {
          navigation: {
            groupId: "workspace",
            order: 20,
            labelKey: "modules.devices.label",
            descriptionKey: "modules.devices.description",
            icon,
          },
        }),
        moduleFor("overview", {
          navigation: {
            groupId: "workspace",
            order: 10,
            labelKey: "modules.overview.label",
            descriptionKey: "modules.overview.description",
            icon,
          },
        }),
      ],
      groups,
    });

    expect(registry.navigation.map((group) => group.id)).toEqual(["workspace", "risk-surface"]);
    expect(registry.navigation[0].entries.map((entry) => entry.id)).toEqual([
      "overview",
      "devices",
    ]);
  });

  it("renders an undeclared group at the end instead of hiding its modules", () => {
    const registry = createWorkspaceRegistry({
      modules: [
        moduleFor("overview"),
        moduleFor("marketplace", {
          navigation: {
            groupId: "commercial",
            labelKey: "modules.marketplace.label",
            descriptionKey: "modules.marketplace.description",
            icon,
          },
        }),
      ],
      groups,
    });

    expect(registry.navigation.map((group) => group.id)).toEqual(["workspace", "commercial"]);
    expect(registry.navigation[1].labelKey).toBe("commercial");
  });

  it("keeps hidden modules resolvable but out of the navigation", () => {
    const registry = createWorkspaceRegistry({
      modules: [
        moduleFor("overview"),
        moduleFor("debug", {
          navigation: {
            groupId: "workspace",
            labelKey: "modules.debug.label",
            descriptionKey: "modules.debug.description",
            icon,
            hidden: true,
          },
        }),
      ],
      groups,
    });

    expect(registry.navigation[0].entries.map((entry) => entry.id)).toEqual(["overview"]);
    expect(registry.resolve("debug")?.id).toBe("debug");
  });

  it("falls back to the default module for unknown ids", () => {
    const registry = createWorkspaceRegistry({
      modules: [moduleFor("overview"), moduleFor("devices")],
      groups,
      preferredDefaultModuleId: "devices",
    });

    expect(registry.defaultModuleId).toBe("devices");
    expect(registry.resolveOrDefault("retired")?.id).toBe("devices");
    expect(registry.resolve("retired")).toBeUndefined();
  });

  it("ignores a preferred default that is not registered", () => {
    const registry = createWorkspaceRegistry({
      modules: [moduleFor("overview")],
      groups,
      preferredDefaultModuleId: "retired",
    });

    expect(registry.defaultModuleId).toBe("overview");
  });

  it("collects messages only from modules that were assembled", () => {
    const registry = createWorkspaceRegistry({
      modules: [
        moduleFor("overview", {
          messages: { "zh-CN": { modules: { overview: { label: "总览" } } } },
        }),
        moduleFor("audit", {
          requires: ["enterprise"],
          messages: { "zh-CN": { modules: { audit: { label: "审计" } } } },
        }),
      ],
      groups,
    });

    expect(registry.messages).toEqual({
      "zh-CN": { modules: { overview: { label: "总览" } } },
    });
  });
});
