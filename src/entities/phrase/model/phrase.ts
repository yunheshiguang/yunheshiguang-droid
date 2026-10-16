import { compareIds, compareNames } from "@/shared/lib/text";
export type Phrase = {
  id: string;
  title: string;
  body: string;
  tags: readonly string[];
};

export const MAX_PHRASE_TITLE_CHARS = 40;
export const MAX_PHRASE_BODY_CHARS = 1000;

export function createPhrase(): Phrase {
  return { id: crypto.randomUUID(), title: "", body: "", tags: [] };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function readPhrase(id: string, body: unknown): Phrase | undefined {
  if (!isRecord(body) || typeof body.title !== "string" || typeof body.body !== "string") {
    return undefined;
  }

  const tags = Array.isArray(body.tags)
    ? body.tags.filter((tag): tag is string => typeof tag === "string" && tag !== "")
    : [];

  return { id, title: body.title, body: body.body, tags };
}

export function sortPhrases(phrases: readonly Phrase[]): Phrase[] {
  return [...phrases].sort(
    (left, right) => compareNames(left.title, right.title) || compareIds(left.id, right.id),
  );
}

export function collectPhraseTags(phrases: readonly Phrase[]): string[] {
  return [...new Set(phrases.flatMap((phrase) => phrase.tags))].sort((left, right) =>
    compareNames(left, right),
  );
}

export function filterPhrases(
  phrases: readonly Phrase[],
  query: string,
  tag: string | "all",
): Phrase[] {
  const needle = query.trim().toLowerCase();

  return phrases.filter(
    (phrase) =>
      (tag === "all" || phrase.tags.includes(tag)) &&
      (!needle ||
        [phrase.title, phrase.body, ...phrase.tags].some((field) =>
          field.toLowerCase().includes(needle),
        )),
  );
}
