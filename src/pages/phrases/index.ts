import { MessageSquareText } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

export const phrasesModule: WorkspaceModuleDefinition = {
  id: "phrases",
  component: lazyModulePage(
    () => import("./ui/phrases-page"),
    (page) => page.PhrasesPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 35,
    labelKey: "modules.phrases.label",
    descriptionKey: "modules.phrases.description",
    icon: MessageSquareText,
  },
  messages: mergeModuleMessages({
    "zh-CN": { modules: { phrases: { label: "文本库", description: "可复用的文本片段" } } },
    "en-US": {
      modules: { phrases: { label: "Text library", description: "Reusable text snippets" } },
    },
  }),
};
