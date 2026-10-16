import { UNGROUPED_GROUP_ID } from "@/shared/lib/workspace-view";
import { type DeviceGroup, enumKeys, GroupTone } from "@/shared/proto";

export type { DeviceGroup };
export { UNGROUPED_GROUP_ID };

export const groupTones = enumKeys(GroupTone);

export function operatorGroups(groups: readonly DeviceGroup[]): DeviceGroup[] {
  return groups.filter((group) => group.id !== UNGROUPED_GROUP_ID);
}
