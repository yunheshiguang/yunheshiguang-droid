import { Workflow } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { automationMessages } from "./messages";

export const automationModule: WorkspaceModuleDefinition = {
  id: "automation",
  component: lazyModulePage(
    () => import("./ui/automation-page"),
    (page) => page.AutomationPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 40,
    labelKey: "modules.automation.label",
    descriptionKey: "modules.automation.description",
    icon: Workflow,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { automation: { label: "自动化", description: "流程、定时任务和运行记录" } },
      },
      "en-US": {
        modules: {
          automation: { label: "Automation", description: "Flows, schedules, and run history" },
        },
      },
    },
    automationMessages,
  ),
};
