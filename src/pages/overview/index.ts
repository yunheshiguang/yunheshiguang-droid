import { LayoutDashboard } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import { lazyModulePage, type WorkspaceModuleDefinition } from "@/shared/workspace-module";

export const overviewModule: WorkspaceModuleDefinition = {
  id: "overview",
  component: lazyModulePage(
    () => import("./ui/overview-page"),
    (page) => page.OverviewPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 10,
    labelKey: "modules.overview.label",
    descriptionKey: "modules.overview.description",
    icon: LayoutDashboard,
  },
  messages: {
    "zh-CN": {
      modules: {
        overview: { label: "总览", description: "连接、任务和告警" },
      },
    },
    "en-US": {
      modules: {
        overview: {
          label: "Overview",
          description: "Connections, tasks, and alerts",
        },
      },
    },
  },
};
