import { Unplug } from "lucide-react";

import type { DeviceRecord } from "@/entities/device";
import { useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { Button, SectionCard } from "@/shared/ui";

import { disconnectableDevices } from "../model/network-devices";
import { useDisconnectDevice } from "../model/use-connection-commands";
import { ConnectionOutcome } from "./connection-outcome";

type NetworkDeviceListProps = {
  devices: readonly DeviceRecord[];
};

export function NetworkDeviceList({ devices }: NetworkDeviceListProps) {
  const { t } = useI18n();
  const disconnect = useDisconnectDevice();
  const network = disconnectableDevices(devices);

  return (
    <SectionCard
      icon={Unplug}
      title={t("connectDevice.network.title")}
      contentClassName="space-y-3"
    >
      {network.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("connectDevice.network.empty")}</p>
      ) : (
        <ul className="space-y-2">
          {network.map((device) => {
            const name = textValue(device.name, false, t);

            return (
              <li
                key={device.id}
                className="flex items-center justify-between gap-3 rounded-md border border-border bg-background p-3"
              >
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{name}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {textValue(device.model, false, t)}
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={disconnect.pending}
                  onClick={() => disconnect.run({ deviceId: device.id })}
                  aria-label={t("connectDevice.network.disconnectDevice", { name })}
                >
                  {t("connectDevice.network.disconnect")}
                </Button>
              </li>
            );
          })}
        </ul>
      )}
      <ConnectionOutcome result={disconnect.result} error={disconnect.error} />
    </SectionCard>
  );
}
