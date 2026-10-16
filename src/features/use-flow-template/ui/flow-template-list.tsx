import { useRef, useState } from "react";

import { useAutomationFlowActions } from "@/entities/automation";
import { resolveBatchAction } from "@/entities/task";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { FlowTemplate } from "@/shared/proto";
import { Button, ErrorState } from "@/shared/ui";

import { flowFromTemplate } from "../model/flow-from-template";
import { useFlowTemplates } from "../model/use-flow-templates";

type FlowTemplateListProps = {
  sourceId?: string;
};

export function FlowTemplateList({ sourceId }: FlowTemplateListProps) {
  const { t } = useI18n();
  const { catalog, loading, error } = useFlowTemplates(sourceId);
  const { save } = useAutomationFlowActions();
  const [added, setAdded] = useState<string | null>(null);
  const [writeError, setWriteError] = useState<string | null>(null);
  const [using, setUsing] = useState<string | null>(null);
  const usingNow = useRef(false);

  const use = (template: FlowTemplate) => {
    if (usingNow.current) {
      return;
    }
    const name = textValue(template.name, true, t) || template.id;
    const key = `${template.sourceId}/${template.id}`;

    usingNow.current = true;
    setUsing(key);
    setWriteError(null);
    save(flowFromTemplate(template, name))
      .then(
        () => setAdded(name),
        (failure: unknown) => setWriteError(errorMessage(failure)),
      )
      .finally(() => {
        usingNow.current = false;
        setUsing(null);
      });
  };

  if (error) {
    return <ErrorState title={t("useFlowTemplate.loadFailed")} detail={error} />;
  }

  if (loading) {
    return <p className="text-sm text-muted-foreground">{t("useFlowTemplate.loading")}</p>;
  }

  if (catalog.templates.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("useFlowTemplate.empty")}</p>;
  }

  return (
    <div className="space-y-4">
      {catalog.sources.map((source) => (
        <section
          key={source.id}
          aria-label={textValue(source.label, true, t)}
          className="space-y-2"
        >
          <h3 className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {textValue(source.label, true, t)}
          </h3>
          <ul className="space-y-2">
            {catalog.templates
              .filter((template) => template.sourceId === source.id)
              .map((template) => {
                const name = textValue(template.name, true, t) || template.id;

                return (
                  <li
                    key={`${template.sourceId}/${template.id}`}
                    className="flex items-start justify-between gap-3 rounded-md border border-border bg-background p-3"
                  >
                    <div className="min-w-0 space-y-1">
                      <p className="text-sm font-medium">{name}</p>
                      <p className="text-xs leading-5 text-muted-foreground">
                        {textValue(template.description, true, t)}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {template.steps
                          .map((step) => {
                            const action = resolveBatchAction(step.kind);

                            return action ? t(action.labelKey) : step.kind;
                          })
                          .join(" → ")}
                      </p>
                    </div>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={using === `${template.sourceId}/${template.id}`}
                      onClick={(event) => {
                        if (event.detail <= 1) {
                          use(template);
                        }
                      }}
                      aria-label={t("useFlowTemplate.useNamed", { name })}
                    >
                      {t("useFlowTemplate.use")}
                    </Button>
                  </li>
                );
              })}
          </ul>
        </section>
      ))}
      {added ? (
        <p role="status" className="text-xs text-muted-foreground">
          {t("useFlowTemplate.added", { name: added })}
        </p>
      ) : null}
      {writeError ? (
        <ErrorState title={t("useFlowTemplate.addFailed")} detail={writeError} />
      ) : null}
    </div>
  );
}
