import { createContext, type ReactNode, useContext, useMemo } from "react";

import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";
import { usePersistedChoice } from "@/shared/lib/preferences";

import type { LocaleMessages, LocalePack, LocaleTag } from "./locale-pack";
import { localeRegistry } from "./locale-registry";
import type { Translate } from "./message-types";
import { createTranslator, type MissingKeyHandler } from "./translator";

export type MessageOverlays = Readonly<Record<LocaleTag, LocaleMessages>>;

type I18nContextValue = {
  locale: LocaleTag;
  localePacks: readonly LocalePack[];
  nextLocalePack: LocalePack | undefined;
  setLocale: (locale: LocaleTag) => void;
  t: Translate;
};

const I18nContext = createContext<I18nContextValue | null>(null);
const STORAGE_KEY = "yunheshiguang.locale";

function detectBrowserLocale(): LocaleTag {
  if (typeof window === "undefined") {
    return localeRegistry.defaultLocale;
  }

  return localeRegistry.match(window.navigator.language);
}

function watchBrowserLocale(onChange: () => void): () => void {
  window.addEventListener("languagechange", onChange);

  return () => window.removeEventListener("languagechange", onChange);
}

function pickFallbackLocale(defaultLocale: LocaleTag | undefined): LocaleTag {
  return defaultLocale && localeRegistry.has(defaultLocale) ? defaultLocale : detectBrowserLocale();
}

function buildTrees(overlays?: MessageOverlays): Record<LocaleTag, PlainObject> {
  const trees: Record<LocaleTag, PlainObject> = {};

  for (const pack of localeRegistry.packs) {
    const overlay = overlays?.[pack.locale];

    trees[pack.locale] = overlay
      ? deepMerge(pack.messages as PlainObject, overlay as PlainObject)
      : (pack.messages as PlainObject);
  }

  return trees;
}

export function I18nProvider({
  children,
  defaultLocale,
  messageOverlays,
  onMissingKey,
}: {
  children: ReactNode;
  defaultLocale?: LocaleTag;
  messageOverlays?: MessageOverlays;
  onMissingKey?: MissingKeyHandler;
}) {
  const fallbackLocale = pickFallbackLocale(defaultLocale);
  const [locale, setLocale] = usePersistedChoice<LocaleTag>({
    storageKey: STORAGE_KEY,
    isValid: (value): value is LocaleTag => localeRegistry.has(value),
    fallback: () => pickFallbackLocale(defaultLocale),
    watchFallback: watchBrowserLocale,
  });
  const translator = useMemo(
    () => createTranslator(buildTrees(messageOverlays), onMissingKey),
    [messageOverlays, onMissingKey],
  );

  const value = useMemo<I18nContextValue>(() => {
    const language = translator.hasLocale(locale) ? locale : fallbackLocale;

    return {
      locale,
      localePacks: localeRegistry.packs,
      nextLocalePack: localeRegistry.next(locale),
      setLocale,
      t: ((key: string, params?: Readonly<Record<string, string>>) =>
        translator.translate(language, key, params)) as Translate,
    };
  }, [fallbackLocale, locale, setLocale, translator]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = useContext(I18nContext);

  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }

  return context;
}
