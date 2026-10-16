import type { WorkspaceExtension } from "@/shared/workspace-module";

import { collectAppMessages } from "./app-messages";
import { discoverWorkspaceModules } from "./discover-workspace-modules";
import { collectWorkspaceExtensions } from "./workspace-extensions";
import { createAppWorkspaceRegistry } from "./workspace-registry";

const checkoutEntrypoints = import.meta.glob<Record<string, unknown>>(
  "../../../extensions/*/web/src/index.ts",
  { eager: true },
);

export type CheckoutExtension = {
  directory: string;
  extension: WorkspaceExtension;
};

export function checkoutExtensions(): CheckoutExtension[] {
  return Object.entries(checkoutEntrypoints).flatMap(([entrypoint, exported]) => {
    const directory = entrypoint.replace(/^(?:\.\.\/)+/, "").replace(/\/src\/index\.ts$/, "");

    return collectWorkspaceExtensions({ [entrypoint]: exported }).map((extension) => ({
      directory,
      extension,
    }));
  });
}

export function fullAssembly() {
  const extensions = checkoutExtensions().map(({ extension }) => extension);
  const modules = [
    ...discoverWorkspaceModules(),
    ...extensions.flatMap((extension) => extension.modules ?? []),
  ];
  const registry = createAppWorkspaceRegistry({
    capabilities: modules.flatMap((module) => module.requires ?? []),
    extensions,
  });

  return { registry, messages: collectAppMessages(registry, extensions) };
}
