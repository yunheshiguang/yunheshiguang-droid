import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import { useId } from "react";

import type { AutomationStep } from "@/entities/automation";
import { PhrasePicker } from "@/entities/phrase";
import { ScriptPicker } from "@/entities/script";
import { BatchActionValueInput, listBatchActions, resolveBatchAction } from "@/entities/task";
import { useI18n } from "@/shared/i18n";
import { BatchActionKind, enumKey } from "@/shared/proto";
import { Button, Label, Select } from "@/shared/ui";

import { stepTakesValue } from "../model/flow-issues";

type FlowStepRowProps = {
  step: AutomationStep;
  index: number;
  count: number;
  missingValue: boolean;
  onChange: (step: AutomationStep) => void;
  onMove: (offset: -1 | 1) => void;
  onRemove: () => void;
};

export function FlowStepRow({
  step,
  index,
  count,
  missingValue,
  onChange,
  onMove,
  onRemove,
}: FlowStepRowProps) {
  const { t } = useI18n();
  const id = useId();
  const position = String(index + 1);
  const action = resolveBatchAction(step.kind);
  const input = action?.input;

  return (
    <li className="space-y-2 rounded-md border border-border bg-background p-3">
      <div className="flex items-center gap-2">
        <Label htmlFor={`${id}-action`} className="shrink-0 text-xs text-muted-foreground">
          {t("editAutomationFlow.stepLabel", { index: position })}
        </Label>
        <Select
          id={`${id}-action`}
          className="min-w-0 flex-1"
          value={enumKey(BatchActionKind, step.kind)}
          onChange={(event) =>
            onChange({
              ...step,
              kind: BatchActionKind[event.currentTarget.value as keyof typeof BatchActionKind],
              value: "",
            })
          }
        >
          {action ? null : (
            <option value={enumKey(BatchActionKind, step.kind)} disabled>
              {t("editAutomationFlow.unknownAction")}
            </option>
          )}
          {listBatchActions().map((known) => (
            <option key={known.kind} value={enumKey(BatchActionKind, known.kind)}>
              {t(known.labelKey)}
            </option>
          ))}
        </Select>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          disabled={index === 0}
          onClick={() => onMove(-1)}
          aria-label={t("editAutomationFlow.moveUp", { index: position })}
        >
          <ArrowUp className="h-4 w-4" />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          disabled={index === count - 1}
          onClick={() => onMove(1)}
          aria-label={t("editAutomationFlow.moveDown", { index: position })}
        >
          <ArrowDown className="h-4 w-4" />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onRemove}
          aria-label={t("editAutomationFlow.removeStep", { index: position })}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      {action ? null : (
        <p className="text-xs text-destructive">{t("editAutomationFlow.issues.unknownAction")}</p>
      )}
      {stepTakesValue(step) ? (
        <div className="space-y-1">
          <Label htmlFor={`${id}-value`} className="sr-only">
            {t("editAutomationFlow.value", { index: position })}
          </Label>
          {input === "script" ? (
            <ScriptPicker
              id={`${id}-value`}
              value={step.value}
              onValueChange={(value) => onChange({ ...step, value })}
            />
          ) : (
            <BatchActionValueInput
              id={`${id}-value`}
              kind={step.kind}
              value={step.value}
              onValueChange={(value) => onChange({ ...step, value })}
            />
          )}
          {input === "text" ? (
            <PhrasePicker onPick={(value) => onChange({ ...step, value })} />
          ) : null}
          {missingValue ? (
            <p className="text-xs text-destructive">{t("editAutomationFlow.issues.value")}</p>
          ) : null}
        </div>
      ) : null}
    </li>
  );
}
