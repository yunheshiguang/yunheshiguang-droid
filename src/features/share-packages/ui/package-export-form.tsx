import { PackagePlus } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import { useAutomationFlows } from "@/entities/automation";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import { Button, Input, Label } from "@/shared/ui";

import { type PackageManifestFields, useExportPackage } from "../model/use-template-packages";

const FIELDS = ["id", "name", "version", "author", "description"] as const;

export function PackageExportForm() {
  const { t } = useI18n();
  const id = useId();
  const flows = useAutomationFlows();
  const { exportAs, exporting } = useExportPackage();
  const [chosen, setChosen] = useState<ReadonlySet<string>>(new Set());
  const [manifest, setManifest] = useState<PackageManifestFields>({
    id: "",
    name: "",
    version: "1.0.0",
    author: "",
    description: "",
  });
  const [outcome, setOutcome] = useState<{ ok: boolean; text: string } | null>(null);

  const toggle = (flowId: string) => {
    const next = new Set(chosen);
    if (!next.delete(flowId)) {
      next.add(flowId);
    }
    setChosen(next);
  };
  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    exportAs({ manifest, flowIds: [...chosen] })
      .then((result) => {
        if (result) {
          setOutcome({ ok: result.accepted, text: textValue(result.detail, true, t) });
        }
      })
      .catch((error: unknown) => setOutcome({ ok: false, text: errorMessage(error) }));
  };

  return (
    <form onSubmit={submit} aria-label={t("sharePackages.export.title")} className="space-y-3">
      <fieldset className="space-y-1">
        <legend className="text-sm font-medium">{t("sharePackages.export.flows")}</legend>
        {flows.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t("sharePackages.export.noFlows")}</p>
        ) : (
          flows.map((flow) => (
            <label key={flow.id} className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={chosen.has(flow.id)}
                onChange={() => toggle(flow.id)}
              />
              {flow.name}
            </label>
          ))
        )}
      </fieldset>
      <div className="grid gap-3 sm:grid-cols-2">
        {FIELDS.map((field) => (
          <div key={field} className="space-y-1">
            <Label htmlFor={`${id}-${field}`}>{t(`sharePackages.manifest.${field}`)}</Label>
            <Input
              id={`${id}-${field}`}
              value={manifest[field]}
              placeholder={t(`sharePackages.placeholders.${field}`)}
              onChange={(event) => setManifest({ ...manifest, [field]: event.currentTarget.value })}
            />
          </div>
        ))}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button type="submit" size="sm" disabled={exporting || chosen.size === 0}>
          <PackagePlus className="h-4 w-4" />
          {t("sharePackages.export.submit")}
        </Button>
        {outcome ? (
          <span
            role={outcome.ok ? "status" : "alert"}
            className={
              outcome.ok ? "break-all text-xs text-muted-foreground" : "text-xs text-destructive"
            }
          >
            {outcome.text}
          </span>
        ) : null}
      </div>
    </form>
  );
}
