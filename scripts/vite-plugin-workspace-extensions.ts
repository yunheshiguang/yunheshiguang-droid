import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { normalizePath, type Plugin } from "vite";

const VIRTUAL_ID = "virtual:workspace-extensions";
const RESOLVED_ID = `\0${VIRTUAL_ID}`;

type ExtensionEntry = { name: string; specifier: string };

export function parseWorkspaceExtensions(value: string | undefined): string[] {
  return (value ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function packageEntry(directory: string): ExtensionEntry {
  const manifestPath = path.join(directory, "package.json");

  if (!existsSync(manifestPath)) {
    throw new Error(`workspace extension ${directory} has no package.json`);
  }

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as {
    name?: unknown;
    exports?: unknown;
  };
  const exported =
    typeof manifest.exports === "string"
      ? manifest.exports
      : (manifest.exports as Record<string, unknown> | undefined)?.["."];

  if (typeof manifest.name !== "string" || typeof exported !== "string") {
    throw new Error(`workspace extension ${directory} must declare a name and a "." export`);
  }

  return { name: manifest.name, specifier: normalizePath(path.resolve(directory, exported)) };
}

export function resolveWorkspaceExtensions(
  root: string,
  items: readonly string[],
): ExtensionEntry[] {
  const entries = items.map((item) =>
    item.startsWith(".") || path.isAbsolute(item)
      ? packageEntry(path.resolve(root, item))
      : { name: item, specifier: item },
  );
  const names = entries.map((entry) => entry.name);
  const repeated = names.find((name, index) => names.indexOf(name) !== index);

  if (repeated) {
    throw new Error(`workspace extension ${repeated} is listed more than once`);
  }

  return entries;
}

export function renderExtensionTable(entries: readonly ExtensionEntry[]): string {
  const imports = entries.map(
    ({ specifier }, index) => `import * as extension${index} from ${JSON.stringify(specifier)};`,
  );
  const rows = entries.map(({ name }, index) => `  ${JSON.stringify(name)}: extension${index},`);

  return [...imports, "export default {", ...rows, "};", ""].join("\n");
}

export function workspaceExtensions(root: string, items: readonly string[]): Plugin {
  const entries = resolveWorkspaceExtensions(root, items);

  return {
    name: "yunheshiguang:workspace-extensions",
    resolveId: (id) => (id === VIRTUAL_ID ? RESOLVED_ID : undefined),
    load: (id) => (id === RESOLVED_ID ? renderExtensionTable(entries) : undefined),
  };
}
