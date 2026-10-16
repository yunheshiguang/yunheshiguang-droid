import { describe, expect, it } from "vitest";

import { parseTagList } from "./tags";

describe("parseTagList", () => {
  it("splits on either comma and on line breaks, dropping blanks", () => {
    expect(parseTagList("客服， TikTok\n美区,,")).toEqual(["客服", "TikTok", "美区"]);
  });

  it("keeps the first spelling of a tag repeated in another case", () => {
    expect(parseTagList("TikTok, tiktok, TIKTOK")).toEqual(["TikTok"]);
  });
});
