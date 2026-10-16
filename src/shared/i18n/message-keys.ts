import { readMessage } from "./message-resolver";

const PLURAL_CATEGORIES = ["zero", "one", "two", "few", "many", "other"] as const;
const PLURAL_SUFFIX = new RegExp(`_(?:${PLURAL_CATEGORIES.join("|")})$`);

export function messageKeyOf(leafKey: string): string {
  return leafKey.replace(PLURAL_SUFFIX, "");
}

export function messageTemplates(tree: unknown, key: string): string[] {
  return [key, ...PLURAL_CATEGORIES.map((category) => `${key}_${category}`)]
    .map((candidate) => readMessage(tree, candidate))
    .filter((template): template is string => template !== undefined);
}

export function hasMessage(tree: unknown, key: string): boolean {
  return messageTemplates(tree, key).length > 0;
}

function leafKeys(tree: unknown, prefix: string, collected: string[]): string[] {
  if (tree === null || typeof tree !== "object") {
    return collected;
  }

  for (const [segment, value] of Object.entries(tree)) {
    const key = prefix ? `${prefix}.${segment}` : segment;

    if (typeof value === "string") {
      collected.push(key);
    } else {
      leafKeys(value, key, collected);
    }
  }

  return collected;
}

export function messageKeys(tree: unknown): string[] {
  return Array.from(new Set(leafKeys(tree, "", []).map(messageKeyOf)));
}
