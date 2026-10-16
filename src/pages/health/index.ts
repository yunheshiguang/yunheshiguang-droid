import { Activity } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { healthPageMessages } from "./messages";

export const healthModule: WorkspaceModuleDefinition = {
  id: "health",
  component: lazyModulePage(
    () => import("./ui/health-page"),
    (page) => page.HealthPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 15,
    labelKey: "modules.health.label",
    descriptionKey: "modules.health.description",
    icon: Activity,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": { modules: { health: { label: "健康", description: "在线率和失败率" } } },
      "en-US": {
        modules: { health: { label: "Health", description: "Availability and failure rates" } },
      },
    },
    healthPageMessages,
  ),
};
