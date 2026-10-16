import { CheckCircle2, CircleAlert, Info, type LucideIcon, XCircle } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import { type ConnectionResult, enumKey, ConnectionOutcome as Outcome } from "@/shared/proto";
import { ErrorState } from "@/shared/ui";

import { connectionTone } from "../model/connection-tone";

const TONE_ICONS = {
  positive: CheckCircle2,
  info: Info,
  caution: CircleAlert,
  critical: XCircle,
  neutral: Info,
} satisfies Record<StatusTone, LucideIcon>;

type ConnectionOutcomeProps = {
  result?: ConnectionResult;
  error: unknown;
};

export function ConnectionOutcome({ result, error }: ConnectionOutcomeProps) {
  const { t } = useI18n();

  if (error != null) {
    return <ErrorState title={t("connectDevice.requestFailed")} detail={errorMessage(error)} />;
  }

  if (!result) {
    return null;
  }

  const tone = connectionTone(result.outcome);
  const Icon = TONE_ICONS[tone];
  const className = statusTextTones[tone];

  return (
    <p role="status" className="flex items-start gap-2 text-sm leading-6">
      <Icon className={cn("mt-1 h-4 w-4 shrink-0", className)} />
      <span className="min-w-0 break-words">
        <span className={cn("font-medium", className)}>
          {t(runtimeKey(`enums.connectionOutcome.${enumKey(Outcome, result.outcome)}`))}
        </span>{" "}
        {textValue(result.detail, true, t)}
      </span>
    </p>
  );
}
