import { useI18n } from "@/shared/i18n";
import { Select } from "@/shared/ui";

import { usePhrases } from "../model/phrase-documents";

type PhrasePickerProps = {
  onPick: (body: string) => void;
  id?: string;
};

export function PhrasePicker({ onPick, id }: PhrasePickerProps) {
  const { t } = useI18n();
  const phrases = usePhrases();

  if (phrases.length === 0) {
    return <p className="text-xs text-muted-foreground">{t("phrase.picker.empty")}</p>;
  }

  return (
    <Select
      id={id}
      aria-label={t("phrase.picker.label")}
      value=""
      onChange={(event) => {
        const phrase = phrases.find((candidate) => candidate.id === event.currentTarget.value);

        if (phrase) {
          onPick(phrase.body);
        }
      }}
    >
      <option value="">{t("phrase.picker.placeholder")}</option>
      {phrases.map((phrase) => (
        <option key={phrase.id} value={phrase.id}>
          {phrase.title}
        </option>
      ))}
    </Select>
  );
}
