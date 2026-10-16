import type { MessageInitShape } from "@bufbuild/protobuf";
import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { workspaceMutationKeys } from "@/shared/api";
import {
  AutomationService,
  type Schedule,
  type ScheduleResult,
  type ScheduleSchema,
  type ScheduleState,
  type Schedules,
} from "@/shared/proto";

export type ListedSchedule = ScheduleState & { schedule: Schedule };

const NO_SCHEDULES: readonly ListedSchedule[] = [];

const listSchedules = AutomationService.method.listSchedules;

const selectListed = (data: Schedules) =>
  data.schedules.filter((state): state is ListedSchedule => state.schedule !== undefined);

export function useSchedules(): readonly ListedSchedule[] {
  return useQuery(listSchedules, {}, { select: selectListed }).data ?? NO_SCHEDULES;
}

export type ScheduleActions = {
  save: (schedule: MessageInitShape<typeof ScheduleSchema>) => Promise<ScheduleResult>;
  remove: (id: string) => Promise<ScheduleResult>;
  saving: boolean;
};

export function useScheduleActions(): ScheduleActions {
  const queryClient = useQueryClient();
  const options = {
    mutationKey: workspaceMutationKeys.schedules(),
    onSettled: () =>
      queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: listSchedules, cardinality: "finite" }),
      }),
  };
  const save = useMutation(AutomationService.method.saveSchedule, options);
  const remove = useMutation(AutomationService.method.removeSchedule, options);

  return {
    save: (schedule) => save.mutateAsync({ schedule }),
    remove: (id) => remove.mutateAsync({ id }),
    saving: save.isPending,
  };
}
