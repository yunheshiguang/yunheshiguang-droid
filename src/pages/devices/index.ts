import { Smartphone } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { devicesMessages } from "./messages";

export const devicesModule: WorkspaceModuleDefinition = {
  id: "devices",
  component: lazyModulePage(
    () => import("./ui/devices-page"),
    (page) => page.DevicesPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 20,
    labelKey: "modules.devices.label",
    descriptionKey: "modules.devices.description",
    icon: Smartphone,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: {
          devices: { label: "设备墙", description: "查看和选择设备" },
        },
      },
      "en-US": {
        modules: {
          devices: {
            label: "Device wall",
            description: "View and select devices",
          },
        },
      },
    },
    devicesMessages,
  ),
};
