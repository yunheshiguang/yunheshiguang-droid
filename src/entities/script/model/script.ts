import { compareIds, compareNames } from "@/shared/lib/text";
import { ScriptLanguage } from "@/shared/proto";

import { decodeScriptDocument } from "./script-document";

export type LibraryScript = {
  id: string;
  name: string;
  language: ScriptLanguage;
  body: string;
};

export const MAX_SCRIPT_NAME_CHARS = 48;

export const scriptLanguages = [ScriptLanguage.SHELL, ScriptLanguage.JAVASCRIPT] as const;

const STARTERS: Partial<Record<ScriptLanguage, string>> = {
  [ScriptLanguage.SHELL]: "getprop ro.product.model\n",
  [ScriptLanguage.JAVASCRIPT]: 'toast("hello");\n',
};

export function createScript(
  language: ScriptLanguage = ScriptLanguage.SHELL,
  name = "",
): LibraryScript {
  return { id: crypto.randomUUID(), name, language, body: STARTERS[language] ?? "" };
}

export function readScript(id: string, body: string): LibraryScript | undefined {
  const script = decodeScriptDocument(body);

  return script ? { id, ...script } : undefined;
}

export function sortScripts(scripts: readonly LibraryScript[]): LibraryScript[] {
  return [...scripts].sort(
    (left, right) => compareNames(left.name, right.name) || compareIds(left.id, right.id),
  );
}
