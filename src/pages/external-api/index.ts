import { TerminalSquare } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { externalApiMessages } from "./messages";

export const externalApiModule: WorkspaceModuleDefinition = {
  id: "external-api",
  component: lazyModulePage(
    () => import("./ui/external-api-page"),
    (page) => page.ExternalApiPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 50,
    labelKey: "modules.externalApi.label",
    descriptionKey: "modules.externalApi.description",
    icon: TerminalSquare,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { externalApi: { label: "API", description: "本机 API 与可调用的过程" } },
      },
      "en-US": {
        modules: {
          externalApi: { label: "API", description: "Local API and callable procedures" },
        },
      },
    },
    externalApiMessages,
  ),
};
