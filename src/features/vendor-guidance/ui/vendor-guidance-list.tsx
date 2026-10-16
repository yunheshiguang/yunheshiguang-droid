import { Factory } from "lucide-react";
import { useMemo } from "react";

import type { DeviceRecord } from "@/entities/device";
import { useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { DeviceStatus } from "@/shared/proto";
import { ErrorState, SectionCard } from "@/shared/ui";

import { useDeviceVendors } from "../model/use-device-vendors";

type VendorGuidanceListProps = {
  devices: readonly DeviceRecord[];
};

export function VendorGuidanceList({ devices }: VendorGuidanceListProps) {
  const { t } = useI18n();
  const online = useMemo(
    () =>
      devices.filter((device) => device.status === DeviceStatus.ONLINE).map((device) => device.id),
    [devices],
  );
  const { vendors, loading, error } = useDeviceVendors(online);
  const names = new Map(devices.map((device) => [device.id, textValue(device.name, false, t)]));

  return (
    <SectionCard icon={Factory} title={t("vendorGuidance.title")} contentClassName="space-y-3">
      {error ? (
        <ErrorState title={t("vendorGuidance.failed")} detail={error} />
      ) : loading ? (
        <p className="text-sm text-muted-foreground">{t("vendorGuidance.loading")}</p>
      ) : vendors.vendors.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("vendorGuidance.empty")}</p>
      ) : (
        vendors.vendors.map((vendor) => {
          const label = textValue(vendor.label, true, t) || vendor.id;
          const owned = vendors.devices
            .filter((device) => device.vendorId === vendor.id)
            .map((device) => names.get(device.deviceId) ?? device.deviceId);

          return (
            <section
              key={vendor.id}
              aria-label={label}
              className="space-y-2 rounded-md border border-border bg-background p-3"
            >
              <div>
                <h3 className="text-sm font-medium">{label}</h3>
                <p className="text-xs text-muted-foreground">
                  {t("vendorGuidance.devices", { names: owned.join(", ") })}
                </p>
              </div>
              <ol className="list-decimal space-y-1 pl-5 text-xs leading-5">
                {vendor.guidance.map((step) => (
                  <li key={step.key}>{textValue(step, true, t)}</li>
                ))}
              </ol>
            </section>
          );
        })
      )}
    </SectionCard>
  );
}
