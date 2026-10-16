import { describe, expect, it } from "vitest";

import { deepMerge } from "./deep-merge";

describe("deepMerge", () => {
  it("merges nested branches instead of replacing them", () => {
    const merged = deepMerge(
      { modules: { overview: { label: "总览" } } },
      { modules: { devices: { label: "设备墙" } } },
    );

    expect(merged).toEqual({
      modules: { overview: { label: "总览" }, devices: { label: "设备墙" } },
    });
  });

  it("lets the overlay win for scalars and arrays", () => {
    const merged = deepMerge({ label: "old", tags: ["a"] }, { label: "new", tags: ["b"] });

    expect(merged).toEqual({ label: "new", tags: ["b"] });
  });

  it("does not mutate its inputs", () => {
    const base = { nav: { label: "old" } };
    deepMerge(base, { nav: { label: "new" } });

    expect(base).toEqual({ nav: { label: "old" } });
  });
});
