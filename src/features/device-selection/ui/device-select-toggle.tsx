import { useI18n } from "@/shared/i18n";
import { Button } from "@/shared/ui";

import { useDeviceSelectionActions, useIsDeviceSelected } from "../model/selection-store";

export function DeviceSelectToggle({ deviceId }: { deviceId: string }) {
  const { t } = useI18n();
  const selected = useIsDeviceSelected(deviceId);
  const { toggle } = useDeviceSelectionActions();

  return (
    <Button
      type="button"
      variant={selected ? "default" : "outline"}
      size="sm"
      aria-pressed={selected}
      onClick={() => toggle(deviceId)}
    >
      {selected ? t("deviceSelection.selected") : t("deviceSelection.select")}
    </Button>
  );
}
