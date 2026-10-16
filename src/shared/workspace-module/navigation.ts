import {
  DEFAULT_MODULE_ORDER,
  type WorkspaceModuleDefinition,
  type WorkspaceNavigationEntry,
  type WorkspaceNavigationGroup,
  type WorkspaceNavigationGroupDefinition,
} from "./types";

function toNavigationEntry(module: WorkspaceModuleDefinition): WorkspaceNavigationEntry {
  return {
    id: module.id,
    labelKey: module.navigation.labelKey,
    descriptionKey: module.navigation.descriptionKey,
    icon: module.navigation.icon,
    stability: module.stability ?? "stable",
  };
}

function bucketByGroup(
  modules: readonly WorkspaceModuleDefinition[],
): Map<string, WorkspaceModuleDefinition[]> {
  const buckets = new Map<string, WorkspaceModuleDefinition[]>();

  for (const module of modules) {
    if (module.navigation.hidden) {
      continue;
    }

    const bucket = buckets.get(module.navigation.groupId);

    if (bucket) {
      bucket.push(module);
      continue;
    }

    buckets.set(module.navigation.groupId, [module]);
  }

  return buckets;
}

export function buildNavigation(
  modules: readonly WorkspaceModuleDefinition[],
  groups: readonly WorkspaceNavigationGroupDefinition[],
): readonly WorkspaceNavigationGroup[] {
  const declared = new Map(groups.map((group) => [group.id, group]));

  return [...bucketByGroup(modules).entries()]
    .map(([groupId, bucket]) => {
      const definition = declared.get(groupId);

      return {
        id: groupId,
        labelKey: definition?.labelKey ?? groupId,
        order: definition?.order ?? DEFAULT_MODULE_ORDER,
        entries: [...bucket]
          .sort(
            (left, right) =>
              (left.navigation.order ?? DEFAULT_MODULE_ORDER) -
              (right.navigation.order ?? DEFAULT_MODULE_ORDER),
          )
          .map(toNavigationEntry),
      };
    })
    .sort((left, right) => left.order - right.order)
    .map(({ id, labelKey, entries }) => ({ id, labelKey, entries }));
}
