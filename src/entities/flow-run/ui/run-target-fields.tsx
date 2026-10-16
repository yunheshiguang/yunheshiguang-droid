import { useId } from "react";

import { useI18n } from "@/shared/i18n";
import { groupLabel, textValue } from "@/shared/lib/workspace-view";
import type { DeviceGroup } from "@/shared/proto";
import { Label, Select } from "@/shared/ui";

import type { RunTargetDraft, RunTargetMode } from "../model/run-target";

type RunTargetFieldsProps = {
  value: RunTargetDraft;
  onChange: (value: RunTargetDraft) => void;
  groups: readonly Pick<DeviceGroup, "id" | "name">[];
  tags: readonly string[];
  devicesHint: string;
  disabled?: boolean;
};

const modes: readonly RunTargetMode[] = ["devices", "group", "tag"];

export function RunTargetFields({
  value,
  onChange,
  groups,
  tags,
  devicesHint,
  disabled = false,
}: RunTargetFieldsProps) {
  const { t } = useI18n();
  const id = useId();

  return (
    <div className="space-y-2">
      <Label htmlFor={`${id}-mode`}>{t("flowRun.target.label")}</Label>
      <Select
        id={`${id}-mode`}
        value={value.mode}
        disabled={disabled}
        onChange={(event) =>
          onChange({ ...value, mode: event.currentTarget.value as RunTargetMode })
        }
      >
        {modes.map((mode) => (
          <option key={mode} value={mode}>
            {t(`flowRun.target.modes.${mode}`)}
          </option>
        ))}
      </Select>

      {value.mode === "devices" ? (
        <p className="text-xs text-muted-foreground">{devicesHint}</p>
      ) : null}

      {value.mode === "group" ? (
        <Select
          aria-label={t("flowRun.target.group")}
          value={value.groupId}
          disabled={disabled}
          onChange={(event) => onChange({ ...value, groupId: event.currentTarget.value })}
        >
          <option value="">{t("flowRun.target.pickGroup")}</option>
          {value.groupId && !groups.some((group) => group.id === value.groupId) ? (
            <option value={value.groupId}>
              {groupLabel(groups, value.groupId, t) ?? t("flowRun.target.missingGroup")}
            </option>
          ) : null}
          {groups.map((group) => (
            <option key={group.id} value={group.id}>
              {textValue(group.name, true, t) || group.id}
            </option>
          ))}
        </Select>
      ) : null}

      {value.mode === "tag" ? (
        tags.length === 0 && !value.tag ? (
          <p className="text-xs text-muted-foreground">{t("flowRun.target.noTags")}</p>
        ) : (
          <Select
            aria-label={t("flowRun.target.tag")}
            value={value.tag}
            disabled={disabled}
            onChange={(event) => onChange({ ...value, tag: event.currentTarget.value })}
          >
            <option value="">{t("flowRun.target.pickTag")}</option>
            {value.tag && !tags.includes(value.tag) ? (
              <option value={value.tag}>{value.tag}</option>
            ) : null}
            {tags.map((tag) => (
              <option key={tag} value={tag}>
                {tag}
              </option>
            ))}
          </Select>
        )
      ) : null}

      {value.mode !== "devices" ? (
        <p className="text-xs text-muted-foreground">{t("flowRun.target.onlineOnly")}</p>
      ) : null}
    </div>
  );
}
