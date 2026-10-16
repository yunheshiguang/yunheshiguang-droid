import type { DeviceGroup } from "@/entities/group";
import { GroupManager } from "@/features/organize-devices";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { groupToneClass, textValue } from "@/shared/lib/workspace-view";
import { enumKey, GroupTone } from "@/shared/proto";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/shared/ui";

export type DeviceGroupListProps = {
  groups: readonly DeviceGroup[];
  onSelectGroup: (groupId: string) => void;
};

export function DeviceGroupList({ groups, onSelectGroup }: DeviceGroupListProps) {
  const { t } = useI18n();

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>{t("deviceWall.groups")}</CardTitle>
        <CardDescription>{t("deviceWall.groupsDescription")}</CardDescription>
        <div>
          <GroupManager groups={groups} />
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {groups.map((group) => (
          <button
            key={group.id}
            type="button"
            className="flex w-full items-center justify-between gap-2 rounded-md border border-border bg-background px-3 py-2 text-left text-sm"
            onClick={() => onSelectGroup(group.id)}
          >
            <span className="flex min-w-0 items-center gap-2">
              <span
                aria-hidden
                className={cn(
                  "h-2 w-2 shrink-0 rounded-full",
                  groupToneClass(enumKey(GroupTone, group.tone)),
                )}
              />
              <span className="truncate">{textValue(group.name, false, t)}</span>
            </span>
            <span className="text-xs text-muted-foreground">
              {group.online}/{group.total}
            </span>
          </button>
        ))}
      </CardContent>
    </Card>
  );
}
