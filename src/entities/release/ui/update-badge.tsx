import { CircleArrowUp } from "lucide-react";

import { useI18n } from "@/shared/i18n";
import { Button } from "@/shared/ui";

import { availableUpdate, openReleaseLink, shortCommit, useUpdateState } from "../model/release";

export function UpdateBadge() {
  const { t } = useI18n();
  const update = availableUpdate(useUpdateState());

  if (!update) {
    return null;
  }

  const version = update.channel === "nightly" ? shortCommit(update.commit) : update.version;

  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      className="h-8 gap-1.5 text-xs"
      aria-label={t("release.badge")}
      title={t("release.badgeTitle", { version })}
      onClick={() => openReleaseLink(update.pageUrl)}
    >
      <CircleArrowUp className="h-3.5 w-3.5" />
      <span className="hidden md:inline">{t("release.badge")}</span>
    </Button>
  );
}
