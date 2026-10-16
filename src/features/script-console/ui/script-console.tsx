import { FileCode2, Play, Plus, Save, Trash2 } from "lucide-react";
import { useId, useRef, useState } from "react";

import type { DeviceRecord } from "@/entities/device";
import {
  createScript,
  type LibraryScript,
  MAX_SCRIPT_NAME_CHARS,
  type ScriptBody,
  ScriptOutputList,
  scriptLanguages,
  useScriptActions,
  useScripts,
  useScriptsError,
} from "@/entities/script";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { cn } from "@/shared/lib/utils";
import { deviceTitle, taskClass, textValue } from "@/shared/lib/workspace-view";
import { enumKey, ScriptLanguage, TaskStatus } from "@/shared/proto";
import { useTheme } from "@/shared/theme";
import {
  Button,
  CodeEditor,
  ConfirmButton,
  ErrorState,
  Input,
  Label,
  SectionCard,
  Select,
} from "@/shared/ui";

import { editorLanguage, isUnsaved } from "../model/script-editing";
import { useRunScript } from "../model/use-run-script";
import { ScriptDraftAssistant } from "./script-draft-assistant";

type ScriptConsoleProps = {
  selectedDevices: readonly DeviceRecord[];
  devices: readonly DeviceRecord[];
};

