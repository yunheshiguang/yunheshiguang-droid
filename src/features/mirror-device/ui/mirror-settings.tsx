import { equals, toJsonString } from "@bufbuild/protobuf";
import { Download, LoaderCircle, MonitorSmartphone, Save } from "lucide-react";
import { useState } from "react";

import { dialogGateway } from "@/shared/api";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import {
  enumKey,
  ScrcpyBinaryKind,
  ScrcpyBinarySource,
  type ScrcpyConfig,
  ScrcpyConfigSchema,
  ScrcpyInstallStatus,
  type ScrcpyState,
} from "@/shared/proto";
import { Button, ErrorState, Label, PathInput, SectionCard, Select } from "@/shared/ui";

import { defaultScrcpyConfig } from "../model/default-config";
import { useMirrorCommands } from "../model/use-mirror-commands";
import { MirrorOptions } from "./mirror-options";

const SOURCES = ["APP_PRIVATE", "SYSTEM_PATH", "MANUAL_DIRECTORY"] as const;

type MirrorSettingsProps = {
  state?: ScrcpyState;
};

export function MirrorSettings({ state }: MirrorSettingsProps) {
  const config = state?.config ?? defaultScrcpyConfig;

  return (
    <MirrorSettingsForm
      key={toJsonString(ScrcpyConfigSchema, config)}
      state={state}
      saved={config}
    />
  );
}

function MirrorSettingsForm({ state, saved }: MirrorSettingsProps & { saved: ScrcpyConfig }) {
  const { t } = useI18n();
  const commands = useMirrorCommands();
  const [draft, setDraft] = useState(saved);
  const [error, setError] = useState<string | null>(null);
  const install = state?.install;
  const changed = !equals(ScrcpyConfigSchema, draft, saved);
  const locked = commands.busy || !state;

  const save = () => {
    setError(null);
    commands.apply(draft).catch((reason: unknown) => setError(errorMessage(reason)));
  };

  return (
    <SectionCard
      icon={MonitorSmartphone}
      title={t("mirrorDevice.settings.title")}
      description={t("mirrorDevice.settings.description")}
      contentClassName="space-y-5"
    >
      <div className="flex flex-wrap items-center gap-3">
        <Button type="button" variant="outline" disabled={locked} onClick={commands.install}>
          {commands.installing ? (
            <LoaderCircle className="h-4 w-4 animate-spin" />
          ) : (
            <Download className="h-4 w-4" />
          )}
          {commands.installing
            ? t("mirrorDevice.settings.installing")
            : t("mirrorDevice.settings.install")}
        </Button>
        {install?.status === ScrcpyInstallStatus.INSTALLED ? (
          <span className="text-sm text-muted-foreground">
            {t("mirrorDevice.settings.installed", { release: install.releaseTag })}
          </span>
        ) : null}
      </div>

      <div className="grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
        <div className="space-y-1.5">
          <Label htmlFor="scrcpy-source">{t("mirrorDevice.settings.source")}</Label>
          <Select
            id="scrcpy-source"
            value={enumKey(ScrcpyBinarySource, draft.source)}
            disabled={locked}
            onChange={(event) =>
              setDraft({
                ...draft,
                source: ScrcpyBinarySource[event.currentTarget.value as (typeof SOURCES)[number]],
              })
            }
          >
            {SOURCES.map((source) => (
              <option key={source} value={source}>
                {t(runtimeKey(`enums.scrcpyBinarySource.${source}`))}
              </option>
            ))}
          </Select>
        </div>
        {draft.source === ScrcpyBinarySource.MANUAL_DIRECTORY ? (
          <div className="space-y-1.5">
            <Label htmlFor="scrcpy-root">{t("mirrorDevice.settings.root")}</Label>
            <PathInput
              id="scrcpy-root"
              value={draft.scrcpyRoot}
              onValueChange={(scrcpyRoot) => setDraft({ ...draft, scrcpyRoot })}
              onBrowse={() => dialogGateway().pickPath({ target: "directory" })}
              browseLabel={t("mirrorDevice.settings.browse")}
              disabled={locked}
            />
          </div>
        ) : null}
      </div>

      <MirrorOptions config={draft} onChange={setDraft} disabled={locked} />

      <Button type="button" disabled={!changed || locked} onClick={save}>
        {commands.busy && !commands.installing ? (
          <LoaderCircle className="h-4 w-4 animate-spin" />
        ) : (
          <Save className="h-4 w-4" />
        )}
        {commands.busy && !commands.installing
          ? t("mirrorDevice.settings.saving")
          : t("mirrorDevice.settings.save")}
      </Button>

      {error ? <ErrorState title={error} /> : null}

      <div className="grid gap-2 md:grid-cols-3">
        {(state?.binaries ?? []).map((binary) => (
          <div key={binary.kind} className="rounded-md border border-border bg-background p-3">
            <div className="text-sm font-medium">
              {t(runtimeKey(`enums.scrcpyBinaryKind.${enumKey(ScrcpyBinaryKind, binary.kind)}`))}
            </div>
            <div className="mt-1 break-all text-xs text-muted-foreground">
              {binary.available ? binary.resolvedPath : t("mirrorDevice.settings.missing")}
            </div>
          </div>
        ))}
      </div>

      {state?.diagnostics.map((item) => (
        <div key={item.id} className="rounded-md border border-border bg-background p-3">
          <div className="text-sm font-medium">{textValue(item.title, true, t)}</div>
          <div className="mt-1 text-sm leading-6 text-muted-foreground">
            {textValue(item.impact, true, t)}
          </div>
          <div className="mt-1 text-sm leading-6">{textValue(item.recovery, true, t)}</div>
        </div>
      ))}
    </SectionCard>
  );
}
