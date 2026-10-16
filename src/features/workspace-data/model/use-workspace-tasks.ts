import { useMemo } from "react";

import { mergeTasks, type TaskItem, useTaskHistory } from "@/entities/task";
import type { WorkspaceSnapshot } from "@/shared/proto";

import { useWorkspaceSnapshot } from "./use-workspace-snapshot";

const NO_TASKS: readonly TaskItem[] = [];
const selectTasks = (snapshot: WorkspaceSnapshot) => snapshot.tasks;

export function useWorkspaceTasks(): readonly TaskItem[] {
  const local = useTaskHistory();
  const remote = useWorkspaceSnapshot(selectTasks) ?? NO_TASKS;

  return useMemo(() => mergeTasks(local, remote), [local, remote]);
}
