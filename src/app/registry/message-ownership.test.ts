import path from "node:path";
import { describe, expect, it } from "vitest";

import { localeRegistry, messageKeys } from "@/shared/i18n";
import type { WorkspaceModuleDefinition } from "@/shared/workspace-module";

import { sliceMessageSources } from "./app-messages";
import { discoverWorkspaceModules } from "./discover-workspace-modules";
import { checkoutExtensions } from "./full-assembly";
import { frontendSources } from "./source-scan";

const layerRank: Record<string, number> = {
  shared: 0,
  entities: 1,
  features: 2,
  widgets: 3,
  pages: 4,
  app: 5,
};

type Owner =
  | { kind: "shared" }
  | { kind: "slice"; layer: string; slice: string }
  | { kind: "module"; id: string; extension?: string }
  | { kind: "extension"; id: string; directory: string };

type Location = {
  layer: string;
  slice: string | null;
  page: string | null;
  extension: string | null;
};

const referenceLocale = localeRegistry.packs[0]?.locale ?? "en-US";

function namespaceOf(name: string): string {
  return name.replace(/-(\w)/g, (_, letter: string) => letter.toUpperCase());
}

function describeOwner(owner: Owner): string {
  switch (owner.kind) {
    case "shared":
      return "the shared locale packs";
    case "slice":
      return owner.slice;
    case "module":
      return `module ${owner.id}`;
    case "extension":
      return `extension ${owner.id}`;
  }
}

function locate(file: string): Location | null {
  const [root, layer, slice] = file.split(path.sep);

  if (root === "extensions") {
    return {
      layer: "extension",
      slice: null,
      page: null,
      extension: [root, layer, slice].join("/"),
    };
  }

  if (root !== "src" || !(layer in layerRank)) {
    return null;
  }

  if (layer === "shared" || layer === "app") {
    return { layer, slice: null, page: null, extension: null };
  }

  return {
    layer,
    slice: `${layer}/${slice}`,
    page: layer === "pages" ? slice : null,
    extension: null,
  };
}

function mayUse(owner: Owner, location: Location): boolean {
  switch (owner.kind) {
    case "shared":
      return true;
    case "slice":
      return (
        location.extension === null &&
        (owner.slice === location.slice || layerRank[owner.layer] < layerRank[location.layer])
      );
    case "module":
      return owner.extension !== undefined
        ? owner.extension === location.extension
        : location.page !== null && namespaceOf(location.page) === namespaceOf(owner.id);
    case "extension":
      return owner.directory === location.extension;
  }
}

const owners = new Map<string, Owner>();
const duplicates: string[] = [];
const misplaced: string[] = [];

function claim(key: string, owner: Owner) {
  const existing = owners.get(key);

  if (existing) {
    duplicates.push(
      `${key} is defined by both ${describeOwner(existing)} and ${describeOwner(owner)}`,
    );
  } else {
    owners.set(key, owner);
  }
}

for (const key of messageKeys(localeRegistry.resolve(referenceLocale)?.messages)) {
  claim(key, { kind: "shared" });
}

for (const { slice, messages } of sliceMessageSources()) {
  const [layer, name] = slice.split("/");
  const namespace = `${namespaceOf(name)}.`;

  for (const key of messageKeys(messages[referenceLocale])) {
    if (!key.startsWith(namespace)) {
      misplaced.push(`${slice} defines ${key} outside its ${namespace} namespace`);
    }

    claim(key, { kind: "slice", layer, slice });
  }
}

function claimModule(module: WorkspaceModuleDefinition, extension?: string) {
  const namespace = namespaceOf(module.id);

  for (const key of messageKeys(module.messages?.[referenceLocale])) {
    if (!key.startsWith(`${namespace}.`) && !key.startsWith(`modules.${namespace}.`)) {
      misplaced.push(`module ${module.id} defines ${key} outside its ${namespace} namespace`);
    }

    claim(key, { kind: "module", id: module.id, extension });
  }
}

for (const module of discoverWorkspaceModules()) {
  claimModule(module);
}

for (const { directory, extension } of checkoutExtensions()) {
  const namespace = `${namespaceOf(extension.id)}.`;

  for (const key of messageKeys(extension.messages?.[referenceLocale])) {
    if (!key.startsWith(namespace)) {
      misplaced.push(`extension ${extension.id} defines ${key} outside its ${namespace} namespace`);
    }

    claim(key, { kind: "extension", id: extension.id, directory });
  }

  for (const module of extension.modules ?? []) {
    claimModule(module, directory);
  }
}

describe("message ownership", () => {
  it("finds copy owned by slices and by modules", () => {
    expect(sliceMessageSources().length).toBeGreaterThan(0);
    expect([...owners.values()].some((owner) => owner.kind === "module")).toBe(true);
  });

  it("gives every key exactly one owner", () => {
    expect(duplicates).toEqual([]);
  });

  it("keeps each owner's copy inside its own namespace", () => {
    expect(misplaced).toEqual([]);
  });

  it("renders only shared copy, the slice's own copy, or copy from a lower layer", () => {
    const borrowed: string[] = [];

    for (const { file, contents } of frontendSources()) {
      const location = locate(file);

      if (!location) {
        continue;
      }

      for (const match of contents.matchAll(/"([a-zA-Z][\w]*(?:\.[\w]+)+)"/g)) {
        const owner = owners.get(match[1]);

        if (owner && !mayUse(owner, location)) {
          borrowed.push(`${file} renders ${match[1]} owned by ${describeOwner(owner)}`);
        }
      }
    }

    expect(borrowed).toEqual([]);
  });
});
