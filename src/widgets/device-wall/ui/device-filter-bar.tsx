import {
  connectionTypeFacets,
  type DeviceFilter,
  type DeviceSortKey,
  deviceStatusFacets,
  listDeviceSorters,
} from "@/entities/device";
import type { DeviceGroup } from "@/entities/group";
import { DeviceSelectionControls } from "@/features/device-selection";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { ConnectionType, DeviceStatus, enumKey } from "@/shared/proto";
import { Badge, Button, Card, CardHeader, CardTitle, Input, Select } from "@/shared/ui";

export type DeviceFilterBarProps = {
  filter: DeviceFilter;
  sortKey: DeviceSortKey;
  groups: readonly DeviceGroup[];
  tags: readonly string[];
  availableIds: readonly string[];
  visibleIds: readonly string[];
  onFilterChange: (filter: DeviceFilter) => void;
  onSortChange: (sortKey: DeviceSortKey) => void;
};

export function DeviceFilterBar({
  filter,
  sortKey,
  groups,
  tags,
  availableIds,
  visibleIds,
  onFilterChange,
  onSortChange,
}: DeviceFilterBarProps) {
  const { t } = useI18n();

  return (
    <Card className="rounded-none border-x-0 border-t-0">
      <CardHeader className="gap-4">
        <div className="flex items-center justify-between gap-4">
          <CardTitle>{t("deviceWall.title")}</CardTitle>
          <Badge variant="outline">{visibleIds.length}</Badge>
        </div>

        <div className="grid gap-3 xl:grid-cols-[1.2fr_0.8fr_0.8fr_0.8fr_0.8fr_auto]">
          <Input
            value={filter.query}
            aria-label={t("deviceWall.search")}
            placeholder={t("deviceWall.search")}
            onChange={(event) => onFilterChange({ ...filter, query: event.target.value })}
          />
          <Select
            value={filter.groupId}
            aria-label={t("deviceWall.group")}
            onChange={(event) => onFilterChange({ ...filter, groupId: event.target.value })}
          >
            <option value="all">{t("deviceWall.groupAll")}</option>
            {groups.map((group) => (
              <option key={group.id} value={group.id}>
                {textValue(group.name, false, t)}
              </option>
            ))}
          </Select>
          <Select
            value={filter.tag}
            aria-label={t("deviceWall.tag")}
            disabled={tags.length === 0 && filter.tag === "all"}
            onChange={(event) => onFilterChange({ ...filter, tag: event.target.value })}
          >
            <option value="all">{t("deviceWall.tagAll")}</option>
            {tags.map((tag) => (
              <option key={tag} value={tag}>
                {tag}
              </option>
            ))}
          </Select>
          <Select
            value={filter.status}
            aria-label={t("deviceWall.status")}
            onChange={(event) =>
              onFilterChange({
                ...filter,
                status: event.target.value === "all" ? "all" : Number(event.target.value),
              })
            }
          >
            <option value="all">{t("deviceWall.statusAll")}</option>
            {deviceStatusFacets.map((status) => (
              <option key={status} value={status}>
                {t(runtimeKey(`enums.deviceStatus.${enumKey(DeviceStatus, status)}`))}
              </option>
            ))}
          </Select>
          <Select
            value={filter.connection}
            aria-label={t("deviceWall.connection")}
            onChange={(event) =>
              onFilterChange({
                ...filter,
                connection: event.target.value === "all" ? "all" : Number(event.target.value),
              })
            }
          >
            <option value="all">{t("deviceWall.connectionAll")}</option>
            {connectionTypeFacets.map((connection) => (
              <option key={connection} value={connection}>
                {t(runtimeKey(`enums.connectionType.${enumKey(ConnectionType, connection)}`))}
              </option>
            ))}
          </Select>
          <Button
            type="button"
            variant={filter.revealPii ? "default" : "outline"}
            aria-pressed={filter.revealPii}
            onClick={() => onFilterChange({ ...filter, revealPii: !filter.revealPii })}
          >
            {t("deviceWall.reveal")}
          </Button>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap gap-2">
            {listDeviceSorters().map((sorter) => (
              <Button
                key={sorter.key}
                type="button"
                variant={sortKey === sorter.key ? "default" : "outline"}
                size="sm"
                aria-pressed={sortKey === sorter.key}
                onClick={() => onSortChange(sorter.key)}
              >
                {t(sorter.labelKey)}
              </Button>
            ))}
          </div>

          <DeviceSelectionControls availableIds={availableIds} visibleIds={visibleIds} />
        </div>
      </CardHeader>
    </Card>
  );
}
