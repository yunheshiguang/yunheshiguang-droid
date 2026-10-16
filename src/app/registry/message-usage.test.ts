import { describe, expect, it } from "vitest";

import { hasMessage, localeRegistry, messageKeys } from "@/shared/i18n";
import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";

import { fullAssembly } from "./full-assembly";
import {
  backendMessageKeys,
  dynamicKeyPrefixes,
  literalMessageKeys,
  referencedKeyLiterals,
} from "./message-usage";

const { messages: appMessages } = fullAssembly();

function treeFor(locale: string): PlainObject {
  const pack = localeRegistry.resolve(locale);

  if (!pack) {
    throw new Error(`locale pack ${locale} is missing`);
  }

  return deepMerge(pack.messages as PlainObject, (appMessages[locale] ?? {}) as PlainObject);
}

const locales = localeRegistry.packs.map((pack) => pack.locale);

describe("literal message keys", () => {
  it("finds the keys that components hard-code", () => {
    expect(literalMessageKeys().length).toBeGreaterThan(0);
  });

  it.each(locales)("resolves every hard-coded key in %s", (locale) => {
    const tree = treeFor(locale);
    const missing = literalMessageKeys()
      .filter(({ key }) => !hasMessage(tree, key))
      .map(({ key, file }) => `${key} (${file})`);

    expect(missing).toEqual([]);
  });
});

describe("locale packs", () => {
  it("carry no copy that nothing references", () => {
    const referenced = referencedKeyLiterals();
    const backend = backendMessageKeys();
    const prefixes = dynamicKeyPrefixes();

    const orphaned = messageKeys(treeFor(locales[0])).filter((key) => {
      if (referenced.has(key) || backend.has(key)) {
        return false;
      }

      return !prefixes.some((prefix) => key.startsWith(prefix));
    });

    expect(orphaned).toEqual([]);
  });
});
