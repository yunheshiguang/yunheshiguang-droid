import type { WorkspaceModuleDefinition } from "./types";

function isRenderable(value: unknown): boolean {
  return typeof value === "function" || (typeof value === "object" && value !== null);
}

export function isWorkspaceModuleDefinition(value: unknown): value is WorkspaceModuleDefinition {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Partial<WorkspaceModuleDefinition>;
  const navigation = candidate.navigation;

  if (typeof candidate.id !== "string" || candidate.id.length === 0) {
    return false;
  }

  if (!isRenderable(candidate.component)) {
    return false;
  }

  if (typeof navigation !== "object" || navigation === null) {
    return false;
  }

  return (
    typeof navigation.groupId === "string" &&
    typeof navigation.labelKey === "string" &&
    typeof navigation.descriptionKey === "string" &&
    isRenderable(navigation.icon)
  );
}
