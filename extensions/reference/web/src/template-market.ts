import { Store } from "lucide-react";
import {
  lazyModulePage,
  type WorkspaceModuleDefinition,
  workspaceNavigationGroupIds,
} from "yunheshiguang/sdk";

export const templateMarketModule: WorkspaceModuleDefinition = {
  id: "template-market",
  component: lazyModulePage(
    () => import("./template-market-page"),
    (page) => page.TemplateMarketPage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.workspace,
    order: 45,
    labelKey: "modules.templateMarket.label",
    descriptionKey: "modules.templateMarket.description",
    icon: Store,
  },
  requires: ["reference.market"],
  messages: {
    "zh-CN": {
      modules: { templateMarket: { label: "模板市场", description: "现成的流程" } },
      templateMarket: { title: "模板市场" },
    },
    "en-US": {
      modules: { templateMarket: { label: "Template market", description: "Ready-made flows" } },
      templateMarket: { title: "Template market" },
    },
  },
};
