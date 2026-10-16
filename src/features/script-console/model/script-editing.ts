import type { LibraryScript } from "@/entities/script";
import { ScriptLanguage } from "@/shared/proto";
import type { CodeLanguage } from "@/shared/ui";

export function editorLanguage(language: ScriptLanguage): CodeLanguage {
  switch (language) {
    case ScriptLanguage.SHELL:
      return "shell";
    case ScriptLanguage.JAVASCRIPT:
      return "javascript";
    default:
      return "plain";
  }
}

export function isUnsaved(draft: LibraryScript, saved: LibraryScript | undefined): boolean {
  return (
    !saved ||
    saved.name !== draft.name ||
    saved.language !== draft.language ||
    saved.body !== draft.body
  );
}
