import {
  CheckCircle2,
  Circle,
  CircleSlash,
  LoaderCircle,
  type LucideIcon,
  XCircle,
} from "lucide-react";

import { useI18n } from "@/shared/i18n";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import { enumKey, type StepRun, StepStatus } from "@/shared/proto";

const STEP_STYLE = {
  [StepStatus.PENDING]: { icon: Circle, tone: "neutral" },
  [StepStatus.RUNNING]: { icon: LoaderCircle, tone: "info" },
  [StepStatus.DONE]: { icon: CheckCircle2, tone: "positive" },
  [StepStatus.FAILED]: { icon: XCircle, tone: "critical" },
  [StepStatus.SKIPPED]: { icon: CircleSlash, tone: "neutral" },
  [StepStatus.UNSPECIFIED]: { icon: Circle, tone: "neutral" },
} as const satisfies Record<StepStatus, { icon: LucideIcon; tone: StatusTone }>;

type FlowRunStepsProps = {
  steps: readonly StepRun[];
  actionLabel: (step: StepRun) => string;
};

export function FlowRunSteps({ steps, actionLabel }: FlowRunStepsProps) {
  const { t } = useI18n();

  return (
    <ol className="space-y-2">
      {steps.map((step, index) => {
        const { icon: Icon, tone } = STEP_STYLE[step.status] ?? STEP_STYLE[StepStatus.UNSPECIFIED];
        const results = step.result?.deviceResults ?? [];
        const succeeded = results.filter((result) => result.success).length;

        return (
          <li
            key={step.stepId || index}
            className="flex items-start gap-3 rounded-md border border-border p-3"
          >
            <Icon
              className={cn(
                "mt-0.5 h-4 w-4 shrink-0",
                statusTextTones[tone],
                step.status === StepStatus.RUNNING && "animate-spin",
              )}
            />
            <div className="min-w-0 flex-1 space-y-1">
              <div className="flex items-baseline justify-between gap-3">
                <span className="truncate text-sm font-medium">
                  {t("flowRun.stepLabel", { index: String(index + 1) })}
                  {" · "}
                  {actionLabel(step)}
                </span>
                <span className="shrink-0 text-xs text-muted-foreground">
                  {t(`enums.stepStatus.${enumKey(StepStatus, step.status)}`)}
                </span>
              </div>
              {results.length > 0 ? (
                <p className="text-xs text-muted-foreground">
                  {t("flowRun.summary", {
                    succeeded: String(succeeded),
                    count: String(results.length),
                  })}
                  {step.attempts > 1
                    ? ` · ${t("flowRun.attempts", { count: String(step.attempts) })}`
                    : ""}
                </p>
              ) : null}
              {step.status === StepStatus.FAILED && step.result?.task?.detail ? (
                <p className="break-words text-xs leading-5 text-destructive">
                  {textValue(step.result.task.detail, true, t)}
                </p>
              ) : null}
            </div>
          </li>
        );
      })}
    </ol>
  );
}
