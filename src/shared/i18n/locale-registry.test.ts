import { describe, expect, it } from "vitest";

import type { LocalePack } from "./locale-pack";
import { createLocaleRegistry, localeRegistry } from "./locale-registry";

const en: LocalePack = {
  locale: "en-US",
  label: "English",
  matches: ["en"],
  messages: { app: { name: "Workspace" } },
};

const zh: LocalePack = {
  locale: "zh-CN",
  label: "中文",
  matches: ["zh"],
  messages: { app: { name: "工作台" } },
};

describe("createLocaleRegistry", () => {
  it("matches a browser language by its prefix", () => {
    const registry = createLocaleRegistry([en, zh]);

    expect(registry.match("zh-Hans-CN")).toBe("zh-CN");
    expect(registry.match("en-GB")).toBe("en-US");
  });

  it("falls back to the default for an unsupported language", () => {
    const registry = createLocaleRegistry([en, zh]);

    expect(registry.match("de-DE")).toBe("en-US");
    expect(registry.resolveOrDefault("de-DE")?.locale).toBe("en-US");
  });

  it("uses the first pack when the preferred default is missing", () => {
    expect(createLocaleRegistry([zh]).defaultLocale).toBe("zh-CN");
  });

  it("cycles to the next language and wraps around", () => {
    const registry = createLocaleRegistry([en, zh]);

    expect(registry.next("en-US")?.locale).toBe("zh-CN");
    expect(registry.next("zh-CN")?.locale).toBe("en-US");
  });

  it("cycles from an unknown tag to the first language", () => {
    const registry = createLocaleRegistry([en, zh]);

    expect(registry.next("de-DE")?.locale).toBe("en-US");
  });
});

describe("localeRegistry", () => {
  it("discovers every locale pack shipped in the repository", () => {
    expect(localeRegistry.packs.map((pack) => pack.locale)).toEqual(["en-US", "zh-CN"]);
  });

  it("gives each discovered pack a label written in its own language", () => {
    expect(localeRegistry.packs.every((pack) => pack.label.length > 0)).toBe(true);
  });
});
