import { useEffect, useMemo, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import {
  AndroidPlatformToolsInstallStatus,
  AndroidPlatformToolsSource,
  type AndroidPlatformToolsState,
  DiagnosticCode,
  enumKey,
} from "@/shared/proto";

import type { PlatformToolsCommands } from "./use-platform-tools-commands";

export const platformToolsSources = ["APP_PRIVATE", "SYSTEM_PATH", "MANUAL_DIRECTORY"] as const;

export type PlatformToolsSource = (typeof platformToolsSources)[number];

function sourceName(source: AndroidPlatformToolsSource | undefined, fallback: PlatformToolsSource) {
  return (
    platformToolsSources.find((name) => AndroidPlatformToolsSource[name] === source) ?? fallback
  );
}

function denialOf(state: AndroidPlatformToolsState) {
  return state.diagnostics.find(
    (diagnostic) => diagnostic.code === DiagnosticCode.ACTION_NOT_AUTHORIZED,
  );
}

function configFromState(state?: AndroidPlatformToolsState) {
  return {
    source: sourceName(state?.config?.source, "SYSTEM_PATH"),
    platformToolsRoot: state?.config?.platformToolsRoot ?? "",
  };
}

export function usePlatformToolsForm(
  state: AndroidPlatformToolsState | undefined,
  commands: PlatformToolsCommands,
) {
  const { t } = useI18n();
  const initialConfig = useMemo(() => configFromState(state), [state]);
  const [source, setSource] = useState(initialConfig.source);
  const [root, setRoot] = useState(initialConfig.platformToolsRoot);
  const [licenseAccepted, setLicenseAccepted] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { applying, installing } = commands;
  const install = state?.install;
  const lastAttempt = state?.lastInstallAttempt;
  const busy = applying || installing;
  const sourceNeedsRoot = source === "MANUAL_DIRECTORY";
  const sourceUsesInstalledTools = source === "APP_PRIVATE";

  const visibleInstallStatus = enumKey(
    AndroidPlatformToolsInstallStatus,
    lastAttempt?.status === AndroidPlatformToolsInstallStatus.INSTALLING ||
      lastAttempt?.status === AndroidPlatformToolsInstallStatus.FAILED
      ? lastAttempt.status
      : (install?.status ?? AndroidPlatformToolsInstallStatus.NOT_INSTALLED),
  );

  useEffect(() => {
    if (busy) {
      return;
    }

    setSource(initialConfig.source);
    setRoot(initialConfig.platformToolsRoot);
  }, [busy, initialConfig.platformToolsRoot, initialConfig.source]);

  async function saveConfig(name: PlatformToolsSource, platformToolsRoot: string) {
    setError(null);

    try {
      const nextState = await commands.apply({
        source: AndroidPlatformToolsSource[name],
        platformToolsRoot,
      });
      const denial = denialOf(nextState);

      setSource(sourceName(nextState.config?.source, "SYSTEM_PATH"));
      setRoot(nextState.config?.platformToolsRoot ?? "");
      if (denial) {
        setError(textValue(denial.impact, true, t));
      }
    } catch (saveError) {
      setError(errorMessage(saveError));
    }
  }

  async function installPrivateTools() {
    if (!licenseAccepted) {
      setError(t("androidPlatformTools.licenseRequired"));
      return;
    }

    setError(null);

    try {
      const nextState = await commands.install(licenseAccepted);
      const denial = denialOf(nextState);

      setSource(sourceName(nextState.config?.source, "APP_PRIVATE"));
      setRoot(nextState.config?.platformToolsRoot ?? "");
      setLicenseAccepted(false);
      if (denial) {
        setError(textValue(denial.impact, true, t));
      }
    } catch (installError) {
      setError(errorMessage(installError));
    }
  }

  return {
    install,
    visibleInstallStatus,
    source,
    setSource,
    root,
    setRoot,
    licenseAccepted,
    setLicenseAccepted,
    busy,
    installing,
    error,
    sourceNeedsRoot,
    canApply:
      !busy &&
      (!sourceNeedsRoot || root.trim().length > 0) &&
      (!sourceUsesInstalledTools ||
        install?.status === AndroidPlatformToolsInstallStatus.INSTALLED),
    applyCurrent: () => saveConfig(source, sourceNeedsRoot ? root.trim() : ""),
    resetToSystemPath: () => saveConfig("SYSTEM_PATH", ""),
    installPrivateTools,
  };
}
