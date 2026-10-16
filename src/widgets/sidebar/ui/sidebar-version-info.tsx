import { Info } from "lucide-react";

import { appBuildInfo } from "@/shared/config";
import { useI18n } from "@/shared/i18n";

export function SidebarVersionInfo() {
  const { t } = useI18n();

  return (
    <div className="rounded-md border border-border bg-background p-3" title={t("app.buildInfo")}>
      <div className="flex items-start gap-3">
        <Info className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" aria-hidden="true" />
        <div className="min-w-0 space-y-1">
          <div className="flex min-w-0 items-center gap-2 text-xs leading-5">
            <span className="shrink-0 text-muted-foreground">{t("app.versionCode")}</span>
            <span className="truncate font-mono text-foreground">{appBuildInfo.versionCode}</span>
          </div>
          <div className="flex min-w-0 items-center gap-2 text-xs leading-5">
            <span className="shrink-0 text-muted-foreground">{t("app.versionName")}</span>
            <span className="truncate font-mono text-foreground">{appBuildInfo.versionName}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
