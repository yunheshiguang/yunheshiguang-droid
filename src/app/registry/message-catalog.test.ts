import { describe, expect, it } from "vitest";

import { localeRegistry, messageKeys, messageTemplates } from "@/shared/i18n";
import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";

import { collectAppMessages } from "./app-messages";
import { discoverWorkspaceModules } from "./discover-workspace-modules";
import { placeholdersIn } from "./message-params";
import { createAppWorkspaceRegistry } from "./workspace-registry";

const CATALOG = "../../shared/i18n/gen/message-catalog.ts";

function coreMessages() {
  const registry = createAppWorkspaceRegistry({
    capabilities: discoverWorkspaceModules().flatMap((module) => module.requires ?? []),
    extensions: [],
  });

  return collectAppMessages(registry, []);
}

function render(): string {
  const [reference] = localeRegistry.packs;
  const tree = deepMerge(
    reference.messages as PlainObject,
    (coreMessages()[reference.locale] ?? {}) as PlainObject,
  );
  const lines = messageKeys(tree)
    .sort()
    .map((key) => {
      const templates = messageTemplates(tree, key);
      const params = new Set(templates.flatMap((template) => [...placeholdersIn(template)]));
      if (templates.length > 1) {
        params.add("count");
      }
      const names = [...params].sort().map((name) => JSON.stringify(name));

      return `  ${JSON.stringify(key)}: ${names.length > 0 ? names.join(" | ") : "never"};`;
    });

  return ["export type MessageCatalog = {", ...lines, "};", ""].join("\n");
}

describe("message catalog", () => {
  it("lists every message key and its placeholders for the type checker", async () => {
    await expect(render()).toMatchFileSnapshot(CATALOG);
  });
});
