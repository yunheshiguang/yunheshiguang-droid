import { isPlainObject } from "@/shared/lib/deep-merge";

import type { WorkspaceExtension } from "./types";

export function isWorkspaceExtension(value: unknown): value is WorkspaceExtension {
  if (!isPlainObject(value)) {
    return false;
  }

  const { id, modules, messages } = value;

  return (
    typeof id === "string" &&
    id.length > 0 &&
    (modules === undefined || Array.isArray(modules)) &&
    (messages === undefined || isPlainObject(messages))
  );
}

export function defineWorkspaceExtension(extension: WorkspaceExtension): WorkspaceExtension {
  return extension;
}
