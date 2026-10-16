import { PanelsTopLeft } from "lucide-react";

import { windowGateway } from "@/shared/api";
import { useI18n } from "@/shared/i18n";
import { Button } from "@/shared/ui";

function createWindowLabel() {
  return `workspace_${Date.now().toString(36)}`;
}

export function OpenWindowButton() {
  const { t } = useI18n();

  return (
    <Button
      type="button"
      variant="secondary"
      className="h-8 gap-2 rounded-md px-3 text-xs"
      onClick={() => {
        void windowGateway().openWorkspaceWindow({
          label: createWindowLabel(),
          url: window.location.pathname + window.location.search,
          title: t("app.name"),
          width: 896,
          height: 560,
          decorations: false,
          resizable: true,
        });
      }}
    >
      <PanelsTopLeft className="h-4 w-4" />
      {t("header.actionLabel")}
    </Button>
  );
}
