import type { WorkspaceModuleDefinition, WorkspaceModuleId } from "./types";

function isEnabled(module: WorkspaceModuleDefinition, capabilities: ReadonlySet<string>): boolean {
  return (module.requires ?? []).every((capability) => capabilities.has(capability));
}

function dedupeById(modules: readonly WorkspaceModuleDefinition[]): WorkspaceModuleDefinition[] {
  const byId = new Map<WorkspaceModuleId, WorkspaceModuleDefinition>();

  for (const module of modules) {
    byId.set(module.id, module);
  }

  return [...byId.values()];
}

function applyReplacements(
  modules: readonly WorkspaceModuleDefinition[],
): WorkspaceModuleDefinition[] {
  const replaced = new Set<WorkspaceModuleId>();

  for (const module of modules) {
    for (const target of module.replaces ?? []) {
      if (target !== module.id) {
        replaced.add(target);
      }
    }
  }

  return modules.filter((module) => !replaced.has(module.id));
}

export function selectModules(
  candidates: readonly WorkspaceModuleDefinition[],
  capabilities: ReadonlySet<string>,
): readonly WorkspaceModuleDefinition[] {
  return applyReplacements(
    dedupeById(candidates.filter((module) => isEnabled(module, capabilities))),
  );
}

export function pickDefaultModuleId(
  modules: readonly WorkspaceModuleDefinition[],
  firstNavigationEntryId: WorkspaceModuleId | undefined,
  preferred: WorkspaceModuleId | undefined,
): WorkspaceModuleId {
  if (preferred && modules.some((module) => module.id === preferred)) {
    return preferred;
  }

  return firstNavigationEntryId ?? modules[0]?.id ?? "";
}
