import { useI18n } from "@/shared/i18n";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { enumKey, RunStatus } from "@/shared/proto";

const RUN_TONES = {
  [RunStatus.RUNNING]: "info",
  [RunStatus.SUCCEEDED]: "positive",
  [RunStatus.FAILED]: "critical",
  [RunStatus.STOPPED]: "neutral",
  [RunStatus.UNSPECIFIED]: "neutral",
} as const satisfies Record<RunStatus, StatusTone>;

export function RunStatusBadge({ status }: { status: RunStatus }) {
  const { t } = useI18n();

  return (
    <span
      className={cn(
        "shrink-0 text-xs font-medium",
        statusTextTones[RUN_TONES[status] ?? "neutral"],
      )}
    >
      {t(`enums.runStatus.${enumKey(RunStatus, status)}`)}
    </span>
  );
}
