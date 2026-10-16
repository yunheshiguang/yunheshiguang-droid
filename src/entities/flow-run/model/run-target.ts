import type { MessageInitShape } from "@bufbuild/protobuf";

import type { RunTarget, RunTargetSchema } from "@/shared/proto";

export type RunTargetMode = "devices" | "group" | "tag";

export type RunTargetInit = MessageInitShape<typeof RunTargetSchema>;

export type RunTargetDraft = {
  mode: RunTargetMode;
  deviceIds: readonly string[];
  groupId: string;
  tag: string;
};

export const emptyRunTarget: RunTargetDraft = {
  mode: "devices",
  deviceIds: [],
  groupId: "",
  tag: "",
};

export function resolveRunTarget(draft: RunTargetDraft): RunTargetInit | undefined {
  switch (draft.mode) {
    case "devices":
      return draft.deviceIds.length > 0
        ? { target: { case: "devices", value: { deviceIds: [...draft.deviceIds] } } }
        : undefined;
    case "group":
      return draft.groupId ? { target: { case: "groupId", value: draft.groupId } } : undefined;
    case "tag":
      return draft.tag.trim() ? { target: { case: "tag", value: draft.tag.trim() } } : undefined;
  }
}

export function draftFromTarget(target: RunTarget | undefined): RunTargetDraft {
  switch (target?.target.case) {
    case "devices":
      return { ...emptyRunTarget, mode: "devices", deviceIds: target.target.value.deviceIds };
    case "groupId":
      return { ...emptyRunTarget, mode: "group", groupId: target.target.value };
    case "tag":
      return { ...emptyRunTarget, mode: "tag", tag: target.target.value };
    default:
      return emptyRunTarget;
  }
}
