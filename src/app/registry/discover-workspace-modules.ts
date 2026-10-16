import { collectPacks } from "@/shared/lib/pack-registry";
import {
  isWorkspaceModuleDefinition,
  type WorkspaceModuleDefinition,
} from "@/shared/workspace-module";

const moduleEntrypoints = import.meta.glob<Record<string, unknown>>("../../pages/*/index.ts", {
  eager: true,
});

export function discoverWorkspaceModules(): readonly WorkspaceModuleDefinition[] {
  return collectPacks(moduleEntrypoints, isWorkspaceModuleDefinition, "workspace");
}
