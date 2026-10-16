import { isPlainObject, type PlainObject } from "@/shared/lib/deep-merge";

export type LocaleTag = string;

export type LocaleMessages = Readonly<PlainObject>;

export interface LocalePack {
  locale: LocaleTag;
  label: string;
  matches: readonly string[];
  messages: LocaleMessages;
}

export function isLocalePack(value: unknown): value is LocalePack {
  if (!isPlainObject(value)) {
    return false;
  }

  const candidate = value as Partial<LocalePack>;

  return (
    typeof candidate.locale === "string" &&
    candidate.locale.length > 0 &&
    typeof candidate.label === "string" &&
    Array.isArray(candidate.matches) &&
    isPlainObject(candidate.messages)
  );
}
