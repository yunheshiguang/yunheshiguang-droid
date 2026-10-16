import { CheckCircle2, XCircle } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { healthClass } from "@/shared/lib/workspace-view";
import {
  type AndroidPlatformToolsBinary,
  AndroidPlatformToolsBinaryKind,
  ConnectionHealth,
  enumKey,
} from "@/shared/proto";
import { Badge } from "@/shared/ui";

export function BinaryStatus({ binary }: { binary: AndroidPlatformToolsBinary }) {
  const { t } = useI18n();
  const Icon = binary.available ? CheckCircle2 : XCircle;

  return (
    <div className="flex min-w-0 items-center justify-between gap-3 rounded-md border border-border bg-background p-3">
      <div className="flex min-w-0 items-center gap-3">
        <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0">
          <div className="text-sm font-medium">
            {t(
              runtimeKey(
                `enums.androidPlatformToolsBinaryKind.${enumKey(AndroidPlatformToolsBinaryKind, binary.kind)}`,
              ),
            )}
          </div>
          <div className="truncate text-xs text-muted-foreground">
            {binary.resolvedPath || t("androidPlatformTools.binaryMissing")}
          </div>
        </div>
      </div>
      <Badge
        variant="outline"
        className={healthClass(
          binary.available ? ConnectionHealth.READY : ConnectionHealth.BLOCKED,
        )}
      >
        {binary.available
          ? t("androidPlatformTools.available")
          : t("androidPlatformTools.unavailable")}
      </Badge>
    </div>
  );
}
