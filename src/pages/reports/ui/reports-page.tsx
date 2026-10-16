import { CalendarDays, Layers, ListChecks, Users } from "lucide-react";
import { useState } from "react";

import { useAuditReport, useWorkspaceGroups } from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { groupLabel, textValue } from "@/shared/lib/workspace-view";
import {
  type AuditTally,
  type DeviceGroup,
  enumKeyOfName,
  GovernedActionSchema,
} from "@/shared/proto";
import { ErrorState, Label, SectionCard, Select } from "@/shared/ui";

const PERIODS = { week: 7, month: 30, quarter: 90 } as const;
type Period = keyof typeof PERIODS;

type Translate = ReturnType<typeof useI18n>["t"];

function Tallies({
  rows,
  name,
}: {
  rows: readonly AuditTally[];
  name: (row: AuditTally) => string;
}) {
  const { t } = useI18n();

  if (rows.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("reports.empty")}</p>;
  }

  return (
    <table className="w-full text-left text-sm">
      <thead className="text-xs text-muted-foreground">
        <tr>
          <th className="py-1 font-medium">{t("reports.columns.name")}</th>
          <th className="py-1 font-medium">{t("reports.columns.total")}</th>
          <th className="py-1 font-medium">{t("reports.columns.failed")}</th>
          <th className="py-1 font-medium">{t("reports.columns.denied")}</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.key} className="border-t border-border">
            <td className="py-2">{name(row)}</td>
            <td className="py-2">{row.total}</td>
            <td className={row.failed > 0 ? "py-2 text-destructive" : "py-2"}>{row.failed}</td>
            <td className="py-2">{row.denied}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function Days({ days }: { days: readonly AuditTally[] }) {
  const { t } = useI18n();
  const busiest = Math.max(1, ...days.map((day) => day.total));

  if (days.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("reports.empty")}</p>;
  }

  return (
    <ol className="flex h-24 items-end gap-1">
      {days.map((day) => {
        const label = t("reports.byDay.bar", {
          day: day.key,
          count: String(day.total),
          failed: String(day.failed),
        });

        return (
          <li
            key={day.key}
            aria-label={label}
            title={label}
            className="flex-1 rounded-sm bg-primary/70"
            style={{ height: `${Math.max(4, (day.total / busiest) * 100)}%` }}
          />
        );
      })}
    </ol>
  );
}

function groupName(groups: readonly DeviceGroup[], t: Translate) {
  return (row: AuditTally) => groupLabel(groups, row.key, t) ?? row.key;
}

export function ReportsPage() {
  const { t } = useI18n();
  const [period, setPeriod] = useState<Period>("week");
  const { report, loading, error } = useAuditReport(PERIODS[period]);
  const groups = useWorkspaceGroups();
  const problem = report?.problem ? textValue(report.problem, true, t) : error;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1.5">
          <Label htmlFor="reports-period">{t("reports.period")}</Label>
          <Select
            id="reports-period"
            value={period}
            onChange={(event) => setPeriod(event.currentTarget.value as Period)}
          >
            {(Object.keys(PERIODS) as Period[]).map((option) => (
              <option key={option} value={option}>
                {t(`reports.periods.${option}`)}
              </option>
            ))}
          </Select>
        </div>
        {report && !report.problem ? (
          <p className="text-sm text-muted-foreground">
            {t("reports.counted", { count: String(report.entries) })}
          </p>
        ) : null}
      </div>

      {loading ? <p className="text-sm text-muted-foreground">{t("reports.loading")}</p> : null}
      {problem ? <ErrorState title={t("reports.failed")} detail={problem} /> : null}
      {report?.truncated ? (
        <p role="status" className="text-sm text-muted-foreground">
          {t("reports.truncated", { count: String(report.entries) })}
        </p>
      ) : null}

      {report && !report.problem ? (
        <div className="grid gap-6 xl:grid-cols-2">
          <SectionCard
            icon={CalendarDays}
            title={t("reports.byDay.title")}
            className="xl:col-span-2"
          >
            <Days days={report.byDay} />
          </SectionCard>
          <SectionCard icon={Users} title={t("reports.byOperator.title")}>
            <Tallies rows={report.byOperator} name={(row) => row.label || row.key} />
          </SectionCard>
          <SectionCard
            icon={Layers}
            title={t("reports.byGroup.title")}
            description={t("reports.byGroup.description")}
          >
            <Tallies rows={report.byGroup} name={groupName(groups, t)} />
          </SectionCard>
          <SectionCard
            icon={ListChecks}
            title={t("reports.byAction.title")}
            className="xl:col-span-2"
          >
            <Tallies
              rows={report.byAction}
              name={(row) =>
                t(
                  runtimeKey(
                    `enums.governedAction.${enumKeyOfName(GovernedActionSchema, row.key)}`,
                  ),
                )
              }
            />
          </SectionCard>
        </div>
      ) : null}
    </div>
  );
}
