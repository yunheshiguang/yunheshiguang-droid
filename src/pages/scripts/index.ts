import { FileCode2 } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import { lazyModulePage, type WorkspaceModuleDefinition } from "@/shared/workspace-module";

export const scriptsModule: WorkspaceModuleDefinition = {
  id: "scripts",
  component: lazyModulePage(
    () => import("./ui/scripts-page"),
    (page) => page.ScriptsPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 45,
    labelKey: "modules.scripts.label",
    descriptionKey: "modules.scripts.description",
    icon: FileCode2,
  },
  messages: {
    "zh-CN": {
      modules: {
        scripts: { label: "脚本", description: "编写和运行脚本" },
      },
    },
    "en-US": {
      modules: {
        scripts: { label: "Scripts", description: "Write and run scripts" },
      },
    },
  },
};
