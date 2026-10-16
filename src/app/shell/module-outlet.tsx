import { PackageOpen } from "lucide-react";
import { Suspense } from "react";

import { useI18n } from "@/shared/i18n";
import { Button, EmptyState, ErrorBoundary, ErrorState } from "@/shared/ui";
import type { WorkspaceModuleDefinition, WorkspaceModuleId } from "@/shared/workspace-module";

interface ModuleOutletProps {
  module: WorkspaceModuleDefinition | undefined;
  moduleId: WorkspaceModuleId;
}

export function ModuleOutlet({ module, moduleId }: ModuleOutletProps) {
  const { t } = useI18n();
  const ModulePage = module?.component;

  if (!ModulePage) {
    return (
      <div className="h-full overflow-auto p-6">
        <EmptyState
          icon={PackageOpen}
          title={t("nav.empty.title")}
          description={t("nav.empty.description")}
        />
      </div>
    );
  }

  return (
    <ErrorBoundary
      resetKey={moduleId}
      fallback={(error, reset) => (
        <div className="h-full overflow-auto p-6">
          <ErrorState
            variant="block"
            title={t("module.error.title")}
            detail={error.message}
            action={
              <Button type="button" variant="outline" size="sm" onClick={reset}>
                {t("module.error.retry")}
              </Button>
            }
          />
        </div>
      )}
    >
      <Suspense fallback={null}>
        <div key={moduleId} className="h-full overflow-auto">
          <ModulePage />
        </div>
      </Suspense>
    </ErrorBoundary>
  );
}
