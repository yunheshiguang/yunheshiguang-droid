import { Battery, Gauge, Smartphone } from "lucide-react";
import { useId } from "react";

import type { DeviceRecord } from "@/entities/device";
import { DeviceSelectToggle, useIsDeviceSelected } from "@/features/device-selection";
import { DeviceProfileButton } from "@/features/organize-devices";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { formatLatency, formatMetricValue, formatPercent } from "@/shared/lib/text";
import { cn } from "@/shared/lib/utils";
import { deviceTitle, healthClass, statusClass, textValue } from "@/shared/lib/workspace-view";
import { ConnectionHealth, ConnectionType, DeviceStatus, enumKey } from "@/shared/proto";
import { Badge } from "@/shared/ui";

type DeviceCardProps = {
  device: DeviceRecord;
  revealPii: boolean;
  onEditProfile: (device: DeviceRecord) => void;
};

export function DeviceCard({ device, revealPii, onEditProfile }: DeviceCardProps) {
  const { t, locale } = useI18n();
  const selected = useIsDeviceSelected(device.id);
  const labels = device.labels;
  const title = deviceTitle(device, revealPii, t);
  const titleId = useId();
  const subtitle = labels?.alias
    ? `${textValue(device.name, revealPii, t)} · ${textValue(device.model, revealPii, t)}`
    : textValue(device.model, revealPii, t);

  return (
    <article
      aria-labelledby={titleId}
      className={cn(
        "rounded-md border bg-card p-3 text-card-foreground transition-colors",
        selected ? "border-primary" : "border-border",
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Smartphone className="h-4 w-4 shrink-0 text-muted-foreground" />
            {labels?.number ? (
              <span className="shrink-0 font-mono text-xs text-muted-foreground">
                #{labels.number}
              </span>
            ) : null}
            <h3 id={titleId} className="truncate text-sm font-semibold">
              {title}
            </h3>
          </div>
          <p className="mt-1 truncate text-xs text-muted-foreground">{subtitle}</p>
        </div>
        <div className="flex shrink-0 items-center">
          <DeviceProfileButton title={title} onOpen={() => onEditProfile(device)} />
          <DeviceSelectToggle deviceId={device.id} />
        </div>
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        <Badge variant="outline" className={statusClass(enumKey(DeviceStatus, device.status))}>
          {t(runtimeKey(`enums.deviceStatus.${enumKey(DeviceStatus, device.status)}`))}
        </Badge>
        <Badge variant="outline" className={healthClass(device.health)}>
          {t(runtimeKey(`enums.connectionHealth.${enumKey(ConnectionHealth, device.health)}`))}
        </Badge>
        <Badge variant="outline">
          {t(runtimeKey(`enums.connectionType.${enumKey(ConnectionType, device.connection)}`))}
        </Badge>
        {(labels?.tags ?? []).map((tag) => (
          <Badge key={tag} variant="secondary">
            {tag}
          </Badge>
        ))}
      </div>

      <div className="mt-3 grid grid-cols-3 gap-2 text-xs">
        <Metric
          icon={Battery}
          label={t("deviceWall.battery")}
          value={formatPercent(device.batteryPercent, locale)}
        />
        <Metric
          icon={Gauge}
          label={t("deviceWall.fps")}
          value={formatMetricValue(device.fps, locale)}
        />
        <Metric
          icon={Gauge}
          label={t("deviceWall.latency")}
          value={formatLatency(device.latencyMs, locale)}
        />
      </div>

      {labels?.note ? (
        <p className="mt-3 line-clamp-2 whitespace-pre-wrap text-xs leading-5">
          {textValue(labels.note, revealPii, t)}
        </p>
      ) : null}
      <p className="mt-1 line-clamp-2 text-xs leading-5 text-muted-foreground">
        {textValue(device.note, revealPii, t)}
      </p>
    </article>
  );
}

type MetricProps = {
  icon: typeof Battery;
  label: string;
  value: string;
};

function Metric({ icon: Icon, label, value }: MetricProps) {
  return (
    <div className="rounded-md bg-muted px-2 py-2">
      <div className="flex items-center gap-1 text-muted-foreground">
        <Icon className="h-3.5 w-3.5" />
        <span className="truncate">{label}</span>
      </div>
      <div className="mt-1 truncate font-medium">{value}</div>
    </div>
  );
}
