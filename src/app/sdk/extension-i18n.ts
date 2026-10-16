import { useMemo } from "react";

import { runtimeKey, useI18n as useWorkspaceI18n } from "@/shared/i18n";

export type ExtensionI18n = {
  locale: ReturnType<typeof useWorkspaceI18n>["locale"];
  t: (key: string, params?: Readonly<Record<string, string>>) => string;
};

export function useI18n(): ExtensionI18n {
  const { locale, t } = useWorkspaceI18n();

  return useMemo(
    () => ({
      locale,
      t: (key: string, params?: Readonly<Record<string, string>>) => t(runtimeKey(key), params),
    }),
    [locale, t],
  );
}
