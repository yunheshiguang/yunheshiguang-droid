import { create, toJsonString } from "@bufbuild/protobuf";
import { Bot } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import {
  type AssistantSettings,
  AssistantSettingsSchema,
  type AssistantState,
  enumKey,
  StructuredOutput,
} from "@/shared/proto";
import { Badge, Button, ConfirmButton, Input, Label, SectionCard, Select } from "@/shared/ui";

import { assistantPresets, useAssistantSettings } from "../model/use-assistant-settings";

const OUTPUT_MODES = ["JSON_SCHEMA", "JSON_OBJECT"] as const;

type Feedback = { ok: boolean; text: string };

type FormProps = {
  state: AssistantState;
  saved: AssistantSettings;
  feedback: Feedback | null;
  onFeedback: (feedback: Feedback) => void;
};

function SettingsForm({ state, saved, feedback, onFeedback }: FormProps) {
  const { t } = useI18n();
  const id = useId();
  const { save, saving, error } = useAssistantSettings();
  const [settings, setSettings] = useState<AssistantSettings>(saved);
  const [apiKey, setApiKey] = useState("");

  const submit = async (next: AssistantSettings, key?: string) => {
    const result = await save({ settings: next, apiKey: key });
    setApiKey("");
    onFeedback({
      ok: result.accepted,
      text: result.accepted
        ? t("assistantSettings.saved")
        : textValue(result.detail, true, t) || t("assistantSettings.failed"),
    });
  };
  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    void submit(settings, apiKey.trim() ? apiKey.trim() : undefined).catch(() => undefined);
  };

  return (
    <form onSubmit={onSubmit} aria-label={t("assistantSettings.title")} className="space-y-4">
      <ul className="space-y-2">
        {state.providers.map((provider) => (
          <li key={provider.id} className="rounded-md border border-border p-3">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0 space-y-1">
                <p className="text-sm font-medium">
                  {textValue(provider.label, true, t) || provider.id}
                </p>
                <p className="text-xs leading-5 text-muted-foreground">
                  {textValue(provider.description, true, t)}
                </p>
                {provider.problem ? (
                  <p className="text-xs text-muted-foreground">
                    {textValue(provider.problem, true, t)}
                  </p>
                ) : null}
              </div>
              <Badge variant={provider.ready ? "default" : "outline"}>
                {provider.ready ? t("assistantSettings.ready") : t("assistantSettings.notReady")}
              </Badge>
            </div>
          </li>
        ))}
      </ul>

      <div className="space-y-1">
        <Label htmlFor={`${id}-provider`}>{t("assistantSettings.provider")}</Label>
        <Select
          id={`${id}-provider`}
          value={settings.providerId}
          onChange={(event) => setSettings({ ...settings, providerId: event.currentTarget.value })}
        >
          <option value="">{t("assistantSettings.firstReady")}</option>
          {state.providers.map((provider) => (
            <option key={provider.id} value={provider.id}>
              {textValue(provider.label, true, t) || provider.id}
            </option>
          ))}
        </Select>
      </div>

      <fieldset className="space-y-3">
        <legend className="text-sm font-medium">{t("assistantSettings.endpoint")}</legend>
        <div className="flex flex-wrap gap-2">
          {assistantPresets.map((preset) => (
            <Button
              key={preset.id}
              type="button"
              variant="outline"
              size="sm"
              onClick={() =>
                setSettings({
                  ...settings,
                  baseUrl: preset.baseUrl,
                  model: preset.model,
                  structuredOutput: preset.structuredOutput,
                })
              }
            >
              {t(`assistantSettings.presets.${preset.id}`)}
            </Button>
          ))}
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor={`${id}-url`}>{t("assistantSettings.baseUrl")}</Label>
            <Input
              id={`${id}-url`}
              value={settings.baseUrl}
              onChange={(event) => setSettings({ ...settings, baseUrl: event.currentTarget.value })}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor={`${id}-model`}>{t("assistantSettings.model")}</Label>
            <Input
              id={`${id}-model`}
              value={settings.model}
              onChange={(event) => setSettings({ ...settings, model: event.currentTarget.value })}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor={`${id}-key`}>{t("assistantSettings.apiKey")}</Label>
            <Input
              id={`${id}-key`}
              type="password"
              autoComplete="off"
              value={apiKey}
              placeholder={
                saved.apiKeySet ? t("assistantSettings.keyKept") : t("assistantSettings.keyMissing")
              }
              onChange={(event) => setApiKey(event.currentTarget.value)}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor={`${id}-output`}>{t("assistantSettings.structuredOutput")}</Label>
            <Select
              id={`${id}-output`}
              value={enumKey(StructuredOutput, settings.structuredOutput)}
              onChange={(event) =>
                setSettings({
                  ...settings,
                  structuredOutput:
                    StructuredOutput[event.currentTarget.value as (typeof OUTPUT_MODES)[number]],
                })
              }
            >
              {OUTPUT_MODES.map((mode) => (
                <option key={mode} value={mode}>
                  {t(`enums.structuredOutput.${mode}`)}
                </option>
              ))}
            </Select>
          </div>
        </div>
        <p className="text-xs text-muted-foreground">{t("assistantSettings.keyNote")}</p>
      </fieldset>

      <div className="flex flex-wrap items-center gap-2">
        <Button type="submit" disabled={saving}>
          {t("assistantSettings.save")}
        </Button>
        {saved.apiKeySet ? (
          <ConfirmButton
            type="button"
            variant="outline"
            disabled={saving}
            title={t("assistantSettings.confirmClearKey")}
            description={t("assistantSettings.confirmClearKeyBody")}
            confirmLabel={t("assistantSettings.clearKey")}
            cancelLabel={t("common.cancel")}
            onConfirm={() => void submit(settings, "").catch(() => undefined)}
          >
            {t("assistantSettings.clearKey")}
          </ConfirmButton>
        ) : null}
        {feedback ? (
          <span
            role={feedback.ok ? "status" : "alert"}
            className={feedback.ok ? "text-xs text-muted-foreground" : "text-xs text-destructive"}
          >
            {feedback.text}
          </span>
        ) : null}
        {error ? (
          <span role="alert" className="text-xs text-destructive">
            {errorMessage(error)}
          </span>
        ) : null}
      </div>
    </form>
  );
}

function saved(state: AssistantState): AssistantSettings {
  return state.settings ?? create(AssistantSettingsSchema);
}

export function AssistantSettingsPanel() {
  const { t } = useI18n();
  const { state } = useAssistantSettings();
  const [feedback, setFeedback] = useState<Feedback | null>(null);

  return (
    <SectionCard
      icon={Bot}
      title={t("assistantSettings.title")}
      description={t("assistantSettings.description")}
    >
      {state ? (
        <SettingsForm
          key={toJsonString(AssistantSettingsSchema, saved(state))}
          state={state}
          saved={saved(state)}
          feedback={feedback}
          onFeedback={setFeedback}
        />
      ) : (
        <p role="status" className="text-sm text-muted-foreground">
          {t("assistantSettings.loading")}
        </p>
      )}
    </SectionCard>
  );
}
