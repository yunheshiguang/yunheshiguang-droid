import {
  type DocumentActions,
  type StoredDocuments,
  useDocumentActions,
  useDocumentList,
  useDocumentListError,
} from "@/shared/api";

import { type LibraryScript, readScript, sortScripts } from "./script";
import { encodeScriptDocument } from "./script-document";

export const SCRIPTS_COLLECTION = "scripts";

const NO_SCRIPTS: readonly LibraryScript[] = [];

export function readScriptDocuments(list: StoredDocuments): LibraryScript[] {
  return sortScripts(
    list.documents.flatMap((document) => {
      const script = readScript(document.id, document.body);

      return script ? [script] : [];
    }),
  );
}

export function useScripts(): readonly LibraryScript[] {
  return useDocumentList(SCRIPTS_COLLECTION, readScriptDocuments) ?? NO_SCRIPTS;
}

export function useScriptsError(): Error | null {
  return useDocumentListError(SCRIPTS_COLLECTION);
}

export type ScriptActions = DocumentActions<LibraryScript>;

export function useScriptActions(): ScriptActions {
  return useDocumentActions<LibraryScript>(SCRIPTS_COLLECTION, encodeScriptDocument);
}
