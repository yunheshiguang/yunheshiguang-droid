import { MonitorSmartphone } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { mirrorPageMessages } from "./messages";

export const mirrorModule: WorkspaceModuleDefinition = {
  id: "mirror",
  component: lazyModulePage(
    () => import("./ui/mirror-page"),
    (page) => page.MirrorPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 25,
    labelKey: "modules.mirror.label",
    descriptionKey: "modules.mirror.description",
    icon: MonitorSmartphone,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": { modules: { mirror: { label: "投屏", description: "查看和操作设备" } } },
      "en-US": {
        modules: { mirror: { label: "Mirroring", description: "View and control devices" } },
      },
    },
    mirrorPageMessages,
  ),
};
