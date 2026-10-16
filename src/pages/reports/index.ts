import { ChartColumn } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { reportsPageMessages } from "./messages";

export const reportsModule: WorkspaceModuleDefinition = {
  id: "reports",
  component: lazyModulePage(
    () => import("./ui/reports-page"),
    (page) => page.ReportsPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 17,
    labelKey: "modules.reports.label",
    descriptionKey: "modules.reports.description",
    icon: ChartColumn,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { reports: { label: "报表", description: "按操作者、分组和日期统计操作" } },
      },
      "en-US": {
        modules: {
          reports: { label: "Reports", description: "Operations by operator, group, and day" },
        },
      },
    },
    reportsPageMessages,
  ),
};
