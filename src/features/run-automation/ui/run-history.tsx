import { timestampDate } from "@bufbuild/protobuf/wkt";
import { History, RotateCcw } from "lucide-react";
import { useMemo, useState } from "react";

import { RunStatusBadge } from "@/entities/flow-run";
import { destructiveActions } from "@/entities/task";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import { enumKey, type FlowRun, RunStatus, RunTrigger } from "@/shared/proto";
import { ConfirmButton, SectionCard } from "@/shared/ui";

import { useFlowRunActions, useFlowRuns } from "../model/use-flow-runs";

const SHOWN_RUNS = 20;

type Replayable = FlowRun & Required<Pick<FlowRun, "target">>;

function replayable(run: FlowRun): run is Replayable {
  return run.status !== RunStatus.RUNNING && run.target !== undefined;
}

function ReplayButton({
  run,
  disabled,
  onReplay,
}: {
  run: Replayable;
  disabled: boolean;
  onReplay: (run: Replayable) => void;
}) {
  const { t, locale } = useI18n();
  const risky = destructiveActions(run.steps.map((step) => step.kind));

  return (
    <ConfirmButton
      type="button"
      size="sm"
      variant="ghost"
      disabled={disabled}
      aria-label={t("runAutomation.history.replayNamed", { name: run.flowName })}
      ask={risky.length > 0}
      title={t("runAutomation.confirm.title", { name: run.flowName })}
      description={t("runAutomation.confirm.body", {
        steps: new Intl.ListFormat(locale).format(risky.map((action) => t(action.labelKey))),
      })}
      confirmLabel={t("runAutomation.history.replay")}
      cancelLabel={t("common.cancel")}
      onConfirm={() => onReplay(run)}
    >
      <RotateCcw className="h-4 w-4" />
      {t("runAutomation.history.replay")}
    </ConfirmButton>
  );
}

export function RunHistory() {
  const { t, locale } = useI18n();
  const runs = useFlowRuns();
  const { start, starting } = useFlowRunActions();
  const [problem, setProblem] = useState<string | null>(null);
  const timeFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "short" }),
    [locale],
  );

  const replay = (run: Replayable) => {
    setProblem(null);
    start(
      run.flowId,
      run.target,
      run.options ?? {
        repeat: 1,
        repeatIntervalSeconds: 0,
        retryAttempts: 0,
        retryDelaySeconds: 0,
      },
    )
      .then((result) => {
        if (!result.accepted) {
          setProblem(textValue(result.detail, true, t));
        }
      })
      .catch((error: unknown) => setProblem(errorMessage(error)));
  };

  return (
    <SectionCard
      icon={History}
      title={t("runAutomation.history.title")}
      contentClassName="space-y-2"
    >
      {problem ? (
        <p role="alert" className="break-words text-xs text-destructive">
          {problem}
        </p>
      ) : null}
      {runs.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("runAutomation.history.empty")}</p>
      ) : (
        <ol className="space-y-2">
          {runs.slice(0, SHOWN_RUNS).map((run) => (
            <li key={run.id} className="rounded-md border border-border bg-background p-3">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{run.flowName}</div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {t(`enums.runTrigger.${enumKey(RunTrigger, run.trigger)}`)}
                    {run.startedAt === undefined
                      ? ""
                      : ` · ${timeFormat.format(timestampDate(run.startedAt))}`}
                    {` · ${t("runAutomation.history.devices", { count: String(run.deviceIds.length) })}`}
                    {run.operator
                      ? ` · ${t("runAutomation.history.operator", { name: run.operator.name || run.operator.id })}`
                      : ""}
                  </p>
                  {run.detail ? (
                    <p className="mt-1 break-words text-xs leading-5 text-muted-foreground">
                      {textValue(run.detail, true, t)}
                    </p>
                  ) : null}
                </div>
                <div className="flex shrink-0 flex-col items-end gap-2">
                  <RunStatusBadge status={run.status} />
                  {replayable(run) ? (
                    <ReplayButton run={run} disabled={starting} onReplay={replay} />
                  ) : null}
                </div>
              </div>
            </li>
          ))}
        </ol>
      )}
    </SectionCard>
  );
}
