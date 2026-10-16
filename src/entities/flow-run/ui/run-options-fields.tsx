import { useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { Input, Label } from "@/shared/ui";

import { clampRunOption, type RunOptionValues, runOptionLimits } from "../model/run-options";

type RunOptionsFieldsProps = {
  value: RunOptionValues;
  onChange: (value: RunOptionValues) => void;
  disabled?: boolean;
};

const fields: readonly (keyof RunOptionValues)[] = [
  "repeat",
  "repeatIntervalSeconds",
  "retryAttempts",
  "retryDelaySeconds",
];

function OptionField({
  id,
  field,
  value,
  onCommit,
}: {
  id: string;
  field: keyof RunOptionValues;
  value: number;
  onCommit: (value: number) => void;
}) {
  const { t } = useI18n();
  const [typed, setTyped] = useState<string | null>(null);

  return (
    <div className="space-y-1">
      <Label htmlFor={id} className="text-xs">
        {t(`flowRun.options.${field}`)}
      </Label>
      <Input
        id={id}
        type="number"
        inputMode="numeric"
        min={runOptionLimits[field].min}
        max={runOptionLimits[field].max}
        value={typed ?? value}
        onChange={(event) => {
          const next = event.currentTarget.valueAsNumber;

          setTyped(event.currentTarget.value);
          if (Number.isFinite(next) && clampRunOption(field, next) === next) {
            onCommit(next);
          }
        }}
        onBlur={(event) => {
          if (typed !== null) {
            onCommit(clampRunOption(field, event.currentTarget.valueAsNumber));
            setTyped(null);
          }
        }}
      />
    </div>
  );
}

export function RunOptionsFields({ value, onChange, disabled = false }: RunOptionsFieldsProps) {
  const { t } = useI18n();
  const id = useId();

  return (
    <fieldset className="grid gap-3 sm:grid-cols-2" disabled={disabled}>
      <legend className="sr-only">{t("flowRun.options.label")}</legend>
      {fields.map((field) => (
        <OptionField
          key={field}
          id={`${id}-${field}`}
          field={field}
          value={value[field]}
          onCommit={(next) => onChange({ ...value, [field]: next })}
        />
      ))}
    </fieldset>
  );
}
