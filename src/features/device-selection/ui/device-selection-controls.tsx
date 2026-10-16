import { useI18n } from "@/shared/i18n";
import { Badge, Button } from "@/shared/ui";

import {
  countSelected,
  useDeviceSelectionActions,
  useSelectedDeviceIds,
} from "../model/selection-store";

type DeviceSelectionControlsProps = {
  availableIds: readonly string[];
  visibleIds: readonly string[];
};

export function DeviceSelectionControls({
  availableIds,
  visibleIds,
}: DeviceSelectionControlsProps) {
  const { t } = useI18n();
  const selectedIds = useSelectedDeviceIds();
  const { add, clear } = useDeviceSelectionActions();

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Badge variant="outline">
        {t("deviceSelection.count", { count: String(countSelected(availableIds, selectedIds)) })}
      </Badge>
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={visibleIds.length === 0}
        onClick={() => add(visibleIds)}
      >
        {t("deviceSelection.selectVisible")}
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={selectedIds.length === 0}
        onClick={clear}
      >
        {t("deviceSelection.clear")}
      </Button>
    </div>
  );
}
