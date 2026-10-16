import { useId, useState } from "react";

import { type DeviceGroup, operatorGroups, UNGROUPED_GROUP_ID } from "@/entities/group";
import { useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { MAX_UINT32 } from "@/shared/proto";
import { Button, Input, Label, Select } from "@/shared/ui";

import { useOrganizeOutcome } from "../model/outcome";
import { type DeviceProfilePatch, useOrganizeDevices } from "../model/use-organize-devices";

type OrganizeSelectionBarProps = {
  deviceIds: readonly string[];
  groups: readonly DeviceGroup[];
};

export function OrganizeSelectionBar({ deviceIds, groups }: OrganizeSelectionBarProps) {
  const { t } = useI18n();
  const id = useId();
  const { update, pending } = useOrganizeDevices();
  const { problem, track } = useOrganizeOutcome();
  const [tag, setTag] = useState("");
  const [start, setStart] = useState("1");
  const [done, setDone] = useState<string | null>(null);

  if (deviceIds.length === 0) {
    return null;
  }

  const count = String(deviceIds.length);
  const first = Number(start);
  const startValid =
    Number.isInteger(first) && first > 0 && first + deviceIds.length - 1 <= MAX_UINT32;

  const run = async (patch: DeviceProfilePatch, message: string) => {
    setDone(null);

    if (await track(update(deviceIds, patch))) {
      setDone(message);
    }
  };

  return (
    <section
      aria-label={t("organizeDevices.bulk.title", { count })}
      className="flex flex-wrap items-end gap-3 rounded-md border border-border bg-muted/30 p-3"
    >
      <p className="self-center text-sm font-medium">
        {t("organizeDevices.bulk.title", { count })}
      </p>

      <div className="space-y-1">
        <Label htmlFor={`${id}-group`}>{t("organizeDevices.bulk.moveTo")}</Label>
        <Select
          id={`${id}-group`}
          value=""
          disabled={pending}
          onChange={(event) => {
            const target = event.currentTarget.value;

            if (target) {
              void run(
                { groupId: target === UNGROUPED_GROUP_ID ? "" : target },
                t("organizeDevices.bulk.moved"),
              );
            }
          }}
        >
          <option value="">{t("organizeDevices.bulk.pickGroup")}</option>
          {operatorGroups(groups).map((group) => (
            <option key={group.id} value={group.id}>
              {textValue(group.name, false, t)}
            </option>
          ))}
          <option value={UNGROUPED_GROUP_ID}>{t("organizeDevices.bulk.ungroup")}</option>
        </Select>
      </div>

      <div className="space-y-1">
        <Label htmlFor={`${id}-tag`}>{t("organizeDevices.bulk.tag")}</Label>
        <div className="flex gap-2">
          <Input
            id={`${id}-tag`}
            className="w-48"
            value={tag}
            onChange={(event) => setTag(event.currentTarget.value)}
            placeholder={t("organizeDevices.bulk.tagPlaceholder")}
          />
          <Button
            type="button"
            variant="outline"
            disabled={pending || !tag.trim()}
            onClick={() => void run({ addTags: [tag.trim()] }, t("organizeDevices.bulk.tagged"))}
          >
            {t("organizeDevices.bulk.addTag")}
          </Button>
          <Button
            type="button"
            variant="ghost"
            disabled={pending || !tag.trim()}
            onClick={() => void run({ removeTags: [tag.trim()] }, t("organizeDevices.bulk.tagged"))}
          >
            {t("organizeDevices.bulk.removeTag")}
          </Button>
        </div>
      </div>

      <div className="space-y-1">
        <Label htmlFor={`${id}-number`}>{t("organizeDevices.bulk.numberFrom")}</Label>
        <div className="flex gap-2">
          <Input
            id={`${id}-number`}
            className="w-24"
            inputMode="numeric"
            value={start}
            onChange={(event) => setStart(event.currentTarget.value)}
            aria-invalid={!startValid || undefined}
          />
          <Button
            type="button"
            variant="outline"
            disabled={pending || !startValid}
            onClick={() => void run({ number: first }, t("organizeDevices.bulk.numbered"))}
          >
            {t("organizeDevices.bulk.number")}
          </Button>
        </div>
      </div>

      {done ? (
        <span role="status" className="self-center text-xs text-muted-foreground">
          {done}
        </span>
      ) : null}
      {problem ? <p className="basis-full text-xs text-destructive">{problem}</p> : null}
    </section>
  );
}
