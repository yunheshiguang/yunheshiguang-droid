import { toJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import { compareIds } from "@/shared/lib/text";
import {
  type DeviceGroupDefinitionJson,
  type DeviceGroupJson,
  type DeviceProfileJson,
  type DeviceRecordJson,
  type OrganizeResultJson,
  OrganizeResultSchema,
  type ProfileService,
  type RemoveDeviceGroupRequestJson,
  RemoveDeviceGroupRequestSchema,
  type SaveDeviceGroupRequestJson,
  SaveDeviceGroupRequestSchema,
  type UiTextJson,
  type UpdateDeviceProfilesRequestJson,
  UpdateDeviceProfilesRequestSchema,
} from "@/shared/proto";

import { fromMockJson } from "./mock-context";

const UNGROUPED_ID = "ungrouped";
const RESERVED_GROUP_IDS = new Set(["all", UNGROUPED_ID]);
const ONLINE = new Set(["DEVICE_STATUS_ONLINE", "DEVICE_STATUS_BUSY", "DEVICE_STATUS_IDLE"]);

function refused(key: string): OrganizeResultJson {
  return { accepted: false, detail: { key } };
}

function tidy(value: string): string {
  return value.trim().split(/\s+/).filter(Boolean).join(" ");
}

function sameTag(left: string, right: string): boolean {
  return left.toLowerCase() === right.toLowerCase();
}

export function createMockProfiles() {
  const profiles = new Map<string, DeviceProfileJson>();
  const groups = new Map<string, DeviceGroupDefinitionJson>();

  const sortedGroups = () =>
    [...groups.values()].sort(
      (left, right) =>
        compareIds(left.name ?? "", right.name ?? "") || compareIds(left.id ?? "", right.id ?? ""),
    );

  const annotate = (device: DeviceRecordJson): DeviceRecordJson => {
    const profile = profiles.get(device.id ?? "");
    const group = profile?.groupId ? groups.get(profile.groupId) : undefined;
    const labeled = Boolean(
      profile && (profile.number || profile.alias || profile.note || profile.tags?.length),
    );
    const note = profile?.note;

    return {
      ...device,
      groupId: group?.id ?? UNGROUPED_ID,
      group: group ? { fallback: group.name } : { key: "workspace.device.group.default" },
      labels: labeled
        ? {
            number: profile?.number,
            alias: profile?.alias,
            note: note
              ? {
                  displayValue: note,
                  canonicalValue: note.toLowerCase(),
                  redactedValue: "******",
                  piiClass: "PII_CLASS_NOTE",
                  visibility: "VISIBILITY_MODE_SEARCHABLE",
                  searchable: true,
                }
              : undefined,
            tags: profile?.tags,
          }
        : undefined,
    };
  };

  const summary = (devices: readonly DeviceRecordJson[]): DeviceGroupJson[] => {
    const rows: DeviceGroupJson[] = sortedGroups().map((group) => ({
      id: group.id,
      name: { fallback: group.name },
      total: 0,
      online: 0,
      tone: group.tone ?? "GROUP_TONE_EMERALD",
    }));
    const ungrouped: DeviceGroupJson = {
      id: UNGROUPED_ID,
      name: { key: "workspace.device.group.default" },
      total: 0,
      online: 0,
      tone: "GROUP_TONE_SLATE",
    };

    for (const device of devices) {
      const row = rows.find((group) => group.id === device.groupId) ?? ungrouped;

      row.total = (row.total ?? 0) + 1;
      row.online = (row.online ?? 0) + (ONLINE.has(device.status ?? "") ? 1 : 0);
    }

    return (ungrouped.total ?? 0) > 0 ? [...rows, ungrouped] : rows;
  };

  const update = ({ deviceIds = [], patch = {} }: UpdateDeviceProfilesRequestJson) => {
    if (deviceIds.length === 0) {
      return refused("workspace.organize.invalid.noDevices");
    }
    if (patch.groupId && !groups.has(patch.groupId)) {
      return refused("workspace.organize.invalid.groupMissing");
    }

    const addTags = (patch.addTags ?? []).map(tidy).filter(Boolean);
    const removeTags = (patch.removeTags ?? []).map(tidy).filter(Boolean);

    deviceIds.forEach((id, index) => {
      const profile: DeviceProfileJson = { deviceKey: id, tags: [], ...profiles.get(id) };

      if (patch.number !== undefined) {
        profile.number = patch.number === 0 ? 0 : patch.number + index;
      }
      if (patch.alias !== undefined) {
        profile.alias = tidy(patch.alias);
      }
      if (patch.note !== undefined) {
        profile.note = patch.note.trim();
      }
      if (patch.groupId !== undefined) {
        profile.groupId = patch.groupId;
      }

      const kept = (profile.tags ?? []).filter(
        (tag) => !removeTags.some((removed) => sameTag(removed, tag)),
      );
      profile.tags = [
        ...kept,
        ...addTags.filter((tag) => !kept.some((known) => sameTag(known, tag))),
      ];
      profiles.set(id, profile);
    });

    return { accepted: true, updated: deviceIds.length };
  };

  const saveGroup = ({ group }: SaveDeviceGroupRequestJson): OrganizeResultJson => {
    const id = group?.id?.trim() ?? "";
    const name = tidy(group?.name ?? "");

    if (!/^[A-Za-z0-9][A-Za-z0-9_-]*$/.test(id) || RESERVED_GROUP_IDS.has(id)) {
      return refused("workspace.organize.invalid.groupId");
    }
    if (!name) {
      return refused("workspace.organize.invalid.groupNameRequired");
    }

    groups.set(id, { id, name, tone: group?.tone });
    return { accepted: true, updated: 1 };
  };

  const removeGroup = ({ id = "" }: RemoveDeviceGroupRequestJson): OrganizeResultJson => {
    groups.delete(id);
    return { accepted: true, updated: 1 };
  };

  return { annotate, summary, update, saveGroup, removeGroup };
}

type Organized = (
  result: OrganizeResultJson,
  targets: readonly string[],
  detail: UiTextJson,
) => OrganizeResultJson;

export function createMockProfileService(
  profiles: ReturnType<typeof createMockProfiles>,
  organized: Organized,
  settle: <T>(value: T) => Promise<T>,
): ServiceImpl<typeof ProfileService> {
  const answer = async (result: OrganizeResultJson) =>
    fromMockJson(OrganizeResultSchema, await settle(result));

  return {
    updateDeviceProfiles: (request) => {
      const json = toJson(UpdateDeviceProfilesRequestSchema, request);
      const targets = json.deviceIds ?? [];

      return answer(
        organized(profiles.update(json), targets, {
          key: "workspace.organize.audit.profiles",
          params: { count: String(targets.length) },
        }),
      );
    },
    saveDeviceGroup: (request) =>
      answer(
        organized(profiles.saveGroup(toJson(SaveDeviceGroupRequestSchema, request)), [], {
          key: "workspace.organize.audit.groupSaved",
          params: { name: request.group?.name ?? "" },
        }),
      ),
    removeDeviceGroup: (request) =>
      answer(
        organized(profiles.removeGroup(toJson(RemoveDeviceGroupRequestSchema, request)), [], {
          key: "workspace.organize.audit.groupRemoved",
          params: { name: "" },
        }),
      ),
  };
}
