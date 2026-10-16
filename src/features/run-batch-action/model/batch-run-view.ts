import type { TaskRun } from "@/entities/task";

export type BatchAttempt = {
  status: "idle" | "pending" | "success" | "error";
  error: unknown;
};

export type BatchRunView =
  | { status: "idle" }
  | { status: "running" }
  | { status: "unreachable"; error: unknown }
  | { status: "finished"; run: TaskRun };

export function resolveBatchRunView(
  attempt: BatchAttempt | undefined,
  latestRun: TaskRun | undefined,
): BatchRunView {
  if (attempt?.status === "pending") {
    return { status: "running" };
  }

  if (attempt?.status === "error") {
    return { status: "unreachable", error: attempt.error };
  }

  return latestRun ? { status: "finished", run: latestRun } : { status: "idle" };
}
