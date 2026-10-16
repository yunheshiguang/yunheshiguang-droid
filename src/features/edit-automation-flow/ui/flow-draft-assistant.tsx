import { useId, useState } from "react";

import { AssistantPrompt, parsePartialDraft } from "@/entities/assistant";
import type { AutomationFlow } from "@/entities/automation";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { UiText } from "@/shared/proto";

import { type ReceivedFlowDraft, useDraftFlow } from "../model/use-draft-flow";

type FlowDraftAssistantProps = {
  onDraft: (flow: AutomationFlow) => void;
};

export function FlowDraftAssistant({ onDraft }: FlowDraftAssistantProps) {
  const { t } = useI18n();
  const titleId = useId();
  const { draft, drafting, error, partial, stop } = useDraftFlow();
  const [received, setReceived] = useState<ReceivedFlowDraft | null>(null);
  const [refused, setRefused] = useState<UiText | undefined>();
  const written = drafting
    ? (parsePartialDraft(partial) as { steps?: unknown } | undefined)?.steps
    : undefined;

  const submit = (prompt: string) => {
    setRefused(undefined);
    setReceived(null);
    void draft(prompt)
      .then((outcome) => {
        setRefused(outcome.refused);
        setReceived(outcome.received ?? null);
        if (outcome.received) {
          onDraft(outcome.received.flow);
        }
      })
      .catch(() => undefined);
  };

  return (
    <section
      aria-labelledby={titleId}
      className="space-y-3 rounded-md border border-dashed border-border p-3"
    >
      <h3 id={titleId} className="text-sm font-medium">
        {t("editAutomationFlow.draft.title")}
      </h3>
      <AssistantPrompt
        label={t("editAutomationFlow.draft.prompt")}
        placeholder={t("editAutomationFlow.draft.placeholder")}
        busy={drafting}
        onSubmit={submit}
        onStop={stop}
      />
      {Array.isArray(written) && written.length > 0 ? (
        <p role="status" className="text-xs text-muted-foreground">
          {t("editAutomationFlow.draft.writing", { count: String(written.length) })}
        </p>
      ) : null}
      {refused || error ? (
        <p role="alert" className="break-words text-xs leading-5 text-destructive">
          {refused ? textValue(refused, true, t) : errorMessage(error)}
        </p>
      ) : null}
      {received ? (
        <div role="status" className="space-y-1 text-xs text-muted-foreground">
          <p>
            {t("editAutomationFlow.draft.received", { count: String(received.flow.steps.length) })}
            {received.model ? ` · ${t("assistant.byModel", { model: received.model })}` : null}
          </p>
          {received.issues.length > 0 ? (
            <ul
              aria-label={t("editAutomationFlow.draft.issues")}
              className="list-disc space-y-1 pl-4"
            >
              {received.issues.map((issue) => (
                <li key={JSON.stringify(issue)}>
                  {issue.step > 0
                    ? `${t("editAutomationFlow.draft.step", { step: String(issue.step) })} `
                    : null}
                  {textValue(issue.message, true, t)}
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
