import {
  type AutomationFlow,
  createAutomationFlow,
  createAutomationStep,
} from "@/entities/automation";
import type { FlowTemplate } from "@/shared/proto";

export function flowFromTemplate(template: FlowTemplate, name: string): AutomationFlow {
  return {
    ...createAutomationFlow(name),
    steps: template.steps.map((step) => createAutomationStep(step.kind, step.value)),
  };
}
