import { Cpu } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { enumKey, ScriptLanguage } from "@/shared/proto";
import { SectionCard } from "@/shared/ui";

import { useScriptEngines } from "../model/use-run-script";

export function ScriptEngineList() {
  const { t } = useI18n();
  const engines = useScriptEngines();

  return (
    <SectionCard icon={Cpu} title={t("scriptConsole.engines.title")} contentClassName="space-y-3">
      <ol className="space-y-3">
        {engines.map((engine) => (
          <li key={engine.id} className="space-y-1 rounded-md border border-border p-3">
            <div className="text-sm font-medium">
              {textValue(engine.label, true, t) || engine.id}
            </div>
            {engine.description ? (
              <p className="text-xs leading-5 text-muted-foreground">
                {textValue(engine.description, true, t)}
              </p>
            ) : null}
            <p className="text-xs text-muted-foreground">
              {engine.languages
                .map((language) =>
                  t(runtimeKey(`enums.scriptLanguage.${enumKey(ScriptLanguage, language)}`)),
                )
                .join(" · ")}
              {" · "}
              {engine.capturesOutput
                ? t("scriptConsole.engines.capturesOutput")
                : t("scriptConsole.engines.launchOnly")}
            </p>
            {engine.setup.length > 0 ? (
              <ol className="list-decimal space-y-1 pl-5 text-xs leading-5">
                {engine.setup.map((step) => (
                  <li key={step.key}>{textValue(step, true, t)}</li>
                ))}
              </ol>
            ) : null}
          </li>
        ))}
      </ol>
    </SectionCard>
  );
}
