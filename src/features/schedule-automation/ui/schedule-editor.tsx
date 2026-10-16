import { useId, useState } from "react";

import { useAutomationFlows } from "@/entities/automation";
import { RunOptionsFields, RunTargetFields } from "@/entities/flow-run";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { DeviceGroup, UiText } from "@/shared/proto";
import { Button, Input, Label, Select } from "@/shared/ui";

import { cronPresets, type ScheduleDraft, toSchedule } from "../model/schedule-draft";
import { useScheduleActions } from "../model/use-schedules";

type ScheduleEditorProps = {
  initial: ScheduleDraft;
  selectedDeviceIds: readonly string[];
  groups: readonly DeviceGroup[];
  tags: readonly string[];
  onDone: () => void;
};

export function ScheduleEditor({
  initial,
  selectedDeviceIds,
  groups,
  tags,
  onDone,
}: ScheduleEditorProps) {
  const { t } = useI18n();
  const id = useId();
  const flows = useAutomationFlows();
  const { save, saving } = useScheduleActions();
  const [draft, setDraft] = useState(initial);
  const [refusal, setRefusal] = useState<UiText | string | undefined>();
  const flowKnown = flows.some((flow) => flow.id === draft.flowId);
  const schedule = flowKnown ? toSchedule(draft) : undefined;
  const patch = (next: Partial<ScheduleDraft>) => setDraft((current) => ({ ...current, ...next }));

  const submit = async () => {
    if (!schedule) {
      return;
    }

    try {
      const result = await save(schedule);

      if (result.accepted) {
        onDone();
      } else {
        setRefusal(result.detail);
      }
    } catch (error) {
      setRefusal(errorMessage(error));
    }
  };

  return (
    <form
      aria-label={t("scheduleAutomation.editor")}
      className="space-y-4 rounded-md border border-border bg-background p-3"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="space-y-1">
          <Label htmlFor={`${id}-name`}>{t("scheduleAutomation.name")}</Label>
          <Input
            id={`${id}-name`}
            value={draft.name}
            maxLength={32}
            onChange={(event) => patch({ name: event.currentTarget.value })}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-flow`}>{t("scheduleAutomation.flow")}</Label>
          <Select
            id={`${id}-flow`}
            value={draft.flowId}
            onChange={(event) => patch({ flowId: event.currentTarget.value })}
          >
            <option value="">{t("scheduleAutomation.pickFlow")}</option>
            {draft.flowId && !flowKnown ? (
              <option value={draft.flowId} disabled>
                {t("scheduleAutomation.flowMissing")}
              </option>
            ) : null}
            {flows.map((flow) => (
              <option key={flow.id} value={flow.id}>
                {flow.name}
              </option>
            ))}
          </Select>
        </div>
      </div>

      <RunTargetFields
        value={draft.target}
        onChange={(target) => patch({ target })}
        groups={groups}
        tags={tags}
        devicesHint={t("scheduleAutomation.devices", {
          count: String(draft.target.deviceIds.length),
        })}
      />
      {draft.target.mode === "devices" ? (
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={selectedDeviceIds.length === 0}
          onClick={() => patch({ target: { ...draft.target, deviceIds: selectedDeviceIds } })}
        >
          {t("scheduleAutomation.useSelection", { count: String(selectedDeviceIds.length) })}
        </Button>
      ) : null}

      <div className="space-y-1">
        <Label htmlFor={`${id}-cron`}>{t("scheduleAutomation.cron")}</Label>
        <div className="flex flex-wrap gap-2">
          <Input
            id={`${id}-cron`}
            className="min-w-0 flex-1 font-mono"
            value={draft.cron}
            onChange={(event) => patch({ cron: event.currentTarget.value })}
          />
          <Select
            aria-label={t("scheduleAutomation.presets.label")}
            className="w-auto"
            value=""
            onChange={(event) => {
              if (event.currentTarget.value) {
                patch({ cron: event.currentTarget.value });
              }
            }}
          >
            <option value="">{t("scheduleAutomation.presets.label")}</option>
            {cronPresets.map((preset) => (
              <option key={preset.id} value={preset.cron}>
                {t(`scheduleAutomation.presets.${preset.id}`)}
              </option>
            ))}
          </Select>
        </div>
        <p className="text-xs text-muted-foreground">{t("scheduleAutomation.cronHint")}</p>
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={draft.enabled}
          onChange={(event) => patch({ enabled: event.currentTarget.checked })}
        />
        {t("scheduleAutomation.enabled")}
      </label>

      <RunOptionsFields value={draft.options} onChange={(options) => patch({ options })} />

      {refusal ? (
        <p role="alert" className="break-words text-xs leading-5 text-destructive">
          {typeof refusal === "string" ? refusal : textValue(refusal, true, t)}
        </p>
      ) : null}
      <div className="flex gap-2">
        <Button type="submit" disabled={!schedule || saving}>
          {t("scheduleAutomation.save")}
        </Button>
        <Button type="button" variant="outline" onClick={onDone}>
          {t("scheduleAutomation.cancel")}
        </Button>
      </div>
    </form>
  );
}
