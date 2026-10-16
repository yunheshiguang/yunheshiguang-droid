import type { MessageInitShape } from "@bufbuild/protobuf";

import {
  defaultRunOptions,
  draftFromTarget,
  emptyRunTarget,
  type RunOptionValues,
  type RunTargetDraft,
  resolveRunTarget,
  runOptionValues,
} from "@/entities/flow-run";
import type { Schedule, ScheduleSchema } from "@/shared/proto";

export type ScheduleDraft = {
  id: string;
  name: string;
  flowId: string;
  target: RunTargetDraft;
  cron: string;
  enabled: boolean;
  options: RunOptionValues;
};

export const cronPresets = [
  { id: "everyFifteenMinutes", cron: "*/15 * * * *" },
  { id: "hourly", cron: "0 * * * *" },
  { id: "dailyNine", cron: "0 9 * * *" },
  { id: "weekdaysNine", cron: "0 9 * * 1-5" },
  { id: "nightly", cron: "0 2 * * *" },
] as const;

export function newScheduleDraft(flowId: string, deviceIds: readonly string[]): ScheduleDraft {
  return {
    id: crypto.randomUUID(),
    name: "",
    flowId,
    target: { ...emptyRunTarget, deviceIds },
    cron: cronPresets[2].cron,
    enabled: true,
    options: defaultRunOptions,
  };
}

export function draftFromSchedule(schedule: Schedule): ScheduleDraft {
  return {
    id: schedule.id,
    name: schedule.name,
    flowId: schedule.flowId,
    target: draftFromTarget(schedule.target),
    cron: schedule.cron,
    enabled: schedule.enabled,
    options: runOptionValues(schedule.options),
  };
}

export function toSchedule(
  draft: ScheduleDraft,
): MessageInitShape<typeof ScheduleSchema> | undefined {
  const target = resolveRunTarget(draft.target);
  const name = draft.name.trim();
  const cron = draft.cron.trim();

  if (!name || !draft.flowId || !target || !cron) {
    return undefined;
  }

  return {
    id: draft.id,
    name,
    flowId: draft.flowId,
    target,
    cron,
    enabled: draft.enabled,
    options: draft.options,
  };
}
