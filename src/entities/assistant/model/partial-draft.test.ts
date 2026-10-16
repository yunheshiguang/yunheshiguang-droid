import { describe, expect, it } from "vitest";

import { parsePartialDraft } from "./partial-draft";

describe("parsePartialDraft", () => {
  it("reads what a draft has written so far", () => {
    expect(parsePartialDraft('{"name":"Battery","body":"dumpsys bat')).toEqual({
      name: "Battery",
      body: "dumpsys bat",
    });
    expect(parsePartialDraft('{"steps":[{"kind":"BATCH_ACTION_KIND_LAUNCH_APP"},{"ki')).toEqual({
      steps: [{ kind: "BATCH_ACTION_KIND_LAUNCH_APP" }, {}],
    });
  });

  it("reads a draft wrapped in a Markdown code block", () => {
    expect(parsePartialDraft('```json\n{"name":"Battery","body":"dumpsys')).toEqual({
      name: "Battery",
      body: "dumpsys",
    });
    expect(parsePartialDraft('```json\n{"name":"Battery","body":"dumpsys battery"}\n```')).toEqual({
      name: "Battery",
      body: "dumpsys battery",
    });
  });

  it("gives up on text that is not JSON at all", () => {
    expect(parsePartialDraft("")).toBeUndefined();
    expect(parsePartialDraft("sorry, I cannot")).toBeUndefined();
  });
});
