import { Command } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { protocolMessages } from "./messages";

export const protocolModule: WorkspaceModuleDefinition = {
  id: "protocol",
  component: lazyModulePage(
    () => import("./ui/protocol-page"),
    (page) => page.ProtocolPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 40,
    labelKey: "modules.protocol.label",
    descriptionKey: "modules.protocol.description",
    icon: Command,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { protocol: { label: "协议", description: "协议版本和工具路径" } },
      },
      "en-US": {
        modules: {
          protocol: { label: "Protocol", description: "Schema version and tool paths" },
        },
      },
    },
    protocolMessages,
  ),
};
