import { describe, expect, it } from "vitest";

import { listDeviceSorters } from "@/entities/device";
import { listBatchActions } from "@/entities/task";
import { workspaceNavigationGroups } from "@/shared/config";
import { hasMessage, localeRegistry, messageKeys } from "@/shared/i18n";
import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";
import { themeRegistry } from "@/shared/theme";

import { fullAssembly } from "./full-assembly";
import { extensionBackendKeys } from "./source-scan";

const { registry, messages: appMessages } = fullAssembly();

function treeFor(locale: string): PlainObject {
  const pack = localeRegistry.resolve(locale);

  if (!pack) {
    throw new Error(`locale pack ${locale} is missing`);
  }

  return deepMerge(pack.messages as PlainObject, (appMessages[locale] ?? {}) as PlainObject);
}

function declaredKeys(): string[] {
  return [
    ...registry.modules.flatMap((module) => [
      module.navigation.labelKey,
      module.navigation.descriptionKey,
    ]),
    ...workspaceNavigationGroups.map((group) => group.labelKey),
    ...themeRegistry.packs.map((pack) => pack.labelKey),
    ...listDeviceSorters().map((sorter) => sorter.labelKey),
    ...listBatchActions().map((action) => action.labelKey),
    ...extensionBackendKeys(),
  ];
}

describe("declared message keys", () => {
  it("discovers at least one locale and one module", () => {
    expect(localeRegistry.packs.length).toBeGreaterThan(0);
    expect(registry.modules.length).toBeGreaterThan(0);
  });

  it.each(localeRegistry.packs.map((pack) => pack.locale))("resolves every key in %s", (locale) => {
    const tree = treeFor(locale);
    const missing = declaredKeys().filter((key) => !hasMessage(tree, key));

    expect(missing).toEqual([]);
  });

  it("gives every locale the same set of declared keys", () => {
    const resolved = localeRegistry.packs.map((pack) => {
      const tree = treeFor(pack.locale);

      return declaredKeys().filter((key) => hasMessage(tree, key));
    });

    for (const keys of resolved) {
      expect(keys).toEqual(resolved[0]);
    }
  });
});

describe("merged message trees", () => {
  it("carry the same keys in every locale", () => {
    const byLocale = localeRegistry.packs.map((pack) => ({
      locale: pack.locale,
      keys: new Set(messageKeys(treeFor(pack.locale))),
    }));
    const reference = byLocale[0];
    const differences: string[] = [];

    for (const candidate of byLocale.slice(1)) {
      for (const key of reference.keys) {
        if (!candidate.keys.has(key)) {
          differences.push(`${key} is missing from ${candidate.locale}`);
        }
      }

      for (const key of candidate.keys) {
        if (!reference.keys.has(key)) {
          differences.push(`${key} is missing from ${reference.locale}`);
        }
      }
    }

    expect(differences).toEqual([]);
  });
});
