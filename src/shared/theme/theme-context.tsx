import { createContext, type ReactNode, useContext, useEffect, useMemo } from "react";

import { usePersistedChoice } from "@/shared/lib/preferences";

import type { ThemeId, ThemePack } from "./theme-pack";
import { themeRegistry } from "./theme-registry";

type ThemeContextValue = {
  theme: ThemeId;
  isDark: boolean;
  themePacks: readonly ThemePack[];
  nextThemePack: ThemePack | undefined;
  setTheme: (theme: ThemeId) => void;
  toggleTheme: () => void;
};

const STORAGE_KEY = "yunheshiguang.theme";

const ThemeContext = createContext<ThemeContextValue | null>(null);

function detectSystemTheme(): ThemeId {
  if (typeof window === "undefined") {
    return themeRegistry.defaultThemeId;
  }

  return themeRegistry.match(window.matchMedia("(prefers-color-scheme: dark)").matches);
}

function watchSystemTheme(onChange: () => void): () => void {
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  media.addEventListener("change", onChange);

  return () => media.removeEventListener("change", onChange);
}

function applyTheme(theme: ThemeId) {
  const pack = themeRegistry.resolveOrDefault(theme);

  if (!pack) {
    return;
  }

  const root = document.documentElement;

  for (const candidate of themeRegistry.packs) {
    if (candidate.className) {
      root.classList.toggle(candidate.className, candidate.id === pack.id);
    }
  }

  root.style.colorScheme = pack.colorScheme;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = usePersistedChoice<ThemeId>({
    storageKey: STORAGE_KEY,
    isValid: (value): value is ThemeId => themeRegistry.has(value),
    fallback: detectSystemTheme,
    watchFallback: watchSystemTheme,
  });

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  const value = useMemo<ThemeContextValue>(() => {
    const pack = themeRegistry.resolveOrDefault(theme);
    const nextThemePack = themeRegistry.next(theme);

    return {
      theme,
      isDark: pack?.colorScheme === "dark",
      themePacks: themeRegistry.packs,
      nextThemePack,
      setTheme,
      toggleTheme: () => {
        if (nextThemePack) {
          setTheme(nextThemePack.id);
        }
      },
    };
  }, [setTheme, theme]);

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const context = useContext(ThemeContext);

  if (!context) {
    throw new Error("useTheme must be used within ThemeProvider");
  }

  return context;
}
