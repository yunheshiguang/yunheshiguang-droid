import { useMemo } from "react";

import {
  collectDeviceTags,
  type DeviceFilter,
  type DeviceRecord,
  type DeviceSortKey,
  filterDevices,
  sortDevices,
  withKnownFacets,
} from "@/entities/device";
import type { DeviceGroup } from "@/entities/group";
import { useSelectedDeviceIds } from "@/features/device-selection";
import { OrganizeSelectionBar } from "@/features/organize-devices";
import { useI18n } from "@/shared/i18n";

import { DeviceFilterBar } from "./device-filter-bar";
import { DeviceGrid } from "./device-grid";
import { DeviceGroupList } from "./device-group-list";

type DeviceWallProps = {
  devices: readonly DeviceRecord[];
  groups: readonly DeviceGroup[];
  filter: DeviceFilter;
  sortKey: DeviceSortKey;
  onFilterChange: (filter: DeviceFilter) => void;
  onSortChange: (sortKey: DeviceSortKey) => void;
};

export function DeviceWall({
  devices,
  groups,
  filter,
  sortKey,
  onFilterChange,
  onSortChange,
}: DeviceWallProps) {
  const { t } = useI18n();
  const tags = useMemo(() => collectDeviceTags(devices), [devices]);
  const current = useMemo(
    () =>
      withKnownFacets(
        filter,
        groups.map((group) => group.id),
        tags,
      ),
    [filter, groups, tags],
  );
  const visible = useMemo(
    () => sortDevices(filterDevices(devices, current, t), sortKey),
    [devices, current, sortKey, t],
  );
  const availableIds = useMemo(() => devices.map((device) => device.id), [devices]);
  const visibleIds = useMemo(() => visible.map((device) => device.id), [visible]);
  const selectedIds = useSelectedDeviceIds();
  const orderedSelection = useMemo(() => {
    const selected = new Set(selectedIds);

    return sortDevices(devices, sortKey)
      .filter((device) => selected.has(device.id))
      .map((device) => device.id);
  }, [devices, selectedIds, sortKey]);

  return (
    <div className="flex h-full flex-col">
      <DeviceFilterBar
        filter={current}
        sortKey={sortKey}
        groups={groups}
        tags={tags}
        availableIds={availableIds}
        visibleIds={visibleIds}
        onFilterChange={onFilterChange}
        onSortChange={onSortChange}
      />

      <div className="px-4 pt-3 empty:hidden">
        <OrganizeSelectionBar deviceIds={orderedSelection} groups={groups} />
      </div>

      <div className="grid min-h-0 flex-1 gap-0 xl:grid-cols-[1fr_18rem]">
        <DeviceGrid devices={visible} groups={groups} revealPii={current.revealPii} />

        <div className="border-l border-border bg-muted/20 p-4">
          <DeviceGroupList
            groups={groups}
            onSelectGroup={(groupId) => onFilterChange({ ...current, groupId })}
          />
        </div>
      </div>
    </div>
  );
}
