import { PanelLeftClose, PanelLeftOpen } from "lucide-react";

import { useI18n } from "@/shared/i18n";
import { Button } from "@/shared/ui";

type SidebarToggleProps = {
  sidebarOpen: boolean;
  onToggle: () => void;
};

export function SidebarToggle({ sidebarOpen, onToggle }: SidebarToggleProps) {
  const { t } = useI18n();

  return (
    <Button
      type="button"
      variant="outline"
      size="icon"
      title={sidebarOpen ? t("windowChrome.hideSidebar") : t("windowChrome.showSidebar")}
      aria-label={sidebarOpen ? t("windowChrome.hideSidebar") : t("windowChrome.showSidebar")}
      className="h-8 w-8 shrink-0 rounded-md"
      onClick={onToggle}
    >
      {sidebarOpen ? <PanelLeftClose className="h-4 w-4" /> : <PanelLeftOpen className="h-4 w-4" />}
    </Button>
  );
}
