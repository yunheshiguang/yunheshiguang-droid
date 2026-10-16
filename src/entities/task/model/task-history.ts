import { createStore, useStore } from "zustand";
import { useShallow } from "zustand/shallow";

import type {
  AdbDeviceActionResult,
  BatchActionRequest,
  BatchActionResult,
  DeviceHint,
  DeviceInputRoute,
  DeviceScriptOutput,
  DiagnosticSignal,
} from "@/shared/proto";

import type { TaskItem } from "./types";

export const TASK_HISTORY_LIMIT = 200;

export type TaskRun = {
  task: TaskItem;
  deviceResults: readonly AdbDeviceActionResult[];
  diagnostics: readonly DiagnosticSignal[];
  deviceHints: readonly DeviceHint[];
  inputRoutes: readonly DeviceInputRoute[];
  scriptOutputs: readonly DeviceScriptOutput[];
  request?: BatchActionRequest;
};

export function toTaskRun(
  result: BatchActionResult,
  request?: BatchActionRequest,
): TaskRun | undefined {
  return result.task
    ? {
        task: result.task,
        deviceResults: result.deviceResults,
        diagnostics: result.diagnostics,
        deviceHints: result.deviceHints,
        inputRoutes: result.inputRoutes,
        scriptOutputs: result.scriptOutputs,
        request,
      }
    : undefined;
}

export function recordRun(
  runs: readonly TaskRun[],
  run: TaskRun,
  limit = TASK_HISTORY_LIMIT,
): readonly TaskRun[] {
  const index = runs.findIndex((item) => item.task.id === run.task.id);

  if (index >= 0) {
    return runs.map((item, position) => (position === index ? run : item));
  }

  return [run, ...runs].slice(0, limit);
}

export function mergeTasks(
  local: readonly TaskItem[],
  remote: readonly TaskItem[],
): readonly TaskItem[] {
  if (remote.length === 0) {
    return local;
  }

  const remoteById = new Map(remote.map((task) => [task.id, task]));
  const localIds = new Set(local.map((task) => task.id));

  return [
    ...local.map((task) => remoteById.get(task.id) ?? task),
    ...remote.filter((task) => !localIds.has(task.id)),
  ];
}

export type TaskHistoryActions = {
  record: (run: TaskRun) => void;
  clear: () => void;
};

type TaskHistoryState = {
  runs: readonly TaskRun[];
  actions: TaskHistoryActions;
};

export function createTaskHistoryStore(limit = TASK_HISTORY_LIMIT) {
  return createStore<TaskHistoryState>()((set) => ({
    runs: [],
    actions: {
      record: (run) => set((state) => ({ runs: recordRun(state.runs, run, limit) })),
      clear: () => set({ runs: [] }),
    },
  }));
}

const taskHistoryStore = createTaskHistoryStore();

export function useTaskHistory(): readonly TaskItem[] {
  return useStore(
    taskHistoryStore,
    useShallow((state) => state.runs.map((run) => run.task)),
  );
}

export function useLatestTaskRun(): TaskRun | undefined {
  return useStore(taskHistoryStore, (state) => state.runs[0]);
}

export function useTaskHistoryActions(): TaskHistoryActions {
  return useStore(taskHistoryStore, (state) => state.actions);
}
