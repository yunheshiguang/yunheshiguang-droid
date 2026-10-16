import { isPlainObject } from "@/shared/lib/deep-merge";
import {
  mergeModuleMessages,
  type WorkspaceExtension,
  type WorkspaceModuleMessages,
  type WorkspaceRegistry,
} from "@/shared/workspace-module";

import { installedWorkspaceExtensions } from "./workspace-extensions";

const sliceEntrypoints = import.meta.glob<Record<string, unknown>>(
  "../../{entities,features,widgets}/*/messages.ts",
  { eager: true },
);

function isMessageOverlay(value: unknown): value is WorkspaceModuleMessages {
  return isPlainObject(value) && Object.values(value).every(isPlainObject);
}

export type SliceMessageSource = {
  slice: string;
  messages: WorkspaceModuleMessages;
};

export function sliceMessageSources(): SliceMessageSource[] {
  return Object.entries(sliceEntrypoints).flatMap(([entrypoint, exported]) => {
    const slice = entrypoint.replace(/^\.\.\/\.\.\//, "").replace(/\/messages\.ts$/, "");

    return Object.values(exported)
      .filter(isMessageOverlay)
      .map((messages) => ({ slice, messages }));
  });
}

export function collectAppMessages(
  registry: WorkspaceRegistry,
  extensions: readonly WorkspaceExtension[] = installedWorkspaceExtensions,
): WorkspaceModuleMessages {
  return mergeModuleMessages(
    ...sliceMessageSources().map((source) => source.messages),
    ...extensions.map((extension) => extension.messages ?? {}),
    registry.messages,
  );
}
