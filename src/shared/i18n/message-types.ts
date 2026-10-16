import type { MessageCatalog } from "./gen/message-catalog";

export type MessageKey = keyof MessageCatalog;

export type PlainMessageKey = {
  [K in MessageKey]: [MessageCatalog[K]] extends [never] ? K : never;
}[MessageKey];

declare const runtime: unique symbol;

export type RuntimeKey = string & { readonly [runtime]: true };

export function runtimeKey(key: string): RuntimeKey {
  return key as RuntimeKey;
}

export type MessageParams<K extends MessageKey> = [MessageCatalog[K]] extends [never]
  ? []
  : [params: Readonly<Record<MessageCatalog[K], string>>];

export interface Translate {
  <K extends MessageKey>(key: K, ...params: MessageParams<K>): string;
  (key: RuntimeKey, params?: Readonly<Record<string, string>>): string;
}
