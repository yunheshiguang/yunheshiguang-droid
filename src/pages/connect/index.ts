import { Cable } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import { lazyModulePage, type WorkspaceModuleDefinition } from "@/shared/workspace-module";

export const connectModule: WorkspaceModuleDefinition = {
  id: "connect",
  component: lazyModulePage(
    () => import("./ui/connect-page"),
    (page) => page.ConnectPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 15,
    labelKey: "modules.connect.label",
    descriptionKey: "modules.connect.description",
    icon: Cable,
  },
  messages: {
    "zh-CN": {
      modules: { connect: { label: "连接", description: "网络设备、云手机和连接引导" } },
    },
    "en-US": {
      modules: {
        connect: {
          label: "Connect",
          description: "Network devices, cloud phones, and connection guides",
        },
      },
    },
  },
};
