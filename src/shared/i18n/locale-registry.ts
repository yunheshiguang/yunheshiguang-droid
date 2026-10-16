import { collectPacks, createPackRegistry, type PackRegistry } from "@/shared/lib/pack-registry";
import { compareIds } from "@/shared/lib/text";

import { isLocalePack, type LocalePack, type LocaleTag } from "./locale-pack";

const localeEntrypoints = import.meta.glob<Record<string, unknown>>("./locales/*/index.ts", {
  eager: true,
});

export interface LocaleRegistry extends PackRegistry<LocalePack, LocaleTag> {
  readonly defaultLocale: LocaleTag;
  match(browserLanguage: string): LocaleTag;
}

function discoverLocalePacks(): readonly LocalePack[] {
  return collectPacks(localeEntrypoints, isLocalePack, "i18n").sort((left, right) =>
    compareIds(left.locale, right.locale),
  );
}

export function createLocaleRegistry(
  packs: readonly LocalePack[],
  preferredDefault = "en-US",
): LocaleRegistry {
  const registry = createPackRegistry<LocalePack, LocaleTag>({
    packs,
    keyOf: (pack) => pack.locale,
    preferredDefault,
  });

  return {
    ...registry,
    defaultLocale: registry.defaultKey,
    match(browserLanguage) {
      const language = browserLanguage.toLowerCase();
      const exact = packs.find((pack) => pack.locale.toLowerCase() === language);

      if (exact) {
        return exact.locale;
      }

      const prefixed = packs.find((pack) =>
        pack.matches.some((prefix) => language.startsWith(prefix.toLowerCase())),
      );

      return prefixed?.locale ?? registry.defaultKey;
    },
  };
}

export const localeRegistry = createLocaleRegistry(discoverLocalePacks());
