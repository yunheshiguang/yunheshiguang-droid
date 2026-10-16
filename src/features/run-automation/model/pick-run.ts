import { type FlowRun, RunStatus } from "@/shared/proto";

export function pickCurrentRun(runs: readonly FlowRun[], flowId: string): FlowRun | undefined {
  const ofFlow = runs.filter((run) => run.flowId === flowId);

  return ofFlow.find((run) => run.status === RunStatus.RUNNING) ?? ofFlow[0];
}
