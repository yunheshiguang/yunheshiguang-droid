import { useMemo } from "react";

import { UpdateBadge } from "@/entities/release";
import {
  LanguageSwitcher,
  OpenWindowButton,
  SidebarToggle,
  ThemeSwitcher,
} from "@/features/workspace";
import { workspaceMeta } from "@/shared/config";
import { useI18n } from "@/shared/i18n";
import { detectDesktopPlatform } from "@/shared/lib/platform";
import { BrandMark, Separator } from "@/shared/ui";

import { OperatorBadge } from "./operator-badge";
import { WindowControls } from "./window-controls";

type WindowChromeProps = {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
};

export function WindowChrome({ sidebarOpen, onToggleSidebar }: WindowChromeProps) {
  const { t } = useI18n();
  const platform = useMemo(() => detectDesktopPlatform(), []);
  const isMac = platform === "macos";
  const controls = <WindowControls isMac={isMac} />;

  return (
    <header
      className="flex h-14 items-stretch border-b border-border bg-background/95 text-foreground"
      data-tauri-drag-region="deep"
    >
      {isMac ? controls : null}

      <div className="flex min-w-0 flex-1 items-center gap-3 px-4">
        <SidebarToggle sidebarOpen={sidebarOpen} onToggle={onToggleSidebar} />

        <div className="flex min-w-0 items-center gap-3">
          <BrandMark size="sm" />

          <div className="min-w-0">
            <div className="truncate text-sm font-semibold">{t(workspaceMeta.nameKey)}</div>
            <div className="truncate text-xs text-muted-foreground">
              {t(workspaceMeta.subtitleKey)}
            </div>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2 px-2">
        <UpdateBadge />
        <OperatorBadge />
        <ThemeSwitcher />
        <LanguageSwitcher />

        <OpenWindowButton />
      </div>

      {isMac ? null : (
        <>
          <Separator orientation="vertical" className="my-3 hidden h-8 md:block" />
          {controls}
        </>
      )}
    </header>
  );
}
