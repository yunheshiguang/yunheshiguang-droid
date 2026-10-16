import { Play, Square } from "lucide-react";
import { useId, useState } from "react";

import { useAutomationFlows } from "@/entities/automation";
import {
  defaultRunOptions,
  emptyRunTarget,
  FlowRunSteps,
  RunOptionsFields,
  type RunOptionValues,
  RunStatusBadge,
  type RunTargetDraft,
  RunTargetFields,
  resolveRunTarget,
} from "@/entities/flow-run";
import { destructiveActions, resolveBatchAction } from "@/entities/task";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import {
  BatchActionKind,
  type DeviceGroup,
  enumKey,
  type FlowRunResult,
  RunStatus,
  type UiText,
} from "@/shared/proto";
import { Button, ConfirmButton, Label, SectionCard, Select } from "@/shared/ui";

import { pickCurrentRun } from "../model/pick-run";
import { useFlowRunActions, useFlowRuns } from "../model/use-flow-runs";

type AutomationRunPanelProps = {
  selectedDeviceIds: readonly string[];
  groups: readonly DeviceGroup[];
  tags: readonly string[];
};

export function AutomationRunPanel({ selectedDeviceIds, groups, tags }: AutomationRunPanelProps) {
  const { t, locale } = useI18n();
  const id = useId();
  const flows = useAutomationFlows();
  const runs = useFlowRuns();
  const { start, stop, starting } = useFlowRunActions();
  const [flowId, setFlowId] = useState("");
  const [target, setTarget] = useState<RunTargetDraft>(emptyRunTarget);
  const [options, setOptions] = useState<RunOptionValues>(defaultRunOptions);
  const [refusal, setRefusal] = useState<UiText | string | undefined>();
  const flow = flows.find((candidate) => candidate.id === flowId) ?? flows[0];
  const resolved = resolveRunTarget({ ...target, deviceIds: selectedDeviceIds });
  const current = flow ? pickCurrentRun(runs, flow.id) : undefined;
  const running = current?.status === RunStatus.RUNNING;
  const canRun = Boolean(flow?.steps.length) && resolved !== undefined && !starting && !running;
  const risky = flow ? destructiveActions(flow.steps.map((step) => step.kind)) : [];

  const settle = async (operation: Promise<FlowRunResult>) => {
    try {
      const result = await operation;
      setRefusal(result.accepted ? undefined : result.detail);
    } catch (error) {
      setRefusal(errorMessage(error));
    }
  };

  const run = () => {
    if (flow && resolved) {
      void settle(start(flow.id, resolved, options));
    }
  };

  return (
    <SectionCard
      icon={Play}
      title={t("runAutomation.title")}
      description={t("runAutomation.description")}
      contentClassName="space-y-4"
    >
      {flow ? (
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor={`${id}-flow`}>{t("runAutomation.flow")}</Label>
            <Select
              id={`${id}-flow`}
              value={flow.id}
              onChange={(event) => setFlowId(event.currentTarget.value)}
            >
              {flows.map((candidate) => (
                <option key={candidate.id} value={candidate.id}>
                  {candidate.name}
                </option>
              ))}
            </Select>
          </div>
          <RunTargetFields
            value={target}
            onChange={setTarget}
            groups={groups}
            tags={tags}
            devicesHint={
              selectedDeviceIds.length === 0
                ? t("runAutomation.noDevices")
                : t("runAutomation.selected", { count: String(selectedDeviceIds.length) })
            }
          />
          <RunOptionsFields value={options} onChange={setOptions} />
          <div className="flex flex-wrap gap-2">
            <ConfirmButton
              type="button"
              disabled={!canRun}
              ask={risky.length > 0}
              title={t("runAutomation.confirm.title", { name: flow.name })}
              description={t("runAutomation.confirm.body", {
                steps: new Intl.ListFormat(locale).format(
                  risky.map((action) => t(action.labelKey)),
                ),
              })}
              confirmLabel={t("runAutomation.run")}
              cancelLabel={t("common.cancel")}
              onConfirm={run}
            >
              <Play className="h-4 w-4" />
              {t("runAutomation.run")}
            </ConfirmButton>
            {running && current ? (
              <Button type="button" variant="outline" onClick={() => void settle(stop(current.id))}>
                <Square className="h-4 w-4" />
                {t("runAutomation.stop")}
              </Button>
            ) : null}
          </div>
          {refusal ? (
            <p role="alert" className="break-words text-xs leading-5 text-destructive">
              {typeof refusal === "string" ? refusal : textValue(refusal, true, t)}
            </p>
          ) : null}
        </div>
      ) : (
        <p className="text-sm text-muted-foreground">{t("runAutomation.noFlows")}</p>
      )}

      {current ? (
        <section aria-label={t("runAutomation.progress")} className="space-y-2">
          <div className="flex items-baseline justify-between gap-3">
            <p role="status" className="min-w-0 truncate text-sm font-medium">
              {t("runAutomation.iteration", {
                name: current.flowName,
                iteration: String(current.iteration),
                iterations: String(current.iterations),
              })}
            </p>
            <RunStatusBadge status={current.status} />
          </div>
          {current.detail ? (
            <p className="break-words text-xs leading-5 text-muted-foreground">
              {textValue(current.detail, true, t)}
            </p>
          ) : null}
          <FlowRunSteps
            steps={current.steps}
            actionLabel={(step) => {
              const action = resolveBatchAction(step.kind);
              return action ? t(action.labelKey) : enumKey(BatchActionKind, step.kind);
            }}
          />
        </section>
      ) : null}
    </SectionCard>
  );
}
