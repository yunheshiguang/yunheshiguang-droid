export { defineWorkspaceExtension, isWorkspaceExtension } from "./extension";
export { lazyModulePage, preloadModulePage } from "./lazy-page";
export { mergeModuleMessages } from "./merge-messages";
export { createWorkspaceRegistry, isWorkspaceModuleDefinition } from "./registry";
export {
  useActiveWorkspaceModule,
  useWorkspaceRegistry,
  WorkspaceRegistryProvider,
} from "./registry-context";
export type {
  WorkspaceExtension,
  WorkspaceModuleDefinition,
  WorkspaceModuleId,
  WorkspaceModuleMessages,
  WorkspaceModuleNavigation,
  WorkspaceNavigationGroup,
  WorkspaceNavigationGroupDefinition,
  WorkspaceRegistry,
  WorkspaceRegistryInput,
} from "./types";
