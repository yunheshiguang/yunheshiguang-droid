import { Download, ExternalLink, Terminal } from "lucide-react";

import { windowGateway } from "@/shared/api";
import { useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import type { AndroidPlatformToolsState } from "@/shared/proto";
import { Button, ErrorState, SectionCard } from "@/shared/ui";

import { androidPlatformToolsReleaseUrl } from "../model/downloads";
import { usePlatformToolsCommands } from "../model/use-platform-tools-commands";
import { usePlatformToolsForm } from "../model/use-platform-tools-form";
import { BinaryStatus } from "./binary-status";
import { InstallPanel } from "./install-panel";
import { SourceForm } from "./source-form";

type AndroidPlatformToolsSettingsProps = {
  state?: AndroidPlatformToolsState;
};

function openExternalUrl(url: string) {
  void windowGateway().openExternal(url);
}

export function AndroidPlatformToolsSettings({ state }: AndroidPlatformToolsSettingsProps) {
  const { t } = useI18n();
  const form = usePlatformToolsForm(state, usePlatformToolsCommands());
  const sourceUrl = form.install?.sourceUrl;

  return (
    <SectionCard
      icon={Terminal}
      title={t("androidPlatformTools.title")}
      description={t("androidPlatformTools.description")}
      contentClassName="space-y-5"
    >
      <InstallPanel
        install={form.install}
        status={form.visibleInstallStatus}
        licenseAccepted={form.licenseAccepted}
        onLicenseAcceptedChange={form.setLicenseAccepted}
        busy={form.busy}
        installing={form.installing}
        onInstall={() => void form.installPrivateTools()}
        onOpenExternal={openExternalUrl}
      />

      <SourceForm
        source={form.source}
        onSourceChange={form.setSource}
        root={form.root}
        onRootChange={form.setRoot}
        rootEditable={form.sourceNeedsRoot}
        busy={form.busy}
        canApply={form.canApply}
        onApply={() => void form.applyCurrent()}
        onReset={() => void form.resetToSystemPath()}
      />

      {form.error ? <ErrorState title={form.error} /> : null}

      <div className="grid gap-3 md:grid-cols-2">
        {(state?.binaries ?? []).map((binary) => (
          <BinaryStatus key={binary.kind} binary={binary} />
        ))}
      </div>

      {state?.diagnostics.length ? (
        <div className="space-y-2">
          {state.diagnostics.map((item) => (
            <div key={item.id} className="rounded-md border border-border bg-background p-3">
              <div className="text-sm font-medium">{textValue(item.title, true, t)}</div>
              <div className="mt-1 text-sm leading-6 text-muted-foreground">
                {textValue(item.impact, true, t)}
              </div>
              <div className="mt-1 text-sm leading-6">{textValue(item.recovery, true, t)}</div>
            </div>
          ))}
        </div>
      ) : null}

      <div className="flex flex-wrap items-center gap-3">
        <Button
          type="button"
          variant="subtle"
          onClick={() => openExternalUrl(androidPlatformToolsReleaseUrl)}
        >
          <ExternalLink className="h-4 w-4" />
          {t("androidPlatformTools.releaseNotes")}
        </Button>
        {sourceUrl ? (
          <Button type="button" variant="ghost" onClick={() => openExternalUrl(sourceUrl)}>
            <Download className="h-4 w-4" />
            {t("androidPlatformTools.sourcePackage")}
          </Button>
        ) : null}
      </div>
    </SectionCard>
  );
}
