import { useWindowControls } from "@/features/workspace";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { Button } from "@/shared/ui";

import {
  restoredMaximizeControl,
  type WindowControlId,
  windowControls,
} from "../model/window-controls";

export function WindowControls({ isMac }: { isMac: boolean }) {
  const { t } = useI18n();
  const { isMaximized, minimize, close, toggleMaximize } = useWindowControls();

  const handlers: Record<WindowControlId, () => void> = {
    minimize,
    close,
    maximize: () => void toggleMaximize(),
  };

  return (
    <div className="flex items-center gap-1 px-2">
      {windowControls.map((entry) => {
        const control = isMaximized ? restoredMaximizeControl(entry) : entry;
        const label = t(control.labelKey);
        const Icon = control.icon;

        return (
          <Button
            key={control.id}
            type="button"
            variant={isMac ? "ghost" : control.desktopVariant}
            size="icon"
            title={label}
            aria-label={label}
            className={cn(
              "h-8 w-8 shrink-0",
              isMac ? cn("rounded-full", control.macAccentClassName) : "rounded-md",
            )}
            onClick={handlers[control.id]}
          >
            <Icon className="h-4 w-4" />
          </Button>
        );
      })}
    </div>
  );
}
