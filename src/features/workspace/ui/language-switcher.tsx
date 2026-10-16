import { Languages } from "lucide-react";

import { useI18n } from "@/shared/i18n";
import { Button } from "@/shared/ui";

export function LanguageSwitcher() {
  const { nextLocalePack, setLocale, t } = useI18n();

  if (!nextLocalePack) {
    return null;
  }

  const label = t("windowChrome.switchLocale", {
    language: nextLocalePack.label,
  });

  return (
    <Button
      type="button"
      variant="outline"
      size="icon"
      title={label}
      aria-label={label}
      className="h-8 w-8 shrink-0 rounded-md"
      onClick={() => {
        setLocale(nextLocalePack.locale);
      }}
    >
      <Languages className="h-4 w-4" />
    </Button>
  );
}
