import { describe, expect, it } from "vitest";

import { ScriptLanguage } from "@/shared/proto";

import { editorLanguage, isUnsaved } from "./script-editing";

describe("script editing", () => {
  it("highlights each script language and falls back to plain text", () => {
    expect(editorLanguage(ScriptLanguage.SHELL)).toBe("shell");
    expect(editorLanguage(ScriptLanguage.JAVASCRIPT)).toBe("javascript");
    expect(editorLanguage(ScriptLanguage.UNSPECIFIED)).toBe("plain");
  });

  it("tells a draft that differs from the library copy", () => {
    const saved = { id: "s", name: "Model", language: ScriptLanguage.SHELL, body: "getprop" };

    expect(isUnsaved(saved, undefined)).toBe(true);
    expect(isUnsaved({ ...saved }, saved)).toBe(false);
    expect(isUnsaved({ ...saved, body: "id" }, saved)).toBe(true);
    expect(isUnsaved({ ...saved, language: ScriptLanguage.JAVASCRIPT }, saved)).toBe(true);
  });
});
