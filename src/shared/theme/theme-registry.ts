import { collectPacks, createPackRegistry, type PackRegistry } from "@/shared/lib/pack-registry";
import { compareIds } from "@/shared/lib/text";

import { isThemePack, type ThemeId, type ThemePack } from "./theme-pack";

const themeEntrypoints = import.meta.glob<Record<string, unknown>>("./themes/*.ts", {
  eager: true,
});

export interface ThemeRegistry extends PackRegistry<ThemePack, ThemeId> {
  readonly defaultThemeId: ThemeId;
  match(prefersDark: boolean): ThemeId;
}

function discoverThemePacks(): readonly ThemePack[] {
  return collectPacks(themeEntrypoints, isThemePack, "theme").sort(
    (left, right) => left.order - right.order || compareIds(left.id, right.id),
  );
}

export function createThemeRegistry(
  packs: readonly ThemePack[],
  preferredDefault = "light",
): ThemeRegistry {
  const registry = createPackRegistry<ThemePack, ThemeId>({
    packs,
    keyOf: (pack) => pack.id,
    preferredDefault,
  });

  return {
    ...registry,
    defaultThemeId: registry.defaultKey,
    match(prefersDark) {
      const matched = packs.find((pack) => pack.prefersDark === prefersDark);

      return matched?.id ?? registry.defaultKey;
    },
  };
}

export const themeRegistry = createThemeRegistry(discoverThemePacks());
