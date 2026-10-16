import { AlertTriangle, Cable, DatabaseZap, type LucideIcon, Smartphone } from "lucide-react";

import { countOnlineDevices, type DeviceRecord } from "@/entities/device";
import { type DeviceGroup, operatorGroups } from "@/entities/group";
import type { PlainMessageKey } from "@/shared/i18n";
import { ConnectionHealth, type DiagnosticSignal, type TaskItem } from "@/shared/proto";

export type OverviewMetric = {
  id: string;
  icon: LucideIcon;
  labelKey: PlainMessageKey;
  value: number;
  noteKey: PlainMessageKey;
};

type MetricInput = {
  devices: readonly DeviceRecord[];
  groups: readonly DeviceGroup[];
  tasks: readonly TaskItem[];
  diagnostics: readonly DiagnosticSignal[];
};

export function buildOverviewMetrics({
  devices,
  groups,
  tasks,
  diagnostics,
}: MetricInput): readonly OverviewMetric[] {
  return [
    {
      id: "devices",
      icon: Smartphone,
      labelKey: "overview.metrics.devices",
      value: countOnlineDevices(devices),
      noteKey: "overview.metrics.devicesNote",
    },
    {
      id: "tasks",
      icon: DatabaseZap,
      labelKey: "overview.metrics.tasks",
      value: tasks.length,
      noteKey: "overview.metrics.tasksNote",
    },
    {
      id: "blocked",
      icon: AlertTriangle,
      labelKey: "overview.metrics.blocked",
      value: diagnostics.filter((item) => item.health === ConnectionHealth.BLOCKED).length,
      noteKey: "overview.metrics.blockedNote",
    },
    {
      id: "groups",
      icon: Cable,
      labelKey: "overview.metrics.groups",
      value: operatorGroups(groups).length,
      noteKey: "overview.metrics.groupsNote",
    },
  ];
}
