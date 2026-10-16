import { FileDown, Package, Store } from "lucide-react";
import { useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { PackagePreview } from "@/shared/proto";
import { Button, SectionCard, Separator } from "@/shared/ui";

import {
  fileOrigin,
  marketOrigin,
  type PackageOrigin,
  pickPackageFile,
  useInspectPackage,
  useMarketPackages,
} from "../model/use-template-packages";
import { PackageExportForm } from "./package-export-form";
import { PackagePreviewCard } from "./package-preview";

export function TemplatePackages() {
  const { t } = useI18n();
  const market = useMarketPackages();
  const { inspect, inspecting } = useInspectPackage();
  const [opened, setOpened] = useState<{ origin: PackageOrigin; preview: PackagePreview } | null>(
    null,
  );
  const [problem, setProblem] = useState<string | null>(null);
  const [installed, setInstalled] = useState<string | null>(null);

  const open = async (origin: PackageOrigin) => {
    setProblem(null);
    setInstalled(null);
    try {
      const result = await inspect(origin);
      if (result.accepted && result.preview) {
        setOpened({ origin, preview: result.preview });
      } else {
        setOpened(null);
        setProblem(textValue(result.detail, true, t) || t("sharePackages.unreadable"));
      }
    } catch (error) {
      setProblem(errorMessage(error));
    }
  };
  const openFile = async () => {
    const path = await pickPackageFile();
    if (path) {
      await open(fileOrigin(path));
    }
  };

  return (
    <SectionCard
      icon={Package}
      title={t("sharePackages.title")}
      description={t("sharePackages.description")}
      contentClassName="space-y-4"
    >
      <PackageExportForm />
      <Separator />
      <div className="space-y-3">
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={inspecting}
          onClick={() => void openFile()}
        >
          <FileDown className="h-4 w-4" />
          {t("sharePackages.import")}
        </Button>
        {market.length > 0 ? (
          <div className="space-y-2">
            <p className="flex items-center gap-2 text-sm font-medium">
              <Store className="h-4 w-4" />
              {t("sharePackages.market")}
            </p>
            <ul className="space-y-2">
              {market.map((item) => (
                <li
                  key={`${item.sourceId}/${item.manifest?.id}`}
                  className="flex items-center justify-between gap-3 rounded-md border border-border p-3"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">
                      {item.manifest?.name}{" "}
                      {item.manifest?.version ? `v${item.manifest.version}` : null}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {textValue(item.source, true, t)}
                      {` · ${t("sharePackages.counts", { flows: String(item.flows), scripts: String(item.scripts) })}`}
                    </p>
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={inspecting}
                    onClick={() => void open(marketOrigin(item.sourceId, item.manifest?.id ?? ""))}
                  >
                    {t("sharePackages.view")}
                  </Button>
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        {problem ? (
          <p role="alert" className="break-words text-xs text-destructive">
            {problem}
          </p>
        ) : null}
        {installed ? (
          <p role="status" className="break-words text-xs text-muted-foreground">
            {installed}
          </p>
        ) : null}
        {opened ? (
          <PackagePreviewCard
            key={JSON.stringify(opened.origin)}
            origin={opened.origin}
            preview={opened.preview}
            onTrusted={() => void open(opened.origin)}
            onInstalled={(message) => {
              setOpened(null);
              setInstalled(message);
            }}
            onDone={() => setOpened(null)}
          />
        ) : null}
      </div>
    </SectionCard>
  );
}
