import installed from "virtual:workspace-extensions";

import { collectPacks } from "@/shared/lib/pack-registry";
import { isWorkspaceExtension, type WorkspaceExtension } from "@/shared/workspace-module";

export function collectWorkspaceExtensions(
  entrypoints: Record<string, Record<string, unknown>>,
): readonly WorkspaceExtension[] {
  const byId = new Map<string, WorkspaceExtension>();

  for (const extension of collectPacks(entrypoints, isWorkspaceExtension, "extension")) {
    const existing = byId.get(extension.id);

    if (existing === undefined) {
      byId.set(extension.id, extension);
    } else if (existing !== extension) {
      console.warn(`[extension] ${extension.id} is installed twice; keeping the first`);
    }
  }

  return [...byId.values()];
}

export const installedWorkspaceExtensions = collectWorkspaceExtensions(installed);
