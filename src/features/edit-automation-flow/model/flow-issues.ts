import type { AutomationFlow, AutomationStep } from "@/entities/automation";
import { resolveBatchAction } from "@/entities/task";

export type FlowIssues = {
  name: boolean;
  empty: boolean;
  stepsMissingValue: ReadonlySet<string>;
  stepsUnknown: ReadonlySet<string>;
};

export function stepTakesValue(step: AutomationStep): boolean {
  return (resolveBatchAction(step.kind)?.input ?? "none") !== "none";
}

export function flowIssues(flow: AutomationFlow): FlowIssues {
  return {
    name: flow.name.trim() === "",
    empty: flow.steps.length === 0,
    stepsMissingValue: new Set(
      flow.steps
        .filter((step) => stepTakesValue(step) && step.value.trim() === "")
        .map((step) => step.id),
    ),
    stepsUnknown: new Set(
      flow.steps.filter((step) => !resolveBatchAction(step.kind)).map((step) => step.id),
    ),
  };
}

export function hasFlowIssues(issues: FlowIssues): boolean {
  return (
    issues.name || issues.empty || issues.stepsMissingValue.size > 0 || issues.stepsUnknown.size > 0
  );
}
