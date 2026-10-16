import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import {
  parseWorkspaceExtensions,
  renderExtensionTable,
  resolveWorkspaceExtensions,
  workspaceExtensions,
} from "./vite-plugin-workspace-extensions.ts";

const roots: string[] = [];

function checkout(packages: Record<string, unknown>): string {
  const root = mkdtempSync(path.join(tmpdir(), "workspace-extensions-"));

  roots.push(root);
  for (const [directory, manifest] of Object.entries(packages)) {
    mkdirSync(path.join(root, directory), { recursive: true });
    writeFileSync(path.join(root, directory, "package.json"), JSON.stringify(manifest));
  }

  return root;
}

afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});

describe("workspace extensions plugin", () => {
  it("reads a comma-separated list and treats a missing one as the open-source build", () => {
    expect(parseWorkspaceExtensions(" ./extensions/a/web, @acme/web ,, ")).toEqual([
      "./extensions/a/web",
      "@acme/web",
    ]);
    expect(parseWorkspaceExtensions(undefined)).toEqual([]);
  });

  it("finds a local package by its manifest and leaves package names to Vite", () => {
    const root = checkout({
      "extensions/a/web": {
        name: "@yunheshiguang/extension-a",
        exports: { ".": "./src/index.ts" },
      },
    });

    expect(resolveWorkspaceExtensions(root, ["./extensions/a/web", "@acme/web"])).toEqual([
      {
        name: "@yunheshiguang/extension-a",
        specifier: path.join(root, "extensions/a/web/src/index.ts").split(path.sep).join("/"),
      },
      { name: "@acme/web", specifier: "@acme/web" },
    ]);
  });

  it("refuses packages it cannot load and extensions listed twice", () => {
    const root = checkout({ "extensions/b/web": { name: "b" } });

    expect(() => resolveWorkspaceExtensions(root, ["./extensions/missing/web"])).toThrow(
      /has no package.json/,
    );
    expect(() => resolveWorkspaceExtensions(root, ["./extensions/b/web"])).toThrow(
      /must declare a name and a "." export/,
    );
    expect(() => resolveWorkspaceExtensions(root, ["@acme/web", "@acme/web"])).toThrow(
      /listed more than once/,
    );
  });

  it("serves the table as a virtual module, empty when nothing is installed", () => {
    const plugin = workspaceExtensions(checkout({}), []);
    const resolveId = plugin.resolveId as (id: string) => string | undefined;
    const load = plugin.load as (id: string) => string | undefined;
    const resolved = resolveId("virtual:workspace-extensions");

    expect(resolved).toBe("\0virtual:workspace-extensions");
    expect(load(resolved ?? "")).toBe("export default {\n};\n");
    expect(resolveId("react")).toBeUndefined();
    expect(renderExtensionTable([{ name: "@acme/web", specifier: "@acme/web" }])).toBe(
      'import * as extension0 from "@acme/web";\nexport default {\n  "@acme/web": extension0,\n};\n',
    );
  });
});
