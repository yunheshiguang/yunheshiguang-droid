import { Layers3 } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { batchMessages } from "./messages";

export const batchModule: WorkspaceModuleDefinition = {
  id: "batch",
  component: lazyModulePage(
    () => import("./ui/batch-page"),
    (page) => page.BatchPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 30,
    labelKey: "modules.batch.label",
    descriptionKey: "modules.batch.description",
    icon: Layers3,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: {
          batch: { label: "批量操作", description: "安装、上传和输入" },
        },
      },
      "en-US": {
        modules: {
          batch: { label: "Batch", description: "Install, upload, and input" },
        },
      },
    },
    batchMessages,
  ),
};
