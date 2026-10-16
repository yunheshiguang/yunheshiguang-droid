import { dialogGateway } from "@/shared/api";
import { useI18n } from "@/shared/i18n";
import type { BatchActionKind } from "@/shared/proto";
import { Input, PathInput, Textarea } from "@/shared/ui";

import {
  batchActionDescriptors,
  batchActionInputPlaceholderKeys,
  pathPickerFor,
} from "../model/batch-actions";

type BatchActionValueInputProps = {
  kind: BatchActionKind;
  value: string;
  onValueChange: (value: string) => void;
  id?: string;
};

export function BatchActionValueInput({
  kind,
  value,
  onValueChange,
  id,
}: BatchActionValueInputProps) {
  const { t } = useI18n();
  const descriptor = batchActionDescriptors.find((candidate) => candidate.kind === kind);
  const input = descriptor?.input ?? "none";

  if (!descriptor || input === "none" || input === "script") {
    return null;
  }

  const placeholder = t(batchActionInputPlaceholderKeys[input]);
  const label = id ? undefined : placeholder;
  const picker = pathPickerFor(descriptor);

  if (input === "text") {
    return (
      <Textarea
        id={id}
        aria-label={label}
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder={placeholder}
      />
    );
  }

  if (picker) {
    return (
      <PathInput
        id={id}
        value={value}
        onValueChange={onValueChange}
        onBrowse={() => dialogGateway().pickPath(picker)}
        browseLabel={t("common.browse")}
        placeholder={placeholder}
        label={label}
      />
    );
  }

  return (
    <Input
      id={id}
      aria-label={label}
      value={value}
      onChange={(event) => onValueChange(event.target.value)}
      placeholder={placeholder}
    />
  );
}
