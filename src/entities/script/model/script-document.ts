import { create, toJsonString } from "@bufbuild/protobuf";

import { type Script, ScriptLanguage, ScriptLanguageSchema, ScriptSchema } from "@/shared/proto";

export type ScriptBody = Pick<Script, "name" | "language" | "body">;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function languageNamed(name: unknown): ScriptLanguage {
  return (
    ScriptLanguageSchema.values.find((value) => value.name === name || value.number === name)
      ?.number ?? ScriptLanguage.UNSPECIFIED
  );
}

export function encodeScriptDocument(script: ScriptBody): string {
  return toJsonString(
    ScriptSchema,
    create(ScriptSchema, { name: script.name, language: script.language, body: script.body }),
    { alwaysEmitImplicit: true },
  );
}

export function decodeScriptDocument(body: string): ScriptBody | undefined {
  let value: unknown;

  try {
    value = JSON.parse(body);
  } catch {
    return undefined;
  }

  if (!isRecord(value) || typeof value.name !== "string") {
    return undefined;
  }

  return {
    name: value.name,
    language: languageNamed(value.language),
    body: typeof value.body === "string" ? value.body : "",
  };
}
