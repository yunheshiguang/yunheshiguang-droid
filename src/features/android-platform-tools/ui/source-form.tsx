import { FolderInput, RefreshCw } from "lucide-react";

import { dialogGateway } from "@/shared/api";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { Button, Label, PathInput, Select } from "@/shared/ui";

import {
  platformToolsSources as SOURCE_OPTIONS,
  type PlatformToolsSource as Source,
} from "../model/use-platform-tools-form";

export type SourceFormProps = {
  source: Source;
  onSourceChange: (source: Source) => void;
  root: string;
  onRootChange: (root: string) => void;
  rootEditable: boolean;
  busy: boolean;
  canApply: boolean;
  onApply: () => void;
  onReset: () => void;
};

export function SourceForm({
  source,
  onSourceChange,
  root,
  onRootChange,
  rootEditable,
  busy,
  canApply,
  onApply,
  onReset,
}: SourceFormProps) {
  const { t } = useI18n();

  return (
    <div className="grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
      <div className="space-y-2">
        <Label htmlFor="android-platform-tools-source">{t("androidPlatformTools.source")}</Label>
        <Select
          id="android-platform-tools-source"
          value={source}
          onChange={(event) => onSourceChange(event.currentTarget.value as Source)}
          disabled={busy}
        >
          {SOURCE_OPTIONS.map((option) => (
            <option key={option} value={option}>
              {t(runtimeKey(`enums.androidPlatformToolsSource.${option}`))}
            </option>
          ))}
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="android-platform-tools-root">{t("androidPlatformTools.directory")}</Label>
        <div className="flex gap-2">
          <PathInput
            id="android-platform-tools-root"
            className="flex-1"
            value={root}
            onValueChange={onRootChange}
            onBrowse={() => dialogGateway().pickPath({ target: "directory" })}
            browseLabel={t("common.browse")}
            placeholder={t("androidPlatformTools.directoryPlaceholder")}
            disabled={busy || !rootEditable}
          />
          <Button
            type="button"
            variant="outline"
            size="icon"
            title={t("androidPlatformTools.apply")}
            disabled={!canApply}
            onClick={onApply}
          >
            <FolderInput className="h-4 w-4" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            title={t("androidPlatformTools.reset")}
            disabled={busy}
            onClick={onReset}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
