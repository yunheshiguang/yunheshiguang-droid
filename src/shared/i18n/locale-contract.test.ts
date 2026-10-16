import { globSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { localeRegistry } from "./locale-registry";
import { hasMessage } from "./message-keys";

const backendMessagesFile = path.join(process.cwd(), "src-tauri/src/workspace/messages.rs");
const protoRoot = path.join(process.cwd(), "proto");

function readBackendMessageKeys(): string[] {
  const keys = Array.from(
    readFileSync(backendMessagesFile, "utf8").matchAll(/const\s+\w+_KEY:\s*&str\s*=\s*"([^"]+)"/g),
    (match) => match[1],
  );

  return Array.from(new Set(keys)).sort();
}

function readProtoEnumValues(): Map<string, Set<string>> {
  const enums = new Map<string, Set<string>>();

  for (const file of globSync("**/*.proto", { cwd: protoRoot })) {
    const contents = readFileSync(path.join(protoRoot, file), "utf8");

    for (const block of contents.matchAll(/enum\s+(\w+)\s*\{([^}]*)\}/g)) {
      const enumName = block[1];
      const prefix = `${enumName.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toUpperCase()}_`;
      const values = Array.from(block[2].matchAll(/^\s*([A-Z][A-Z0-9_]*)\s*=/gm), (match) =>
        match[1].slice(prefix.length),
      );

      enums.set(enumName, new Set(values));
    }
  }

  return enums;
}

const backendKeys = readBackendMessageKeys();
const protoEnums = readProtoEnumValues();
const locales = localeRegistry.packs.map((pack) => pack.locale);

describe("backend message keys", () => {
  it("finds the keys the backend declares", () => {
    expect(backendKeys.length).toBeGreaterThan(0);
    expect(locales.length).toBeGreaterThan(0);
  });

  it.each(locales)("resolves every backend key in %s", (locale) => {
    const pack = localeRegistry.resolve(locale);
    const missing = backendKeys.filter((key) => !hasMessage(pack?.messages, key));

    expect(missing).toEqual([]);
  });
});

describe("contract enum labels", () => {
  const enumLabelGroups = () => {
    const pack = localeRegistry.resolve(locales[0]);
    const labels = (pack?.messages as { enums?: Record<string, unknown> })?.enums ?? {};

    return Object.entries(labels);
  };

  it("names a real contract enum for every label group", () => {
    const unknown = enumLabelGroups()
      .map(([name]) => name.charAt(0).toUpperCase() + name.slice(1))
      .filter((name) => !protoEnums.has(name));

    expect(unknown).toEqual([]);
  });

  it("labels only values the contract still declares", () => {
    const stale: string[] = [];

    for (const [name, values] of enumLabelGroups()) {
      const enumName = name.charAt(0).toUpperCase() + name.slice(1);
      const declared = protoEnums.get(enumName);

      for (const value of Object.keys(values ?? {})) {
        if (declared && !declared.has(value)) {
          stale.push(`enums.${name}.${value}`);
        }
      }
    }

    expect(stale).toEqual([]);
  });

  it("labels every value of an enum the interface renders", () => {
    const missing: string[] = [];

    for (const [name, values] of enumLabelGroups()) {
      const enumName = name.charAt(0).toUpperCase() + name.slice(1);
      const labeled = new Set(Object.keys(values ?? {}));

      for (const value of protoEnums.get(enumName) ?? []) {
        if (!labeled.has(value)) {
          missing.push(`enums.${name}.${value}`);
        }
      }
    }

    expect(missing).toEqual([]);
  });
});
