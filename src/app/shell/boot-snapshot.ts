import { useEffect } from "react";

import { useI18n } from "@/shared/i18n";
import { useTheme } from "@/shared/theme";

export const BOOT_SNAPSHOT_KEY = "yunheshiguang.boot";

export type BootSnapshot = {
  lang: string;
  loadingLabel: string;
  themeClass: string;
  colorScheme: "light" | "dark";
};

export function useBootSnapshot() {
  const { locale, t } = useI18n();
  const { theme, themePacks } = useTheme();
  const pack = themePacks.find((candidate) => candidate.id === theme);
  const loadingLabel = t("loading.boot");

  useEffect(() => {
    if (!pack) {
      return;
    }

    const snapshot: BootSnapshot = {
      lang: locale,
      loadingLabel,
      themeClass: pack.className,
      colorScheme: pack.colorScheme,
    };

    try {
      window.localStorage.setItem(BOOT_SNAPSHOT_KEY, JSON.stringify(snapshot));
    } catch {}
  }, [locale, loadingLabel, pack]);
}
