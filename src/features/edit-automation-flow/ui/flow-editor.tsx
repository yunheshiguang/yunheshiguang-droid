import { ListChecks, Plus } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import {
  type AutomationFlow,
  createAutomationFlow,
  createAutomationStep,
  moveAutomationStep,
  useAutomationFlowActions,
  useAutomationFlows,
  useAutomationFlowsError,
} from "@/entities/automation";
import { listBatchActions } from "@/entities/task";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { Button, ConfirmButton, ErrorState, Input, Label, SectionCard } from "@/shared/ui";

import { flowIssues, hasFlowIssues } from "../model/flow-issues";
import { FlowDraftAssistant } from "./flow-draft-assistant";
import { FlowStepRow } from "./flow-step-row";

type FlowDraftEditorProps = {
  initial: AutomationFlow;
  onClose: () => void;
};

function FlowDraftEditor({ initial, onClose }: FlowDraftEditorProps) {
  const { t } = useI18n();
  const id = useId();
  const flows = useAutomationFlows();
  const { save, remove } = useAutomationFlowActions();
  const [draft, setDraft] = useState(initial);
  const [attempted, setAttempted] = useState(false);
  const [saved, setSaved] = useState(false);
  const [writeError, setWriteError] = useState<string | null>(null);
  const issues = flowIssues(draft);
  const stored = flows.find((flow) => flow.id === draft.id);

  const update = (next: AutomationFlow) => {
    setDraft(next);
    setSaved(false);
  };

  const write = (operation: Promise<void>, onDone: () => void) => {
    setWriteError(null);
    operation.then(onDone, (error: unknown) => setWriteError(errorMessage(error)));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setAttempted(true);

    if (!hasFlowIssues(issues)) {
      write(save({ ...draft, name: draft.name.trim() }), () => setSaved(true));
    }
  };

  return (
    <form onSubmit={submit} aria-label={t("editAutomationFlow.editor")} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor={`${id}-name`}>{t("editAutomationFlow.name")}</Label>
        <Input
          id={`${id}-name`}
          value={draft.name}
          onChange={(event) => update({ ...draft, name: event.currentTarget.value })}
          placeholder={t("editAutomationFlow.namePlaceholder")}
          aria-invalid={(attempted && issues.name) || undefined}
        />
        {attempted && issues.name ? (
          <p className="text-xs text-destructive">{t("editAutomationFlow.issues.name")}</p>
        ) : null}
      </div>

      <ol aria-label={t("editAutomationFlow.steps")} className="space-y-3">
        {draft.steps.map((step, index) => (
          <FlowStepRow
            key={step.id}
            step={step}
            index={index}
            count={draft.steps.length}
            missingValue={attempted && issues.stepsMissingValue.has(step.id)}
            onChange={(next) =>
              update({
                ...draft,
                steps: draft.steps.map((item) => (item.id === step.id ? next : item)),
              })
            }
            onMove={(offset) =>
              update({ ...draft, steps: moveAutomationStep(draft.steps, index, offset) })
            }
            onRemove={() =>
              update({ ...draft, steps: draft.steps.filter((item) => item.id !== step.id) })
            }
          />
        ))}
      </ol>
      {attempted && issues.empty ? (
        <p className="text-xs text-destructive">{t("editAutomationFlow.issues.empty")}</p>
      ) : null}

      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() =>
          update({
            ...draft,
            steps: [
              ...draft.steps,
              createAutomationStep(listBatchActions()[0]?.kind ?? "SCREENSHOT"),
            ],
          })
        }
      >
        <Plus className="h-4 w-4" />
        {t("editAutomationFlow.addStep")}
      </Button>

      <div className="flex flex-wrap items-center gap-2">
        <Button type="submit">{t("editAutomationFlow.save")}</Button>
        {stored ? (
          <ConfirmButton
            type="button"
            variant="outline"
            title={t("editAutomationFlow.confirmDelete", { name: stored.name })}
            description={t("common.irreversible")}
            confirmLabel={t("editAutomationFlow.delete")}
            cancelLabel={t("common.cancel")}
            onConfirm={() => write(remove(draft.id), onClose)}
          >
            {t("editAutomationFlow.delete")}
          </ConfirmButton>
        ) : null}
        <Button type="button" variant="ghost" onClick={onClose}>
          {t("editAutomationFlow.close")}
        </Button>
        {saved ? (
          <span role="status" className="text-xs text-muted-foreground">
            {t("editAutomationFlow.saved")}
          </span>
        ) : null}
      </div>
      {writeError ? (
        <ErrorState title={t("editAutomationFlow.writeFailed")} detail={writeError} />
      ) : null}
    </form>
  );
}

export function FlowEditor() {
  const { t } = useI18n();
  const flows = useAutomationFlows();
  const loadError = useAutomationFlowsError();
  const [draft, setDraft] = useState<AutomationFlow | null>(null);
  const adoptInto = (targetId: string | undefined) => (flow: AutomationFlow) =>
    setDraft((current) => (current?.id === targetId ? flow : current));

  return (
    <SectionCard
      icon={ListChecks}
      title={t("editAutomationFlow.title")}
      contentClassName="space-y-4"
    >
      {loadError ? (
        <ErrorState title={t("common.documentsError")} detail={errorMessage(loadError)} />
      ) : null}
      <div className="flex flex-wrap gap-2">
        {flows.map((flow) => (
          <Button
            key={flow.id}
            type="button"
            size="sm"
            variant={flow.id === draft?.id ? "default" : "outline"}
            aria-pressed={flow.id === draft?.id}
            onClick={() => setDraft(flow)}
          >
            {flow.name}
          </Button>
        ))}
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => setDraft(createAutomationFlow())}
        >
          <Plus className="h-4 w-4" />
          {t("editAutomationFlow.create")}
        </Button>
      </div>

      <FlowDraftAssistant onDraft={adoptInto(draft?.id)} />

      {draft ? (
        <FlowDraftEditor key={draft.id} initial={draft} onClose={() => setDraft(null)} />
      ) : (
        <p className="text-sm text-muted-foreground">
          {flows.length === 0 ? t("editAutomationFlow.empty") : t("editAutomationFlow.pick")}
        </p>
      )}
    </SectionCard>
  );
}
