import { describe, expect, it } from "vitest";

import { localeRegistry, messageKeys, messageTemplates } from "@/shared/i18n";
import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";

import { fullAssembly } from "./full-assembly";
import { placeholdersIn, suppliedParams } from "./message-params";

const { messages: appMessages } = fullAssembly();

type Message = {
  placeholders: Set<string>;
  plural: boolean;
};

function messagesFor(locale: string): Map<string, Message> {
  const pack = localeRegistry.resolve(locale);

  if (!pack) {
    throw new Error(`locale pack ${locale} is missing`);
  }

  const tree = deepMerge(pack.messages as PlainObject, (appMessages[locale] ?? {}) as PlainObject);

  return new Map(
    messageKeys(tree).map((key) => {
      const templates = messageTemplates(tree, key);

      return [
        key,
        {
          placeholders: new Set(templates.flatMap((template) => [...placeholdersIn(template)])),
          plural: templates.length > 1,
        },
      ];
    }),
  );
}

const sorted = (names: Set<string>) => Array.from(names).sort().join(",");

const locales = localeRegistry.packs.map((pack) => pack.locale);
const [referenceLocale, ...otherLocales] = locales;

describe("message placeholders", () => {
  it("finds copy that takes placeholders", () => {
    const withParams = Array.from(messagesFor(referenceLocale).values()).filter(
      (message) => message.placeholders.size > 0,
    );

    expect(withParams.length).toBeGreaterThan(0);
  });

  it.each(otherLocales)(
    "keeps the same placeholder set in %s as the reference locale",
    (locale) => {
      const reference = messagesFor(referenceLocale);
      const translated = messagesFor(locale);
      const drifted: string[] = [];

      for (const [key, message] of reference) {
        const other = translated.get(key);

        if (other === undefined) {
          continue;
        }

        if (sorted(message.placeholders) !== sorted(other.placeholders)) {
          drifted.push(
            `${key}: ${referenceLocale} [${sorted(message.placeholders)}] vs ${locale} [${sorted(other.placeholders)}]`,
          );
        }
      }

      expect(drifted).toEqual([]);
    },
  );

  it("has a supplier for every placeholder it declares", () => {
    const { byKey, viaComputedKey } = suppliedParams();
    const unsupplied: string[] = [];

    for (const [key, message] of messagesFor(referenceLocale)) {
      const provided = byKey.get(key) ?? new Set<string>();

      for (const name of message.placeholders) {
        if (!provided.has(name) && !viaComputedKey.has(name)) {
          unsupplied.push(`${key} -> {{${name}}}`);
        }
      }
    }

    expect(unsupplied).toEqual([]);
  });

  it("declares a placeholder for every param a caller supplies", () => {
    const messages = messagesFor(referenceLocale);
    const unused: string[] = [];

    for (const [key, provided] of suppliedParams().exactByKey) {
      const message = messages.get(key);

      if (message === undefined) {
        continue;
      }

      for (const name of provided) {
        if (!message.placeholders.has(name) && !(message.plural && name === "count")) {
          unused.push(`${key} <- ${name}`);
        }
      }
    }

    expect(unused).toEqual([]);
  });
});
