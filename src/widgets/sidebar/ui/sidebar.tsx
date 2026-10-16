import { workspaceMeta } from "@/shared/config";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { BrandMark, Button, ScrollArea, Separator } from "@/shared/ui";
import type { WorkspaceModuleId, WorkspaceNavigationGroup } from "@/shared/workspace-module";

import { SidebarVersionInfo } from "./sidebar-version-info";

interface SidebarProps {
  groups: readonly WorkspaceNavigationGroup[];
  activeModuleId: WorkspaceModuleId;
  onSelectModule: (moduleId: WorkspaceModuleId) => void;
}

export function Sidebar({ groups, activeModuleId, onSelectModule }: SidebarProps) {
  const { t } = useI18n();

  return (
    <aside className="flex h-full w-80 flex-col border-r border-border bg-sidebar/60 backdrop-blur">
      <div className="flex flex-col gap-3 px-5 py-5">
        <div className="flex items-center gap-3">
          <BrandMark size="lg" />
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold text-sidebar-foreground">
              {t(workspaceMeta.nameKey)}
            </div>
            <div className="truncate text-xs text-muted-foreground">
              {t(workspaceMeta.subtitleKey)}
            </div>
          </div>
        </div>
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
          {t(workspaceMeta.trackKey)}
        </div>
      </div>

      <Separator />

      <ScrollArea className="flex-1">
        <nav aria-label={t("nav.label")} className="space-y-6 px-4 py-4">
          {groups.map((group) => (
            <div key={group.id} className="space-y-2">
              <div className="px-2 text-[11px] font-medium uppercase tracking-[0.2em] text-muted-foreground">
                {t(runtimeKey(group.labelKey))}
              </div>
              <div className="space-y-1">
                {group.entries.map((entry) => {
                  const Icon = entry.icon;
                  const isActive = activeModuleId === entry.id;

                  return (
                    <Button
                      key={entry.id}
                      aria-current={isActive ? "page" : undefined}
                      variant={isActive ? "secondary" : "ghost"}
                      className={cn(
                        "h-auto w-full justify-start rounded-md px-3 py-3 text-left",
                        isActive && "bg-accent text-accent-foreground",
                      )}
                      onClick={() => onSelectModule(entry.id)}
                    >
                      <Icon className="h-4 w-4 shrink-0" />
                      <span className="min-w-0 flex-1">
                        <span className="block truncate text-sm font-medium">
                          {t(runtimeKey(entry.labelKey))}
                        </span>
                        <span className="block truncate text-xs text-muted-foreground">
                          {t(runtimeKey(entry.descriptionKey))}
                        </span>
                      </span>
                    </Button>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>
      </ScrollArea>

      <div className="border-t border-border p-4">
        <SidebarVersionInfo />
      </div>
    </aside>
  );
}
