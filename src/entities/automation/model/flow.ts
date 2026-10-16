import { compareIds, compareNames } from "@/shared/lib/text";
import type { BatchActionKind } from "@/shared/proto";

import { decodeFlowDocument } from "./flow-document";

export type AutomationStep = {
  id: string;
  kind: BatchActionKind;
  value: string;
};

export type AutomationFlow = {
  id: string;
  name: string;
  steps: readonly AutomationStep[];
};

export function createAutomationStep(kind: BatchActionKind, value = ""): AutomationStep {
  return { id: crypto.randomUUID(), kind, value };
}

export function createAutomationFlow(name = ""): AutomationFlow {
  return { id: crypto.randomUUID(), name, steps: [] };
}

export function moveAutomationStep(
  steps: readonly AutomationStep[],
  index: number,
  offset: -1 | 1,
): readonly AutomationStep[] {
  const target = index + offset;

  if (index < 0 || index >= steps.length || target < 0 || target >= steps.length) {
    return steps;
  }

  const next = [...steps];
  [next[index], next[target]] = [next[target], next[index]];

  return next;
}

export function readAutomationFlow(id: string, body: string): AutomationFlow | undefined {
  const flow = decodeFlowDocument(body);

  return flow ? { id, name: flow.name, steps: flow.steps } : undefined;
}

export function sortAutomationFlows(flows: readonly AutomationFlow[]): AutomationFlow[] {
  return [...flows].sort(
    (left, right) => compareNames(left.name, right.name) || compareIds(left.id, right.id),
  );
}
