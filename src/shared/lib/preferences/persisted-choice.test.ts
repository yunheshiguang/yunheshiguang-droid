import { afterEach, describe, expect, it, vi } from "vitest";

import { createPersistedChoice } from "./persisted-choice";

type Theme = "light" | "dark";

const isTheme = (value: string): value is Theme => value === "light" || value === "dark";

function withStorage(store: Record<string, string>, behaviour?: { throws?: boolean }) {
  const storage = {
    getItem: (key: string) => {
      if (behaviour?.throws) {
        throw new Error("storage disabled");
      }

      return store[key] ?? null;
    },
    setItem: (key: string, value: string) => {
      if (behaviour?.throws) {
        throw new Error("storage disabled");
      }

      store[key] = value;
    },
  };

  vi.stubGlobal("window", { localStorage: storage });
}

function choice() {
  return createPersistedChoice<Theme>({
    storageKey: "app.theme",
    isValid: isTheme,
    fallback: () => "light",
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("createPersistedChoice", () => {
  it("reads a stored value back", () => {
    withStorage({ "app.theme": "dark" });

    expect(choice().read()).toBe("dark");
  });

  it("falls back when the stored value is no longer a valid option", () => {
    withStorage({ "app.theme": "solarized" });

    expect(choice().read()).toBe("light");
  });

  it("falls back instead of throwing when storage is unavailable", () => {
    withStorage({}, { throws: true });

    expect(choice().read()).toBe("light");
    expect(() => choice().write("dark")).not.toThrow();
  });

  it("ignores storage events for other keys", () => {
    withStorage({});
    const event = { key: "app.locale", newValue: "dark" } as StorageEvent;

    expect(choice().valueFromStorageEvent(event)).toBeUndefined();
  });

  it("adopts the value another window wrote", () => {
    withStorage({});
    const event = { key: "app.theme", newValue: "dark" } as StorageEvent;

    expect(choice().valueFromStorageEvent(event)).toBe("dark");
  });

  it("falls back when another window cleared or corrupted the value", () => {
    withStorage({});
    const cleared = { key: "app.theme", newValue: null } as StorageEvent;

    expect(choice().valueFromStorageEvent(cleared)).toBe("light");
  });

  it("keeps a valid selection and replaces one that is no longer offered", () => {
    expect(choice().coerce("dark")).toBe("dark");
    expect(choice().coerce("solarized")).toBe("light");
  });
});
