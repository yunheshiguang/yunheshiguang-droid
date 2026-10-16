import { timestampDate } from "@bufbuild/protobuf/wkt";
import { Download, ExternalLink, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";

import { availableUpdate, openReleaseLink, shortCommit } from "@/entities/release";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import { type AvailableUpdate, type UiText, type UpdateState, UpdateStatus } from "@/shared/proto";
import { Badge, Button, SectionCard } from "@/shared/ui";

import { useUpdateSettings } from "../model/use-update-settings";

const CHANNELS = {
  nightly: "release.channel.nightly",
  release: "release.channel.release",
} as const;

function channelKey(channel: string | undefined) {
  return channel === "nightly" || channel === "release"
    ? CHANNELS[channel]
    : "release.channel.local";
}

function StatusLine({ state }: { state: UpdateState }) {
  const { t, locale } = useI18n();
  const timeFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "short" }),
    [locale],
  );

  switch (state.status) {
    case UpdateStatus.LOCAL_BUILD:
      return (
        <p role="status" className="text-sm text-muted-foreground">
          {t("settings.updates.status.localBuild")}
        </p>
      );
    case UpdateStatus.CHECKING:
      return (
        <p role="status" className="text-sm text-muted-foreground">
          {t("settings.updates.status.checking")}
        </p>
      );
    case UpdateStatus.UP_TO_DATE:
      return (
        <p role="status" className="text-sm text-emerald-600 dark:text-emerald-400">
          {state.checkedAt
            ? t("settings.updates.status.upToDate", {
                time: timeFormat.format(timestampDate(state.checkedAt)),
              })
            : t("settings.updates.status.latest")}
        </p>
      );
    case UpdateStatus.FAILED:
      return (
        <p role="status" className="break-words text-sm text-destructive">
          {t("settings.updates.status.failed")}
          {state.problem ? ` · ${textValue(state.problem, true, t)}` : ""}
        </p>
      );
    case UpdateStatus.AVAILABLE:
      return null;
    default:
      return (
        <p role="status" className="text-sm text-muted-foreground">
          {t("settings.updates.status.idle")}
        </p>
      );
  }
}

function UpdateOffer({ update }: { update: AvailableUpdate }) {
  const { t } = useI18n();
  const title =
    update.channel === "nightly"
      ? t("release.available.nightly", { commit: shortCommit(update.commit) })
      : t("release.available.release", { version: update.version });

  return (
    <div className="space-y-2 rounded-md border border-border p-3">
      <p role="status" className="text-sm font-medium">
        {title}
      </p>
      <div className="flex flex-wrap gap-2">
        {update.downloadUrl ? (
          <Button type="button" size="sm" onClick={() => openReleaseLink(update.downloadUrl)}>
            <Download className="h-4 w-4" />
            {t("settings.updates.download", { name: update.downloadName })}
          </Button>
        ) : null}
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => openReleaseLink(update.pageUrl)}
        >
          <ExternalLink className="h-4 w-4" />
          {t("settings.updates.releaseNotes")}
        </Button>
      </div>
      {update.downloadUrl ? null : (
        <p className="text-xs text-muted-foreground">{t("settings.updates.noDownload")}</p>
      )}
    </div>
  );
}

export function UpdateSettings() {
  const { t } = useI18n();
  const { state, check, checking, setAutomatic, saving } = useUpdateSettings();
  const [problem, setProblem] = useState<UiText | string | undefined>();

  if (!state) {
    return null;
  }

  const local = state.status === UpdateStatus.LOCAL_BUILD;
  const build = state.build;
  const update = availableUpdate(state);
  const toggle = async (checkAutomatically: boolean) => {
    try {
      const result = await setAutomatic(checkAutomatically);
      setProblem(result.accepted ? undefined : result.detail);
    } catch (error) {
      setProblem(errorMessage(error));
    }
  };
  const checkNow = async () => {
    try {
      await check();
      setProblem(undefined);
    } catch (error) {
      setProblem(errorMessage(error));
    }
  };

  return (
    <SectionCard icon={RefreshCw} title={t("settings.updates.title")}>
      <div className="space-y-4">
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <span className="text-muted-foreground">{t("settings.updates.installed")}</span>
          <span className="font-medium">{build?.version}</span>
          <Badge variant="outline">{t(channelKey(build?.channel))}</Badge>
          {build?.commit ? (
            <span className="font-mono text-xs text-muted-foreground">
              {t("release.commit", { commit: shortCommit(build.commit) })}
            </span>
          ) : null}
        </div>

        <StatusLine state={state} />
        {update ? <UpdateOffer update={update} /> : null}

        <label className="flex items-start gap-2 text-sm">
          <input
            type="checkbox"
            className="mt-1"
            checked={state.checkAutomatically}
            disabled={local || saving}
            onChange={(event) => void toggle(event.currentTarget.checked)}
          />
          <span>{t("settings.updates.automatic")}</span>
        </label>

        {problem ? (
          <p role="alert" className="break-words text-sm text-destructive">
            {typeof problem === "string" ? problem : textValue(problem, true, t)}
          </p>
        ) : null}

        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={local || checking}
            onClick={() => void checkNow()}
          >
            <RefreshCw className={cn("h-4 w-4", checking && "animate-spin")} />
            {t("settings.updates.checkNow")}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => openReleaseLink(state.releasesUrl)}
          >
            <ExternalLink className="h-4 w-4" />
            {t("settings.updates.allReleases")}
          </Button>
        </div>
      </div>
    </SectionCard>
  );
}
