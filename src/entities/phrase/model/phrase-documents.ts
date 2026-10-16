import {
  type DocumentActions,
  type StoredDocuments,
  useDocumentActions,
  useDocumentList,
  useDocumentListError,
} from "@/shared/api";

import { type Phrase, readPhrase, sortPhrases } from "./phrase";

export const PHRASES_COLLECTION = "phrases";

const PHRASE_DOCUMENT_VERSION = 1;

const NO_PHRASES: readonly Phrase[] = [];

export function toPhraseDocumentBody(phrase: Phrase): string {
  return JSON.stringify({
    version: PHRASE_DOCUMENT_VERSION,
    title: phrase.title,
    body: phrase.body,
    tags: phrase.tags,
  });
}

function parse(body: string): unknown {
  try {
    return JSON.parse(body);
  } catch {
    return undefined;
  }
}

export function readPhraseDocuments(list: StoredDocuments): Phrase[] {
  return sortPhrases(
    list.documents.flatMap((document) => {
      const phrase = readPhrase(document.id, parse(document.body));

      return phrase ? [phrase] : [];
    }),
  );
}

export function usePhrases(): readonly Phrase[] {
  return useDocumentList(PHRASES_COLLECTION, readPhraseDocuments) ?? NO_PHRASES;
}

export function usePhrasesError(): Error | null {
  return useDocumentListError(PHRASES_COLLECTION);
}

export type PhraseActions = DocumentActions<Phrase>;

export function usePhraseActions(): PhraseActions {
  return useDocumentActions(PHRASES_COLLECTION, toPhraseDocumentBody);
}
