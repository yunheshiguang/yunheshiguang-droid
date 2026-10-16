import type { WorkspaceNavigationGroupDefinition } from "@/shared/workspace-module";

export const workspaceNavigationGroupIds = {
  workspace: "workspace",
  riskSurface: "risk-surface",
} as const;

export const workspaceNavigationGroups = [
  {
    id: workspaceNavigationGroupIds.workspace,
    order: 10,
    labelKey: "nav.workspace",
  },
  {
    id: workspaceNavigationGroupIds.riskSurface,
    order: 20,
    labelKey: "nav.riskSurface",
  },
] as const satisfies readonly WorkspaceNavigationGroupDefinition[];
