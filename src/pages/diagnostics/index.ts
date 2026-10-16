import { AlertTriangle } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import { lazyModulePage, type WorkspaceModuleDefinition } from "@/shared/workspace-module";

export const diagnosticsModule: WorkspaceModuleDefinition = {
  id: "diagnostics",
  component: lazyModulePage(
    () => import("./ui/diagnostics-page"),
    (page) => page.DiagnosticsPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 10,
    labelKey: "modules.diagnostics.label",
    descriptionKey: "modules.diagnostics.description",
    icon: AlertTriangle,
  },
  messages: {
    "zh-CN": {
      modules: {
        diagnostics: { label: "诊断", description: "连接失败原因" },
      },
    },
    "en-US": {
      modules: {
        diagnostics: {
          label: "Diagnostics",
          description: "Connection failures",
        },
      },
    },
  },
};
