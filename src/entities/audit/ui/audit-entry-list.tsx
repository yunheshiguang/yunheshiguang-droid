import { timestampDate } from "@bufbuild/protobuf/wkt";
import { useMemo } from "react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import { type AuditEntry, AuditOutcome, enumKey, GovernedAction } from "@/shared/proto";

const OUTCOME_TONES = {
  [AuditOutcome.UNSPECIFIED]: "neutral",
  [AuditOutcome.SUCCEEDED]: "positive",
  [AuditOutcome.FAILED]: "critical",
  [AuditOutcome.DENIED]: "caution",
} as const satisfies Record<AuditOutcome, StatusTone>;

type AuditEntryListProps = {
  entries: readonly AuditEntry[];
  groupName?: (id: string) => string;
};

export function AuditEntryList({ entries, groupName = (id) => id }: AuditEntryListProps) {
  const { t, locale } = useI18n();
  const timeFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "medium" }),
    [locale],
  );

  return (
    <ol className="space-y-2">
      {entries.map((entry) => {
        const detail = textValue(entry.detail, true, t);

        return (
          <li key={entry.id} className="rounded-md border border-border bg-background p-3">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0 space-y-1">
                <div className="break-words text-sm font-medium">
                  {t(runtimeKey(`enums.governedAction.${enumKey(GovernedAction, entry.action)}`))}
                  {detail ? ` · ${detail}` : null}
                </div>
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                  {entry.occurredAt === undefined ? (
                    <span>{t("audit.unknownTime")}</span>
                  ) : (
                    <time dateTime={timestampDate(entry.occurredAt).toISOString()}>
                      {timeFormat.format(timestampDate(entry.occurredAt))}
                    </time>
                  )}
                  {entry.operator ? (
                    <span>
                      {t("audit.operator", {
                        operator: entry.operator.name || entry.operator.id,
                      })}
                    </span>
                  ) : null}
                  <span>{t("audit.actor", { actor: entry.actor })}</span>
                  {entry.targets.length > 0 ? (
                    <span>{t("audit.targets", { targets: entry.targets.join(", ") })}</span>
                  ) : null}
                  {entry.groups.length > 0 ? (
                    <span>
                      {t("audit.groups", { groups: entry.groups.map(groupName).join(", ") })}
                    </span>
                  ) : null}
                </div>
              </div>
              <span
                className={cn(
                  "shrink-0 text-xs font-medium",
                  statusTextTones[OUTCOME_TONES[entry.outcome] ?? "neutral"],
                )}
              >
                {t(runtimeKey(`enums.auditOutcome.${enumKey(AuditOutcome, entry.outcome)}`))}
              </span>
            </div>
          </li>
        );
      })}
    </ol>
  );
}
