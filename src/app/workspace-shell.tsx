import { TransportProvider } from "@connectrpc/connect-query";
import { QueryClientProvider } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";

import { useGrantedCapabilities } from "@/features/workspace-data";
import { createWorkspaceQueryClient, rpcTransport } from "@/shared/api";
import { workspaceMeta } from "@/shared/config";
import { I18nProvider, useI18n } from "@/shared/i18n";
import { ThemeProvider } from "@/shared/theme";
import { LoadingScreen } from "@/shared/ui";
import {
  useActiveWorkspaceModule,
  useWorkspaceRegistry,
  type WorkspaceExtension,
  type WorkspaceRegistry,
  WorkspaceRegistryProvider,
} from "@/shared/workspace-module";
import { Sidebar } from "@/widgets/sidebar";
import { WindowChrome } from "@/widgets/window-chrome";

import { reportMissingCopy } from "./error-reporting";
import { collectAppMessages, createAppWorkspaceRegistry } from "./registry";
import {
  ModuleOutlet,
  useAutomationSync,
  useBootSnapshot,
  useBootSplashTeardown,
  useCloudPhonesSync,
  useDeviceProfileSync,
  useDeviceWatchSync,
  useDocumentSync,
  useRestoreLoading,
  useShellInputGuards,
  useUpdateSync,
} from "./shell";

function WorkspaceShellInner() {
  const { locale, t } = useI18n();
  const registry = useWorkspaceRegistry();
  const { activeModule, activeModuleId, selectModule } = useActiveWorkspaceModule(registry);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const restoreLoading = useRestoreLoading();

  useShellInputGuards();
  useBootSplashTeardown();
  useBootSnapshot();
  useDocumentSync();
  useDeviceWatchSync();
  useDeviceProfileSync();
  useAutomationSync();
  useCloudPhonesSync();
  useUpdateSync();

  useEffect(() => {
    document.documentElement.lang = locale;
    document.title = t(workspaceMeta.nameKey);
  }, [locale, t]);

  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-background text-foreground">
      <WindowChrome
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen((value) => !value)}
      />

      <div className="flex min-h-0 flex-1 overflow-hidden">
        {sidebarOpen ? (
          <Sidebar
            groups={registry.navigation}
            activeModuleId={activeModuleId}
            onSelectModule={selectModule}
          />
        ) : null}

        <main className="min-w-0 flex-1 overflow-hidden">
          <ModuleOutlet module={activeModule} moduleId={activeModuleId} />
        </main>
      </div>

      {restoreLoading ? <LoadingScreen label={t("loading.restore")} /> : null}
    </div>
  );
}

interface WorkspaceShellProps {
  registry?: WorkspaceRegistry;
  extensions?: readonly WorkspaceExtension[];
}

function WorkspaceRegistryScope({ registry, extensions }: WorkspaceShellProps) {
  const capabilities = useGrantedCapabilities();
  const resolvedRegistry = useMemo(
    () => registry ?? createAppWorkspaceRegistry({ capabilities, extensions }),
    [registry, capabilities, extensions],
  );
  const messages = useMemo(
    () => collectAppMessages(resolvedRegistry, extensions),
    [resolvedRegistry, extensions],
  );

  return (
    <I18nProvider messageOverlays={messages} onMissingKey={reportMissingCopy}>
      <WorkspaceRegistryProvider registry={resolvedRegistry}>
        <WorkspaceShellInner />
      </WorkspaceRegistryProvider>
    </I18nProvider>
  );
}

export function WorkspaceShell({ registry, extensions }: WorkspaceShellProps = {}) {
  const [queryClient] = useState(createWorkspaceQueryClient);

  return (
    <QueryClientProvider client={queryClient}>
      <TransportProvider transport={rpcTransport()}>
        <ThemeProvider>
          <WorkspaceRegistryScope registry={registry} extensions={extensions} />
        </ThemeProvider>
      </TransportProvider>
    </QueryClientProvider>
  );
}
