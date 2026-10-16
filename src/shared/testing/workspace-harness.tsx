import { TransportProvider } from "@connectrpc/connect-query";
import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { createWorkspaceQueryClient, rpcTransport } from "@/shared/api";
import { I18nProvider, type MessageOverlays } from "@/shared/i18n";
import { mergeModuleMessages, type WorkspaceModuleMessages } from "@/shared/workspace-module";

import { recordMissingCopy } from "./missing-copy";

export type WorkspaceTestHarness = {
  queryClient: QueryClient;
  wrapper: (props: { children: ReactNode }) => ReactNode;
};

export function createWorkspaceTestHarness(
  ...messages: readonly WorkspaceModuleMessages[]
): WorkspaceTestHarness {
  const queryClient = createWorkspaceQueryClient();
  const overlays: MessageOverlays = mergeModuleMessages(...messages);

  return {
    queryClient,
    wrapper: ({ children }) => (
      <I18nProvider
        defaultLocale="en-US"
        messageOverlays={overlays}
        onMissingKey={recordMissingCopy}
      >
        <QueryClientProvider client={queryClient}>
          <TransportProvider transport={rpcTransport()}>{children}</TransportProvider>
        </QueryClientProvider>
      </I18nProvider>
    ),
  };
}
