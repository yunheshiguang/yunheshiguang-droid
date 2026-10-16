import { describe, expect, it } from "vitest";

import { compareIds, compareNames } from "./collate";

describe("collation", () => {
  it("orders names the same way whatever the computer's language", () => {
    expect(["设备10", "美区", "TikTok", "设备2", "alpha", "Beta"].sort(compareNames)).toEqual([
      "alpha",
      "Beta",
      "TikTok",
      "美区",
      "设备2",
      "设备10",
    ]);
  });

  it("orders identifiers by code point", () => {
    expect(["b", "B", "a-2", "a-10"].sort(compareIds)).toEqual(["B", "a-10", "a-2", "b"]);
  });
});
