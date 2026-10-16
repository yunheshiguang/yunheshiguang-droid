import { PackageOpen } from "lucide-react";
import { useMemo } from "react";

import { pickSelectedDevices, useSelectedDeviceIds } from "@/features/device-selection";
import { useWorkspaceDevices, useWorkspaceSnapshotStatus } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import { EmptyState } from "@/shared/ui";
import { BatchPanel } from "@/widgets/batch-panel";

export function BatchPage() {
  const { t } = useI18n();
  const devices = useWorkspaceDevices();
  const { loading } = useWorkspaceSnapshotStatus();
  const selectedIds = useSelectedDeviceIds();
  const selectedDevices = useMemo(
    () => pickSelectedDevices(devices, selectedIds),
    [devices, selectedIds],
  );

  if (devices.length === 0) {
    return loading ? null : (
      <div className="h-full p-6">
        <EmptyState
          icon={PackageOpen}
          title={t("batch.empty.title")}
          description={t("batch.empty.description")}
        />
      </div>
    );
  }

  return <BatchPanel selectedDevices={selectedDevices} devices={devices} />;
}