export function ScriptConsole({ selectedDevices, devices }: ScriptConsoleProps) {
  const { t } = useI18n();
  const { isDark } = useTheme();
  const id = useId();
  const scripts = useScripts();
  const loadError = useScriptsError();
  const { save, remove } = useScriptActions();
  const { run, running, result, error } = useRunScript();
  const [draft, setDraft] = useState<LibraryScript | undefined>();
  const [problem, setProblem] = useState<string | undefined>();
  const [starting, setStarting] = useState(false);
  const startingNow = useRef(false);
  const saved = draft ? scripts.find((script) => script.id === draft.id) : undefined;
  const unsaved = draft ? isUnsaved(draft, saved) : false;
  const names = new Map(devices.map((device) => [device.id, deviceTitle(device, false, t)]));
  const titleOf = (deviceId: string) => names.get(deviceId) ?? t("scriptConsole.unknownDevice");

  const persist = async (script: LibraryScript) => {
    const normalized = { ...script, name: script.name.trim() || t("script.untitled") };

    await save(normalized);
    setDraft((current) =>
      current?.id === script.id && !isUnsaved(current, script) ? normalized : current,
    );
  };

  const guard = async (work: () => Promise<unknown>) => {
    setProblem(undefined);
    try {
      await work();
    } catch (failure) {
      setProblem(errorMessage(failure));
    }
  };

  const adoptInto = (targetId: string | undefined) => (script: ScriptBody) =>
    setDraft((current) => {
      if (current?.id !== targetId) {
        return current;
      }

      return current
        ? { ...current, language: script.language, body: script.body }
        : {
            ...createScript(script.language, script.name || t("scriptConsole.newName")),
            body: script.body,
          };
    });

  const runDraft = async (script: LibraryScript) => {
    if (startingNow.current) {
      return;
    }
    startingNow.current = true;
    setStarting(true);
    try {
      await guard(async () => {
        if (isUnsaved(script, saved)) {
          await persist(script);
        }
        await run(
          script.id,
          selectedDevices.map((device) => device.id),
        );
      });
    } finally {
      startingNow.current = false;
      setStarting(false);
    }
  };

  return (
    <div className="grid gap-6 xl:grid-cols-[18rem_1fr]">
      <SectionCard
        icon={FileCode2}
        title={t("scriptConsole.library.title")}
        contentClassName="space-y-3"
      >
        <div className="flex flex-wrap gap-2">
          {scriptLanguages.map((language) => (
            <Button
              key={language}
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setDraft(createScript(language, t("scriptConsole.newName")))}
            >
              <Plus className="h-4 w-4" />
              {t(runtimeKey(`enums.scriptLanguage.${enumKey(ScriptLanguage, language)}`))}
            </Button>
          ))}
        </div>
        {loadError ? (
          <ErrorState title={t("common.documentsError")} detail={errorMessage(loadError)} />
        ) : null}
        {scripts.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("scriptConsole.library.empty")}</p>
        ) : (
          <ul className="space-y-1">
            {scripts.map((script) => (
              <li key={script.id}>
                <button
                  type="button"
                  aria-current={draft?.id === script.id}
                  className={cn(
                    "w-full rounded-md px-3 py-2 text-left text-sm hover:bg-accent",
                    draft?.id === script.id && "bg-accent",
                  )}
                  onClick={() => setDraft({ ...script })}
                >
                  <span className="block truncate font-medium">
                    {script.name || t("script.untitled")}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {t(
                      runtimeKey(
                        `enums.scriptLanguage.${enumKey(ScriptLanguage, script.language)}`,
                      ),
                    )}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </SectionCard>

      <SectionCard icon={Play} title={t("scriptConsole.editor.title")} contentClassName="space-y-4">
        {draft ? (
          <>
            <div className="grid gap-3 sm:grid-cols-[1fr_12rem]">
              <div className="space-y-1">
                <Label htmlFor={`${id}-name`}>{t("scriptConsole.editor.name")}</Label>
                <Input
                  id={`${id}-name`}
                  value={draft.name}
                  maxLength={MAX_SCRIPT_NAME_CHARS}
                  onChange={(event) => setDraft({ ...draft, name: event.currentTarget.value })}
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor={`${id}-language`}>{t("scriptConsole.editor.language")}</Label>
                <Select
                  id={`${id}-language`}
                  value={enumKey(ScriptLanguage, draft.language)}
                  onChange={(event) =>
                    setDraft({
                      ...draft,
                      language:
                        ScriptLanguage[event.currentTarget.value as keyof typeof ScriptLanguage],
                    })
                  }
                >
                  {scriptLanguages.map((language) => (
                    <option key={language} value={enumKey(ScriptLanguage, language)}>
                      {t(runtimeKey(`enums.scriptLanguage.${enumKey(ScriptLanguage, language)}`))}
                    </option>
                  ))}
                </Select>
              </div>
            </div>
            <CodeEditor
              value={draft.body}
              onChange={(body) => setDraft({ ...draft, body })}
              language={editorLanguage(draft.language)}
              label={t("scriptConsole.editor.body")}
              dark={isDark}
            />
            <ScriptDraftAssistant
              key={draft.id}
              language={draft.language}
              currentBody={draft.body}
              onDraft={adoptInto(draft.id)}
            />
            <p className="text-xs text-muted-foreground">
              {selectedDevices.length === 0
                ? t("scriptConsole.noDevices")
                : t("scriptConsole.target", { count: String(selectedDevices.length) })}
            </p>
            <div className="flex flex-wrap gap-2">
              <Button
                type="button"
                disabled={running || starting || selectedDevices.length === 0 || !draft.body.trim()}
                onClick={() => void runDraft(draft)}
              >
                <Play className="h-4 w-4" />
                {t("scriptConsole.run")}
              </Button>
              <Button
                type="button"
                variant="outline"
                disabled={!unsaved}
                onClick={() => void guard(() => persist(draft))}
              >
                <Save className="h-4 w-4" />
                {t("scriptConsole.save")}
              </Button>
              {saved ? (
                <ConfirmButton
                  type="button"
                  variant="ghost"
                  title={t("scriptConsole.confirmRemove", {
                    name: saved.name || t("script.untitled"),
                  })}
                  description={t("common.irreversible")}
                  confirmLabel={t("scriptConsole.remove")}
                  cancelLabel={t("common.cancel")}
                  onConfirm={() =>
                    void guard(async () => {
                      await remove(saved.id);
                      setDraft((current) => (current?.id === saved.id ? undefined : current));
                    })
                  }
                >
                  <Trash2 className="h-4 w-4" />
                  {t("scriptConsole.remove")}
                </ConfirmButton>
              ) : null}
            </div>
            {unsaved ? (
              <p className="text-xs text-muted-foreground">{t("scriptConsole.unsaved")}</p>
            ) : null}
          </>
        ) : (
          <>
            <p className="text-sm text-muted-foreground">{t("scriptConsole.pick")}</p>
            <ScriptDraftAssistant
              language={ScriptLanguage.SHELL}
              currentBody=""
              onDraft={adoptInto(undefined)}
            />
          </>
        )}

        {problem || error ? (
          <p role="alert" className="break-words text-xs leading-5 text-destructive">
            {problem ?? errorMessage(error)}
          </p>
        ) : null}

        {result?.task ? (
          <section aria-label={t("scriptConsole.results")} className="space-y-3">
            <div className="flex items-start justify-between gap-3">
              <p role="status" className="break-words text-xs leading-5 text-muted-foreground">
                {textValue(result.task.detail, true, t)}
              </p>
              <span
                className={cn(
                  "shrink-0 text-xs font-medium",
                  taskClass(enumKey(TaskStatus, result.task.status)),
                )}
              >
                {t(runtimeKey(`enums.taskStatus.${enumKey(TaskStatus, result.task.status)}`))}
              </span>
            </div>
            <ScriptOutputList outputs={result.scriptOutputs} titleOf={titleOf} />
          </section>
        ) : null}
      </SectionCard>
    </div>
  );
}
