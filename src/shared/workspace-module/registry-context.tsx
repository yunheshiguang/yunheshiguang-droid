import {
  createContext,
  type ReactNode,
  startTransition,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";

import type { WorkspaceModuleId, WorkspaceRegistry } from "./types";

const WorkspaceRegistryContext = createContext<WorkspaceRegistry | null>(null);

interface WorkspaceRegistryProviderProps {
  registry: WorkspaceRegistry;
  children: ReactNode;
}

export function WorkspaceRegistryProvider({ registry, children }: WorkspaceRegistryProviderProps) {
  return (
    <WorkspaceRegistryContext.Provider value={registry}>
      {children}
    </WorkspaceRegistryContext.Provider>
  );
}

export function useWorkspaceRegistry(): WorkspaceRegistry {
  const registry = useContext(WorkspaceRegistryContext);

  if (!registry) {
    throw new Error("useWorkspaceRegistry must be used inside WorkspaceRegistryProvider");
  }

  return registry;
}

export function useActiveWorkspaceModule(registry: WorkspaceRegistry) {
  const [requestedId, setRequestedId] = useState<WorkspaceModuleId>(registry.defaultModuleId);
  const selectModule = useCallback((moduleId: WorkspaceModuleId) => {
    startTransition(() => setRequestedId(moduleId));
  }, []);
  const activeModule = useMemo(
    () => registry.resolveOrDefault(requestedId),
    [registry, requestedId],
  );

  return {
    activeModule,
    activeModuleId: activeModule?.id ?? registry.defaultModuleId,
    selectModule,
  };
}
