import { FolderOpen } from "lucide-react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { Button, ErrorState } from "@/shared/ui";

import { useRevealLogs } from "../model/use-reveal-logs";

export function RevealLogsButton() {
  const { t } = useI18n();
  const { reveal, pending, error } = useRevealLogs();

  return (
    <div className="flex flex-col items-end gap-2">
      <Button type="button" variant="outline" size="sm" disabled={pending} onClick={reveal}>
        <FolderOpen className="h-4 w-4" />
        {t("revealLogs.action")}
      </Button>
      {error ? <ErrorState title={t("revealLogs.failed")} detail={errorMessage(error)} /> : null}
    </div>
  );
}
