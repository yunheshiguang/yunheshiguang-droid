import { Sparkles, Square } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { Button, Label, Textarea } from "@/shared/ui";

import { useAssistantReadiness } from "../model/use-assistant";

type AssistantPromptProps = {
  label: string;
  placeholder: string;
  busy: boolean;
  onSubmit: (prompt: string) => void;
  onStop?: () => void;
};

export function AssistantPrompt({
  label,
  placeholder,
  busy,
  onSubmit,
  onStop,
}: AssistantPromptProps) {
  const { t } = useI18n();
  const id = useId();
  const { ready, problem } = useAssistantReadiness();
  const [prompt, setPrompt] = useState("");

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (prompt.trim()) {
      onSubmit(prompt.trim());
    }
  };

  return (
    <form onSubmit={submit} className="space-y-2">
      <Label htmlFor={id}>{label}</Label>
      <Textarea
        id={id}
        value={prompt}
        rows={3}
        placeholder={placeholder}
        onChange={(event) => setPrompt(event.currentTarget.value)}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Button type="submit" size="sm" disabled={!ready || busy || !prompt.trim()}>
          <Sparkles className="h-4 w-4" />
          {busy ? t("assistant.drafting") : t("assistant.draft")}
        </Button>
        {busy && onStop ? (
          <Button type="button" size="sm" variant="outline" onClick={onStop}>
            <Square className="h-4 w-4" />
            {t("assistant.stop")}
          </Button>
        ) : null}
        {ready ? (
          <span className="text-xs text-muted-foreground">{t("assistant.review")}</span>
        ) : (
          <span role="status" className="text-xs text-muted-foreground">
            {problem ? textValue(problem, true, t) : t("assistant.notReady")}
          </span>
        )}
      </div>
    </form>
  );
}
