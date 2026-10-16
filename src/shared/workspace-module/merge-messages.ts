import { deepMerge, type PlainObject } from "@/shared/lib/deep-merge";

import type { WorkspaceModuleMessages } from "./types";

export function mergeModuleMessages(
  ...sources: readonly WorkspaceModuleMessages[]
): WorkspaceModuleMessages {
  const merged: Record<string, PlainObject> = {};

  for (const source of sources) {
    for (const [locale, tree] of Object.entries(source)) {
      merged[locale] = deepMerge(merged[locale] ?? {}, tree as PlainObject);
    }
  }

  return merged;
}
