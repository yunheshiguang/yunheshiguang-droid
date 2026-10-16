import { Activity, ListChecks, Send } from "lucide-react";

import { resolveBatchAction } from "@/entities/task";
import { useWorkspaceHealth } from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import {
  BatchActionKind,
  GovernedAction,
  type OperationHealth,
  type ProbeHealth,
  type WorkspaceHealth,
} from "@/shared/proto";
import { ErrorState, SectionCard } from "@/shared/ui";

function Availability({ probes }: { probes: readonly ProbeHealth[] }) {
  const { t } = useI18n();
  const [latest] = probes;

  if (!latest) {
    return <p className="text-sm text-muted-foreground">{t("health.availability.noProbes")}</p>;
  }

  const trend = [...probes].reverse();

  return (
    <div className="space-y-3">
      <p className="text-2xl font-semibold">
        {t("health.availability.online", {
          online: String(latest.online),
          devices: String(latest.devices),
        })}
      </p>
      <p className="text-xs text-muted-foreground">
        {t("health.availability.unauthorized", { count: String(latest.unauthorized) })} ·{" "}
        {t("health.availability.offline", { count: String(latest.offline) })} ·{" "}
        {t("health.availability.probeTook", { ms: String(latest.durationMs) })}
      </p>
      <figure className="space-y-1">
        <div className="flex h-12 items-end gap-px" aria-hidden="true">
          {trend.map((probe) => (
            <div
              key={`${probe.probedAt?.seconds}.${probe.probedAt?.nanos}`}
              className="flex-1 rounded-sm bg-primary/70"
              style={{
                height: `${probe.devices === 0 ? 4 : Math.max(4, (probe.online / probe.devices) * 100)}%`,
              }}
            />
          ))}
        </div>
        <figcaption className="text-xs text-muted-foreground">
          {t("health.availability.trend", { count: String(probes.length) })}
        </figcaption>
      </figure>
    </div>
  );
}

function operationName(operation: OperationHealth, t: ReturnType<typeof useI18n>["t"]): string {
  const governed = t(runtimeKey(`enums.governedAction.${GovernedAction[operation.action]}`));

  if (
    operation.action !== GovernedAction.RUN_BATCH_ACTION ||
    operation.batchAction === BatchActionKind.UNSPECIFIED
  ) {
    return governed;
  }

  const action = resolveBatchAction(operation.batchAction);

  return action ? `${governed} · ${t(action.labelKey)}` : governed;
}

function Operations({ operations }: { operations: readonly OperationHealth[] }) {
  const { t } = useI18n();

  if (operations.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("health.operations.empty")}</p>;
  }

  const ms = (value: number) => t("health.operations.milliseconds", { ms: String(value) });

  return (
    <table className="w-full text-left text-sm">
      <thead className="text-xs text-muted-foreground">
        <tr>
          <th className="py-1 font-medium">{t("health.operations.operation")}</th>
          <th className="py-1 font-medium">{t("health.operations.runs")}</th>
          <th className="py-1 font-medium">{t("health.operations.failures")}</th>
          <th className="py-1 font-medium">{t("health.operations.average")}</th>
          <th className="py-1 font-medium">{t("health.operations.slowest")}</th>
        </tr>
      </thead>
      <tbody>
        {operations.map((operation) => (
          <tr
            key={`${operation.action}/${operation.batchAction}`}
            className="border-t border-border"
          >
            <td className="py-2">{operationName(operation, t)}</td>
            <td className="py-2">{operation.runs}</td>
            <td className={operation.failures > 0 ? "py-2 text-destructive" : "py-2"}>
              {operation.failures}
            </td>
            <td className="py-2">{ms(operation.averageMs)}</td>
            <td className="py-2">{ms(operation.slowestMs)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function Sinks({ sinks }: { sinks: WorkspaceHealth["sinks"] }) {
  const { t } = useI18n();

  return (
    <ul className="space-y-2">
      {sinks.map((sink) => (
        <li key={sink.id}>
          <p className="text-sm font-medium">{textValue(sink.label, true, t) || sink.id}</p>
          <p className="text-xs text-muted-foreground">{textValue(sink.description, true, t)}</p>
        </li>
      ))}
    </ul>
  );
}

export function HealthPage() {
  const { t } = useI18n();
  const { health, loading, error } = useWorkspaceHealth();

  if (error) {
    return (
      <div className="p-6">
        <ErrorState title={t("health.failed")} detail={error} />
      </div>
    );
  }

  if (loading || !health) {
    return <p className="p-6 text-sm text-muted-foreground">{t("health.loading")}</p>;
  }

  return (
    <div className="grid gap-6 p-6 xl:grid-cols-[1fr_1fr]">
      <SectionCard icon={Activity} title={t("health.availability.title")}>
        <Availability probes={health.probes} />
      </SectionCard>
      <SectionCard icon={Send} title={t("health.sinks.title")}>
        <Sinks sinks={health.sinks} />
      </SectionCard>
      <SectionCard
        icon={ListChecks}
        title={t("health.operations.title")}
        description={t("health.operations.description")}
        className="xl:col-span-2"
      >
        <Operations operations={health.operations} />
      </SectionCard>
    </div>
  );
}
