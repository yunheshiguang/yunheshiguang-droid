import { describe, expect, it } from "vitest";

import { ScriptLanguage } from "@/shared/proto";

import { createScript, readScript, sortScripts } from "./script";
import { encodeScriptDocument } from "./script-document";
import { readScriptDocuments } from "./script-documents";

describe("library scripts", () => {
  it("starts new scripts with a line that runs on any device", () => {
    const shell = createScript(ScriptLanguage.SHELL, "Model");

    expect(shell).toMatchObject({ name: "Model", language: ScriptLanguage.SHELL });
    expect(shell.body).toContain("getprop");
    expect(createScript(ScriptLanguage.JAVASCRIPT).body).toContain("toast");
    expect(createScript().id).not.toBe(shell.id);
  });

  it("stores scripts as contract JSON the backend can run", () => {
    const body = encodeScriptDocument({
      name: "Model",
      language: ScriptLanguage.SHELL,
      body: "getprop",
    });

    expect(JSON.parse(body)).toEqual({
      name: "Model",
      language: "SCRIPT_LANGUAGE_SHELL",
      body: "getprop",
    });
    expect(readScript("s-1", body)).toEqual({
      id: "s-1",
      name: "Model",
      language: ScriptLanguage.SHELL,
      body: "getprop",
    });
    expect(readScript("broken", "{")).toBeUndefined();
    expect(readScript("broken", JSON.stringify({ name: 1, body: "" }))).toBeUndefined();
    expect(readScript("empty", JSON.stringify({ name: "Empty", language: 1 }))).toEqual({
      id: "empty",
      name: "Empty",
      language: ScriptLanguage.SHELL,
      body: "",
    });
  });

  it("reads a collection in name order and skips documents it cannot read", () => {
    const scripts = readScriptDocuments({
      documents: [
        {
          id: "b",
          body: encodeScriptDocument({ name: "Zeta", language: ScriptLanguage.SHELL, body: "x" }),
        },
        {
          id: "a",
          body: encodeScriptDocument({
            name: "Alpha",
            language: ScriptLanguage.JAVASCRIPT,
            body: "y",
          }),
        },
        { id: "c", body: "not json" },
      ],
    });

    expect(scripts.map((script) => script.id)).toEqual(["a", "b"]);
    expect(sortScripts([...scripts].reverse()).map((script) => script.id)).toEqual(["a", "b"]);
  });
});
