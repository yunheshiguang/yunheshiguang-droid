import { useI18n } from "@/shared/i18n";
import { enumKey, ScriptLanguage } from "@/shared/proto";
import { Select } from "@/shared/ui";

import { useScripts } from "../model/script-documents";

type ScriptPickerProps = {
  value: string;
  onValueChange: (id: string) => void;
  id?: string;
};

export function ScriptPicker({ value, onValueChange, id }: ScriptPickerProps) {
  const { t } = useI18n();
  const scripts = useScripts();

  if (scripts.length === 0) {
    return <p className="text-xs text-muted-foreground">{t("script.picker.empty")}</p>;
  }

  return (
    <Select
      id={id}
      aria-label={t("script.picker.label")}
      value={scripts.some((script) => script.id === value) ? value : ""}
      onChange={(event) => onValueChange(event.currentTarget.value)}
    >
      <option value="">{t("script.picker.placeholder")}</option>
      {scripts.map((script) => (
        <option key={script.id} value={script.id}>
          {script.name || t("script.untitled")} ·{" "}
          {t(`enums.scriptLanguage.${enumKey(ScriptLanguage, script.language)}`)}
        </option>
      ))}
    </Select>
  );
}
