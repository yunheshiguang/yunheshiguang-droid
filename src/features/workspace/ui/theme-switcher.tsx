import { MoonStar, SunMedium } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { useTheme } from "@/shared/theme";
import { Button } from "@/shared/ui";

export function ThemeSwitcher() {
  const { t } = useI18n();
  const { nextThemePack, setTheme } = useTheme();

  if (!nextThemePack) {
    return null;
  }

  const label = t("windowChrome.switchTheme", {
    theme: t(runtimeKey(nextThemePack.labelKey)),
  });
  const Icon = nextThemePack.colorScheme === "dark" ? MoonStar : SunMedium;

  return (
    <Button
      type="button"
      variant="outline"
      size="icon"
      title={label}
      aria-label={label}
      className="h-8 w-8 shrink-0 rounded-md"
      onClick={() => {
        setTheme(nextThemePack.id);
      }}
    >
      <Icon className="h-4 w-4" />
    </Button>
  );
}
