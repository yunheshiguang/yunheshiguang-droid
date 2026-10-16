import { Settings2 } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { settingsMessages } from "./messages";

export const settingsModule: WorkspaceModuleDefinition = {
  id: "settings",
  component: lazyModulePage(
    () => import("./ui/settings-page"),
    (page) => page.SettingsPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 60,
    labelKey: "modules.settings.label",
    descriptionKey: "modules.settings.description",
    icon: Settings2,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { settings: { label: "设置", description: "外观、工具和本机 API" } },
      },
      "en-US": {
        modules: {
          settings: {
            label: "Settings",
            description: "Appearance, tools, and the local API",
          },
        },
      },
    },
    settingsMessages,
  ),
};
