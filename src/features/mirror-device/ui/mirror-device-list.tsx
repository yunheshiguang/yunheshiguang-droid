import { MonitorPlay, X } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { deviceTitle, textValue } from "@/shared/lib/workspace-view";
import {
  type DeviceRecord,
  DeviceStatus,
  enumKey,
  ScrcpyConnectionHealth,
  ScrcpySessionStatus,
  type ScrcpyState,
} from "@/shared/proto";
import { Button, SectionCard } from "@/shared/ui";

import { useMirrorCommands } from "../model/use-mirror-commands";

const SESSION_TONES = {
  [ScrcpySessionStatus.UNSPECIFIED]: "neutral",
  [ScrcpySessionStatus.IDLE]: "neutral",
  [ScrcpySessionStatus.STARTING]: "info",
  [ScrcpySessionStatus.RUNNING]: "positive",
  [ScrcpySessionStatus.STOPPING]: "info",
  [ScrcpySessionStatus.FAILED]: "critical",
} as const satisfies Record<ScrcpySessionStatus, StatusTone>;

type MirrorDeviceListProps = {
  devices: readonly DeviceRecord[];
  state?: ScrcpyState;
};

export function MirrorDeviceList({ devices, state }: MirrorDeviceListProps) {
  const { t } = useI18n();
  const commands = useMirrorCommands();
  const blocked = !state || state.health === ScrcpyConnectionHealth.BLOCKED;

  return (
    <SectionCard
      icon={MonitorPlay}
      title={t("mirrorDevice.devices.title")}
      contentClassName="space-y-2"
    >
      {devices.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("mirrorDevice.devices.empty")}</p>
      ) : null}
      <ul className="space-y-2">
        {devices.map((device) => {
          const session = state?.sessions.find((candidate) => candidate.serial === device.id);
          const running = session?.status === ScrcpySessionStatus.RUNNING;
          const online = device.status === DeviceStatus.ONLINE;
          const title = deviceTitle(device, false, t);

          return (
            <li
              key={device.id}
              aria-label={title}
              className="flex flex-wrap items-center gap-3 rounded-md border border-border bg-background p-3"
            >
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-medium">{title}</div>
                <div className="truncate text-xs text-muted-foreground">
                  {textValue(device.model, false, t)}
                </div>
                {session ? (
                  <div
                    className={`mt-1 text-xs ${statusTextTones[SESSION_TONES[session.status] ?? "neutral"]}`}
                  >
                    {t(
                      runtimeKey(
                        `enums.scrcpySessionStatus.${enumKey(ScrcpySessionStatus, session.status)}`,
                      ),
                    )}
                    {session.lastError ? ` · ${session.lastError}` : ""}
                  </div>
                ) : null}
              </div>
              {running && session ? (
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={commands.busy}
                  onClick={() => commands.stop(session.id)}
                >
                  <X className="h-4 w-4" />
                  {t("mirrorDevice.devices.stop")}
                </Button>
              ) : (
                <Button
                  type="button"
                  size="sm"
                  disabled={!online || blocked || commands.busy}
                  title={online ? undefined : t("mirrorDevice.devices.notOnline")}
                  onClick={() => commands.launch(device.id)}
                >
                  <MonitorPlay className="h-4 w-4" />
                  {t("mirrorDevice.devices.mirror")}
                </Button>
              )}
            </li>
          );
        })}
      </ul>
    </SectionCard>
  );
}
