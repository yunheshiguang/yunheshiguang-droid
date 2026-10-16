import { ShieldCheck } from "lucide-react";

import { workspaceNavigationGroupIds } from "@/shared/config";
import {
  lazyModulePage,
  mergeModuleMessages,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

import { governancePageMessages } from "./messages";

export const governanceModule: WorkspaceModuleDefinition = {
  id: "governance",
  component: lazyModulePage(
    () => import("./ui/governance-page"),
    (page) => page.GovernancePage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 20,
    labelKey: "modules.governance.label",
    descriptionKey: "modules.governance.description",
    icon: ShieldCheck,
  },
  messages: mergeModuleMessages(
    {
      "zh-CN": {
        modules: { governance: { label: "治理", description: "授权、审计和隐私" } },
      },
      "en-US": {
        modules: {
          governance: { label: "Governance", description: "Authorization, audit, and privacy" },
        },
      },
    },
    governancePageMessages,
  ),
};
