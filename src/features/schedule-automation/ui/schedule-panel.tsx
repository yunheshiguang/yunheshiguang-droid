import { timestampDate } from "@bufbuild/protobuf/wkt";
import { CalendarClock, Pencil, Plus, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";

import { useAutomationFlows } from "@/entities/automation";
import { RunStatusBadge } from "@/entities/flow-run";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { DeviceGroup, ScheduleResult } from "@/shared/proto";
import { Button, ConfirmButton, SectionCard } from "@/shared/ui";

import { draftFromSchedule, newScheduleDraft, type ScheduleDraft } from "../model/schedule-draft";
import { type ListedSchedule, useScheduleActions, useSchedules } from "../model/use-schedules";
import { ScheduleEditor } from "./schedule-editor";

type SchedulePanelProps = {
  selectedDeviceIds: readonly string[];
  groups: readonly DeviceGroup[];
  tags: readonly string[];
};

export function SchedulePanel({ selectedDeviceIds, groups, tags }: SchedulePanelProps) {
  const { t, locale } = useI18n();
  const schedules = useSchedules();
  const flows = useAutomationFlows();
  const { save, remove } = useScheduleActions();
  const [editing, setEditing] = useState<ScheduleDraft | undefined>();
  const [problem, setProblem] = useState<string | null>(null);
  const timeFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "short" }),
    [locale],
  );
  const flowName = (flowId: string) =>
    flows.find((flow) => flow.id === flowId)?.name ?? t("scheduleAutomation.missingFlow");
  const settle = (operation: Promise<ScheduleResult>) => {
    setProblem(null);
    operation.then(
      (result) => setProblem(result.accepted ? null : textValue(result.detail, true, t)),
      (error: unknown) => setProblem(errorMessage(error)),
    );
  };
  const toggle = (state: ListedSchedule) =>
    settle(save({ ...state.schedule, enabled: !state.schedule.enabled }));

  return (
    <SectionCard
      icon={CalendarClock}
      title={t("scheduleAutomation.title")}
      description={t("scheduleAutomation.description")}
      contentClassName="space-y-3"
    >
      {editing ? (
        <ScheduleEditor
          key={editing.id}
          initial={editing}
          selectedDeviceIds={selectedDeviceIds}
          groups={groups}
          tags={tags}
          onDone={() => setEditing(undefined)}
        />
      ) : (
        <Button
          type="button"
          variant="outline"
          disabled={flows.length === 0}
          onClick={() => setEditing(newScheduleDraft(flows[0]?.id ?? "", selectedDeviceIds))}
        >
          <Plus className="h-4 w-4" />
          {t("scheduleAutomation.add")}
        </Button>
      )}

      {problem ? (
        <p role="alert" className="break-words text-xs leading-5 text-destructive">
          {problem}
        </p>
      ) : null}

      {schedules.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("scheduleAutomation.empty")}</p>
      ) : (
        <ul className="space-y-2">
          {schedules.map((state) => (
            <li key={state.schedule.id} className="rounded-md border border-border p-3">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0 space-y-1">
                  <div className="truncate text-sm font-medium">{state.schedule.name}</div>
                  <p className="text-xs text-muted-foreground">
                    {flowName(state.schedule.flowId)} ·{" "}
                    <code className="font-mono">{state.schedule.cron}</code>
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {state.nextRunAt === undefined
                      ? t("scheduleAutomation.paused")
                      : t("scheduleAutomation.next", {
                          time: timeFormat.format(timestampDate(state.nextRunAt)),
                        })}
                  </p>
                  {state.lastRunAt === undefined ? null : (
                    <p className="flex items-center gap-2 text-xs text-muted-foreground">
                      {t("scheduleAutomation.last", {
                        time: timeFormat.format(timestampDate(state.lastRunAt)),
                      })}
                      <RunStatusBadge status={state.lastStatus} />
                    </p>
                  )}
                  {state.problem ? (
                    <p className="break-words text-xs leading-5 text-destructive">
                      {textValue(state.problem, true, t)}
                    </p>
                  ) : null}
                </div>
                <div className="flex shrink-0 gap-1">
                  <Button type="button" variant="ghost" size="sm" onClick={() => toggle(state)}>
                    {state.schedule.enabled
                      ? t("scheduleAutomation.pause")
                      : t("scheduleAutomation.resume")}
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={t("scheduleAutomation.edit", { name: state.schedule.name })}
                    onClick={() => setEditing(draftFromSchedule(state.schedule))}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <ConfirmButton
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={t("scheduleAutomation.remove", { name: state.schedule.name })}
                    title={t("scheduleAutomation.confirmRemove", { name: state.schedule.name })}
                    description={t("common.irreversible")}
                    confirmLabel={t("scheduleAutomation.delete")}
                    cancelLabel={t("common.cancel")}
                    onConfirm={() => settle(remove(state.schedule.id))}
                  >
                    <Trash2 className="h-4 w-4" />
                  </ConfirmButton>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </SectionCard>
  );
}
