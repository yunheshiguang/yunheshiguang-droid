import i18next from "i18next";

import type { PlainObject } from "@/shared/lib/deep-merge";

export type TranslationParams = Record<string, string>;

export type Translator = {
  hasLocale(locale: string): boolean;
  translate(locale: string, key: string, params?: TranslationParams): string;
};

export type MissingKeyHandler = (key: string, locale: string) => void;

function pluralCount(value: string | undefined): number | undefined {
  if (value === undefined || value.trim() === "") {
    return undefined;
  }

  const count = Number(value);

  return Number.isFinite(count) ? count : undefined;
}

export function createTranslator(
  trees: Readonly<Record<string, PlainObject>>,
  onMissingKey?: MissingKeyHandler,
): Translator {
  const instance = i18next.createInstance();

  void instance.init({
    resources: Object.fromEntries(
      Object.entries(trees).map(([locale, tree]) => [locale, { translation: tree }]),
    ),
    fallbackLng: false,
    initAsync: false,
    interpolation: { escapeValue: false },
    keySeparator: ".",
    nsSeparator: false,
    returnNull: false,
    saveMissing: onMissingKey !== undefined,
    missingKeyHandler: onMissingKey
      ? (locales, _namespace, key) => onMissingKey(key, locales[0] ?? "")
      : false,
  });

  return {
    hasLocale: (locale) => Object.hasOwn(trees, locale),
    translate(locale, key, params) {
      const count = pluralCount(params?.count);

      return instance.t(key, { ...params, ...(count === undefined ? {} : { count }), lng: locale });
    },
  };
}
