import { Download } from "lucide-react";
import { useId, useState } from "react";

import { AuditEntryList } from "@/entities/audit";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import { AuditExportFormat } from "@/shared/proto";
import { Button, ErrorState, Label, Select } from "@/shared/ui";

import {
  type AuditFilter,
  type AuditPeriod,
  auditOutcomes,
  auditPeriods,
  emptyAuditFilter,
  governedActions,
  withPeriod,
} from "../model/audit-filter";
import { useAuditHistory, useAuditOperators, useExportAudit } from "../model/use-audit-history";

type AuditBrowserProps = {
  groupName?: (id: string) => string;
};

const EXPORT_FORMATS = ["CSV", "JSONL"] as const;

export function AuditBrowser({ groupName }: AuditBrowserProps) {
  const { t } = useI18n();
  const id = useId();
  const [filter, setFilter] = useState(emptyAuditFilter);
  const history = useAuditHistory(filter);
  const { exportAs, exporting, result, error: exportError } = useExportAudit();
  const counted = useAuditOperators();
  const operators = new Map([
    ...counted.map((tally) => [tally.key, tally.label || tally.key] as const),
    ...history.entries.flatMap((entry) =>
      entry.operator
        ? [[entry.operator.id, entry.operator.name || entry.operator.id] as const]
        : [],
    ),
  ]);
  const exportDetail = result?.detail ? textValue(result.detail, true, t) : undefined;

  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div className="space-y-1">
          <Label htmlFor={`${id}-action`}>{t("governance.browse.action")}</Label>
          <Select
            id={`${id}-action`}
            value={filter.action}
            onChange={(event) =>
              setFilter({
                ...filter,
                action: event.currentTarget.value as AuditFilter["action"],
              })
            }
          >
            <option value="">{t("governance.browse.any")}</option>
            {governedActions.map((action) => (
              <option key={action} value={action}>
                {t(runtimeKey(`enums.governedAction.${action}`))}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-outcome`}>{t("governance.browse.outcome")}</Label>
          <Select
            id={`${id}-outcome`}
            value={filter.outcome}
            onChange={(event) =>
              setFilter({
                ...filter,
                outcome: event.currentTarget.value as AuditFilter["outcome"],
              })
            }
          >
            <option value="">{t("governance.browse.any")}</option>
            {auditOutcomes.map((outcome) => (
              <option key={outcome} value={outcome}>
                {t(runtimeKey(`enums.auditOutcome.${outcome}`))}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-operator`}>{t("governance.browse.operator")}</Label>
          <Select
            id={`${id}-operator`}
            value={filter.operatorId}
            onChange={(event) => setFilter({ ...filter, operatorId: event.currentTarget.value })}
          >
            <option value="">{t("governance.browse.any")}</option>
            {[...operators].map(([operatorId, name]) => (
              <option key={operatorId} value={operatorId}>
                {name}
              </option>
            ))}
            {filter.operatorId && !operators.has(filter.operatorId) ? (
              <option value={filter.operatorId}>{filter.operatorId}</option>
            ) : null}
          </Select>
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-period`}>{t("governance.browse.period")}</Label>
          <Select
            id={`${id}-period`}
            value={filter.period}
            onChange={(event) =>
              setFilter(withPeriod(filter, event.currentTarget.value as AuditPeriod, Date.now()))
            }
          >
            {auditPeriods.map((period) => (
              <option key={period} value={period}>
                {t(`governance.browse.periods.${period}`)}
              </option>
            ))}
          </Select>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {EXPORT_FORMATS.map((format) => (
          <Button
            key={format}
            type="button"
            variant="outline"
            size="sm"
            disabled={exporting}
            onClick={() =>
              void exportAs({ filter, format: AuditExportFormat[format] }).catch(() => undefined)
            }
          >
            <Download className="h-4 w-4" />
            {t(`governance.browse.export.${format}`)}
          </Button>
        ))}
        {exportDetail ? (
          <span
            role={result?.accepted ? "status" : "alert"}
            className="break-all text-xs text-muted-foreground"
          >
            {exportDetail}
          </span>
        ) : null}
        {exportError ? (
          <span role="alert" className="text-xs text-destructive">
            {errorMessage(exportError)}
          </span>
        ) : null}
      </div>

      {history.problem || history.error ? (
        <ErrorState
          title={t("governance.browse.unavailable")}
          detail={
            history.problem ? textValue(history.problem, true, t) : errorMessage(history.error)
          }
        />
      ) : history.loading ? (
        <p role="status" className="text-sm text-muted-foreground">
          {t("governance.browse.loading")}
        </p>
      ) : history.entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("governance.browse.empty")}</p>
      ) : (
        <AuditEntryList entries={history.entries} groupName={groupName} />
      )}

      {history.hasMore ? (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          disabled={history.loadingMore}
          onClick={history.loadMore}
        >
          {t("governance.browse.more")}
        </Button>
      ) : null}
    </div>
  );
}
