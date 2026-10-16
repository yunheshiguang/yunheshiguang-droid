import { RefreshCcw, Smartphone } from "lucide-react";
import { useState } from "react";

import type { DeviceFilter, DeviceSortKey } from "@/entities/device";
import {
  useWorkspaceDevices,
  useWorkspaceGroups,
  useWorkspaceSnapshotStatus,
} from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import { Button, EmptyState, ErrorState } from "@/shared/ui";
import { DeviceWall } from "@/widgets/device-wall";

const initialFilter: DeviceFilter = {
  query: "",
  groupId: "all",
  tag: "all",
  status: "all",
  connection: "all",
  revealPii: false,
};

export function DevicesPage() {
  const { t } = useI18n();
  const devices = useWorkspaceDevices();
  const groups = useWorkspaceGroups();
  const { loading, error, refresh } = useWorkspaceSnapshotStatus();
  const [filter, setFilter] = useState<DeviceFilter>(initialFilter);
  const [sortKey, setSortKey] = useState<DeviceSortKey>("rank");

  if (!loading && devices.length === 0) {
    const retry = (
      <Button type="button" onClick={() => void refresh()}>
        <RefreshCcw className="h-4 w-4" />
        {t("common.retry")}
      </Button>
    );

    return (
      <div className="h-full p-6">
        {error ? (
          <ErrorState
            variant="block"
            title={t("common.snapshotError")}
            detail={error}
            action={retry}
          />
        ) : (
          <EmptyState
            icon={Smartphone}
            title={t("devices.empty.title")}
            description={t("devices.empty.description")}
            action={retry}
          />
        )}
      </div>
    );
  }

  return (
    <DeviceWall
      devices={devices}
      groups={groups}
      filter={filter}
      sortKey={sortKey}
      onFilterChange={setFilter}
      onSortChange={setSortKey}
    />
  );
}
