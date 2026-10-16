import { ExternalLink, HardDriveDownload } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { installStatusClass } from "@/shared/lib/workspace-view";
import {
  type AndroidPlatformToolsInstall,
  AndroidPlatformToolsPackageKind,
  enumKey,
} from "@/shared/proto";
import { Badge, Button } from "@/shared/ui";

import { androidSdkLicenseUrl } from "../model/downloads";
import { DetailRow } from "./detail-row";

export type InstallPanelProps = {
  install?: AndroidPlatformToolsInstall;
  status: string;
  licenseAccepted: boolean;
  onLicenseAcceptedChange: (accepted: boolean) => void;
  busy: boolean;
  installing: boolean;
  onInstall: () => void;
  onOpenExternal: (url: string) => void;
};

export function InstallPanel({
  install,
  status,
  licenseAccepted,
  onLicenseAcceptedChange,
  busy,
  installing,
  onInstall,
  onOpenExternal,
}: InstallPanelProps) {
  const { t } = useI18n();

  return (
    <div className="rounded-md border border-border bg-background p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" className={installStatusClass(status)}>
              {t(runtimeKey(`enums.androidPlatformToolsInstallStatus.${status}`))}
            </Badge>
            <span className="text-sm font-medium">{t("androidPlatformTools.privateInstall")}</span>
          </div>
          <div className="space-y-2">
            <DetailRow
              label={t("androidPlatformTools.package")}
              value={t(
                `enums.androidPlatformToolsPackageKind.${enumKey(AndroidPlatformToolsPackageKind, install?.packageKind ?? AndroidPlatformToolsPackageKind.UNSPECIFIED)}`,
              )}
            />
            <DetailRow
              label={t("androidPlatformTools.installRoot")}
              value={install?.installRoot ?? ""}
            />
            <DetailRow
              label={t("androidPlatformTools.platformToolsRoot")}
              value={install?.platformToolsRoot ?? ""}
            />
            <DetailRow label={t("androidPlatformTools.version")} value={install?.version ?? ""} />
            <DetailRow label="SHA-256" value={install?.sha256 ?? ""} />
          </div>
        </div>

        <div className="grid gap-3 sm:max-w-md">
          <label className="flex items-start gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              className="mt-1"
              checked={licenseAccepted}
              onChange={(event) => onLicenseAcceptedChange(event.currentTarget.checked)}
              disabled={busy}
            />
            <span>{t("androidPlatformTools.licenseAcceptance")}</span>
          </label>
          <Button
            type="button"
            variant="ghost"
            className="justify-start"
            onClick={() => onOpenExternal(androidSdkLicenseUrl)}
          >
            <ExternalLink className="h-4 w-4" />
            {t("androidPlatformTools.licenseAgreement")}
          </Button>
          <Button
            type="button"
            className="min-w-36"
            disabled={busy || !licenseAccepted}
            onClick={onInstall}
          >
            <HardDriveDownload className="h-4 w-4" />
            {installing ? t("androidPlatformTools.installing") : t("androidPlatformTools.install")}
          </Button>
        </div>
      </div>
    </div>
  );
}
