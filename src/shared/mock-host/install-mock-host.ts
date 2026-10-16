import { createRouterTransport } from "@connectrpc/connect";

import {
  type CommandGateway,
  setCommandGateway,
  setDialogGateway,
  setLogGateway,
  setRpcTransport,
  setWindowGateway,
} from "@/shared/api";

import { createMockBackend, type MockBackend, type MockBackendOptions } from "./mock-backend";

function mockCommandGateway({ handlers }: MockBackend): CommandGateway {
  const byName: Readonly<Record<string, MockBackend["handlers"][keyof MockBackend["handlers"]]>> =
    handlers;

  return {
    invoke: async <TResult>(command: string, payload?: Record<string, unknown>) => {
      const handler = byName[command];

      if (!handler) {
        throw new Error(`the mock host has no handler for "${command}"`);
      }

      return (await handler(payload)) as TResult;
    },
    subscribe: async () => () => {},
  };
}

export function installMockHost(options?: MockBackendOptions): void {
  const backend = createMockBackend(options);

  setCommandGateway(mockCommandGateway(backend));
  setRpcTransport(createRouterTransport(backend.services));
  setWindowGateway({
    minimize: async () => {},
    close: async () => window.close(),
    toggleMaximize: async () => {},
    isMaximized: async () => false,
    openWorkspaceWindow: async ({ url }) => {
      window.open(url, "_blank", "noopener");
    },
    openExternal: async (url) => {
      window.open(url, "_blank", "noopener,noreferrer");
    },
  });
  setLogGateway({
    error: (message) => console.error(message),
    warn: (message) => console.warn(message),
  });
  const pickLabels = { file: "File path", directory: "Folder path", save: "Save as" } as const;
  setDialogGateway({
    pickPath: async (request) =>
      window.prompt(
        pickLabels[request.target],
        request.target === "save" ? `/tmp/${request.suggestedName}` : "",
      ),
  });
}
