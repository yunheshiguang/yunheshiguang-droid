import { open, save } from "@tauri-apps/plugin-dialog";

import type { DialogGateway, PathPickRequest } from "./dialog-gateway";

function filtersFor(request: PathPickRequest) {
  if (request.target === "directory" || !request.extensions?.length) {
    return undefined;
  }

  const extensions = [...request.extensions];

  return [{ name: extensions.map((extension) => extension.toUpperCase()).join(", "), extensions }];
}

export function createTauriDialogGateway(): DialogGateway {
  return {
    async pickPath(request) {
      if (request.target === "save") {
        return save({ defaultPath: request.suggestedName, filters: filtersFor(request) });
      }

      const picked = await open({
        directory: request.target === "directory",
        multiple: false,
        filters: filtersFor(request),
      });

      return typeof picked === "string" ? picked : null;
    },
  };
}
