import { Activity, RefreshCcw, Smartphone } from "lucide-react";
import type { ReactNode } from "react";

import {
  useWorkspaceDevices,
  useWorkspaceDiagnostics,
  useWorkspaceGroups,
  useWorkspaceSnapshotStatus,
  useWorkspaceTasks,
} from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { deviceTitle, healthClass, textValue } from "@/shared/lib/workspace-view";
import { ConnectionHealth, enumKey } from "@/shared/proto";
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  EmptyState,
  ErrorState,
  ScrollArea,
} from "@/shared/ui";

import { buildOverviewMetrics } from "../model/metrics";
import { DiagnosticsCard } from "./diagnostics-card";
import { MetricCard } from "./metric-card";

const DEVICE_PREVIEW_LIMIT = 6;

type OverviewShellProps = {
  connectionPlans?: ReactNode;
};

export function OverviewShell({ connectionPlans }: OverviewShellProps) {
  const { t, locale } = useI18n();
  const devices = useWorkspaceDevices();
  const groups = useWorkspaceGroups();
  const tasks = useWorkspaceTasks();
  const diagnostics = useWorkspaceDiagnostics();
  const { loading, refreshing, error, refresh } = useWorkspaceSnapshotStatus();
  const metrics = buildOverviewMetrics({ devices, groups, tasks, diagnostics });

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex items-center justify-between gap-4 border-b border-border px-6 py-4">
        <div className="min-w-0">
          <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
            <Activity className="h-4 w-4" />
            {t("overview.sectionLabel")}
          </div>
          <h1 className="truncate text-xl font-semibold">{t("overview.title")}</h1>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={refreshing}
          onClick={() => void refresh()}
        >
          <RefreshCcw className={cn("h-4 w-4", refreshing && "animate-spin")} />
          {refreshing ? t("common.refreshing") : t("common.refresh")}
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="space-y-6 px-6 py-6">
          {error ? <ErrorState title={t("common.snapshotError")} detail={error} /> : null}

          <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            {metrics.map((metric) => (
              <MetricCard
                key={metric.id}
                icon={metric.icon}
                label={t(metric.labelKey)}
                value={metric.value}
                note={t(metric.noteKey)}
                locale={locale}
              />
            ))}
          </section>

          {loading || (error && devices.length === 0) ? null : devices.length === 0 ? (
            <EmptyState
              icon={Smartphone}
              title={t("overview.empty.title")}
              description={t("overview.empty.description")}
              action={
                <Button type="button" onClick={() => void refresh()}>
                  <RefreshCcw className="h-4 w-4" />
                  {t("common.retry")}
                </Button>
              }
            />
          ) : (
            <section className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
              <Card>
                <CardHeader>
                  <CardTitle>{t("overview.devices.title")}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  {devices.slice(0, DEVICE_PREVIEW_LIMIT).map((device) => (
                    <div
                      key={device.id}
                      className="rounded-md border border-border bg-background p-3"
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <div className="truncate text-sm font-medium">
                            {deviceTitle(device, false, t)}
                          </div>
                          <div className="mt-1 truncate text-xs text-muted-foreground">
                            {textValue(device.model, false, t)}
                          </div>
                        </div>
                        <Badge variant="outline" className={healthClass(device.health)}>
                          {t(
                            runtimeKey(
                              `enums.connectionHealth.${enumKey(ConnectionHealth, device.health)}`,
                            ),
                          )}
                        </Badge>
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>

              <DiagnosticsCard />
            </section>
          )}

          {connectionPlans}
        </div>
      </ScrollArea>
    </div>
  );
}
