import { workspaceNavigationGroups } from "@/shared/config";
import {
  createWorkspaceRegistry,
  type WorkspaceExtension,
  type WorkspaceRegistry,
} from "@/shared/workspace-module";

import { discoverWorkspaceModules } from "./discover-workspace-modules";
import { installedWorkspaceExtensions } from "./workspace-extensions";

interface WorkspaceRegistryOptions {
  capabilities?: readonly string[];
  preferredDefaultModuleId?: string;
  extensions?: readonly WorkspaceExtension[];
}

export function createAppWorkspaceRegistry(
  options: WorkspaceRegistryOptions = {},
): WorkspaceRegistry {
  const extensions = options.extensions ?? installedWorkspaceExtensions;

  return createWorkspaceRegistry({
    modules: [
      ...discoverWorkspaceModules(),
      ...extensions.flatMap((extension) => extension.modules ?? []),
    ],
    groups: workspaceNavigationGroups,
    capabilities: options.capabilities,
    preferredDefaultModuleId: options.preferredDefaultModuleId ?? "overview",
  });
}
