import { useState } from "react";

import { AssistantPrompt, parsePartialDraft } from "@/entities/assistant";
import type { ScriptBody } from "@/entities/script";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { ScriptLanguage } from "@/shared/proto";

import { useDraftScript } from "../model/use-draft-script";

type ScriptDraftAssistantProps = {
  language: ScriptLanguage;
  currentBody: string;
  onDraft: (script: ScriptBody) => void;
};

export function ScriptDraftAssistant({
  language,
  currentBody,
  onDraft,
}: ScriptDraftAssistantProps) {
  const { t } = useI18n();
  const { draft, drafting, error, partial, stop } = useDraftScript();
  const [note, setNote] = useState<{ ok: boolean; text: string } | null>(null);
  const writing = drafting
    ? (parsePartialDraft(partial) as { body?: unknown } | undefined)?.body
    : undefined;

  const submit = (prompt: string) => {
    setNote(null);
    void draft({ prompt, language, currentBody })
      .then((result) => {
        if (!result) {
          return;
        }
        if (result.accepted && result.draft?.script) {
          onDraft(result.draft.script);
          setNote({
            ok: true,
            text: [
              result.draft.explanation,
              result.draft.model ? t("assistant.byModel", { model: result.draft.model }) : "",
            ]
              .filter(Boolean)
              .join(" · "),
          });
        } else {
          setNote({ ok: false, text: textValue(result.detail, true, t) });
        }
      })
      .catch(() => undefined);
  };

  return (
    <div className="space-y-2 rounded-md border border-dashed border-border p-3">
      <AssistantPrompt
        label={currentBody.trim() ? t("scriptConsole.ai.change") : t("scriptConsole.ai.write")}
        placeholder={t("scriptConsole.ai.placeholder")}
        busy={drafting}
        onSubmit={submit}
        onStop={stop}
      />
      {typeof writing === "string" && writing ? (
        <pre
          role="log"
          aria-label={t("scriptConsole.ai.writing")}
          className="max-h-48 overflow-auto whitespace-pre-wrap break-words rounded-md bg-muted p-2 font-mono text-xs"
        >
          {writing}
        </pre>
      ) : null}
      {note || error ? (
        <p
          role={note?.ok ? "status" : "alert"}
          className={
            note?.ok
              ? "break-words text-xs leading-5 text-muted-foreground"
              : "break-words text-xs leading-5 text-destructive"
          }
        >
          {note ? note.text : errorMessage(error)}
        </p>
      ) : null}
    </div>
  );
}
