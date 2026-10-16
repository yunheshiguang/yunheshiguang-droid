import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";

import { isWorkspaceModuleDefinition } from "./contract";
import { buildNavigation } from "./navigation";
import { pickDefaultModuleId, selectModules } from "./selection";
import type {
  WorkspaceModuleDefinition,
  WorkspaceModuleMessages,
  WorkspaceRegistry,
  WorkspaceRegistryInput,
} from "./types";

function collectMessages(modules: readonly WorkspaceModuleDefinition[]): WorkspaceModuleMessages {
  const byLocale: Record<string, PlainObject> = {};

  for (const module of modules) {
    for (const [locale, messages] of Object.entries(module.messages ?? {})) {
      byLocale[locale] = deepMerge(byLocale[locale] ?? {}, messages as PlainObject);
    }
  }

  return byLocale;
}

export function createWorkspaceRegistry(input: WorkspaceRegistryInput): WorkspaceRegistry {
  const capabilities = new Set(input.capabilities ?? []);
  const modules = selectModules(input.modules.filter(isWorkspaceModuleDefinition), capabilities);
  const navigation = buildNavigation(modules, input.groups ?? []);
  const defaultModuleId = pickDefaultModuleId(
    modules,
    navigation[0]?.entries[0]?.id,
    input.preferredDefaultModuleId,
  );
  const byId = new Map(modules.map((module) => [module.id, module]));

  return {
    modules,
    navigation,
    defaultModuleId,
    messages: collectMessages(modules),
    has: (id) => byId.has(id),
    resolve: (id) => byId.get(id),
    resolveOrDefault: (id) => byId.get(id) ?? byId.get(defaultModuleId),
  };
}

export { isWorkspaceModuleDefinition };
