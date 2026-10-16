import { Pencil } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import type { DeviceRecord } from "@/entities/device";
import { type DeviceGroup, operatorGroups, UNGROUPED_GROUP_ID } from "@/entities/group";
import { useI18n } from "@/shared/i18n";
import { parseTagList } from "@/shared/lib/text";
import { textValue } from "@/shared/lib/workspace-view";
import { MAX_UINT32 } from "@/shared/proto";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Select,
  Textarea,
} from "@/shared/ui";

import { useOrganizeOutcome } from "../model/outcome";
import { useOrganizeDevices } from "../model/use-organize-devices";

type DeviceProfileDialogProps = {
  device: DeviceRecord | null;
  groups: readonly DeviceGroup[];
  onClose: () => void;
};

type ProfileFormProps = {
  device: DeviceRecord;
  groups: readonly DeviceGroup[];
  onDone: () => void;
};

function ProfileForm({ device, groups, onDone }: ProfileFormProps) {
  const { t } = useI18n();
  const id = useId();
  const { update, pending } = useOrganizeDevices();
  const { problem, track } = useOrganizeOutcome();
  const labels = device.labels;
  const [number, setNumber] = useState(labels?.number ? String(labels.number) : "");
  const [alias, setAlias] = useState(labels?.alias ?? "");
  const [note, setNote] = useState(labels?.note?.displayValue ?? "");
  const [tags, setTags] = useState((labels?.tags ?? []).join(", "));
  const [groupId, setGroupId] = useState(
    device.groupId === UNGROUPED_GROUP_ID ? "" : device.groupId,
  );
  const numberValue = number.trim() === "" ? 0 : Number(number);
  const numberInvalid =
    !Number.isInteger(numberValue) || numberValue < 0 || numberValue > MAX_UINT32;

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (numberInvalid) {
      return;
    }

    const nextTags = parseTagList(tags);
    const kept = (tag: string) => nextTags.some((next) => next.toLowerCase() === tag.toLowerCase());
    const done = await track(
      update([device.id], {
        number: numberValue,
        alias,
        note,
        groupId,
        addTags: nextTags,
        removeTags: (labels?.tags ?? []).filter((tag) => !kept(tag)),
      }),
    );

    if (done) {
      onDone();
    }
  };

  return (
    <form onSubmit={submit} className="space-y-3" aria-label={t("organizeDevices.profile.title")}>
      <div className="grid gap-3 sm:grid-cols-[8rem_1fr]">
        <div className="space-y-1">
          <Label htmlFor={`${id}-number`}>{t("organizeDevices.profile.number")}</Label>
          <Input
            id={`${id}-number`}
            inputMode="numeric"
            value={number}
            onChange={(event) => setNumber(event.currentTarget.value)}
            aria-invalid={numberInvalid || undefined}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-alias`}>{t("organizeDevices.profile.alias")}</Label>
          <Input
            id={`${id}-alias`}
            value={alias}
            onChange={(event) => setAlias(event.currentTarget.value)}
            placeholder={t("organizeDevices.profile.aliasPlaceholder")}
          />
        </div>
      </div>
      {numberInvalid ? (
        <p className="text-xs text-destructive">{t("organizeDevices.profile.numberInvalid")}</p>
      ) : null}
      <div className="space-y-1">
        <Label htmlFor={`${id}-group`}>{t("organizeDevices.profile.group")}</Label>
        <Select
          id={`${id}-group`}
          value={groupId}
          onChange={(event) => setGroupId(event.currentTarget.value)}
        >
          <option value="">{t("organizeDevices.profile.noGroup")}</option>
          {operatorGroups(groups).map((group) => (
            <option key={group.id} value={group.id}>
              {textValue(group.name, false, t)}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${id}-tags`}>{t("organizeDevices.profile.tags")}</Label>
        <Input
          id={`${id}-tags`}
          value={tags}
          onChange={(event) => setTags(event.currentTarget.value)}
          placeholder={t("organizeDevices.profile.tagsPlaceholder")}
        />
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${id}-note`}>{t("organizeDevices.profile.note")}</Label>
        <Textarea
          id={`${id}-note`}
          value={note}
          onChange={(event) => setNote(event.currentTarget.value)}
          placeholder={t("organizeDevices.profile.notePlaceholder")}
        />
      </div>
      {problem ? <p className="text-xs text-destructive">{problem}</p> : null}
      <DialogFooter>
        <Button type="submit" disabled={pending || numberInvalid}>
          {pending ? t("organizeDevices.profile.saving") : t("organizeDevices.profile.save")}
        </Button>
      </DialogFooter>
    </form>
  );
}

export function DeviceProfileButton({ title, onOpen }: { title: string; onOpen: () => void }) {
  const { t } = useI18n();

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      aria-label={t("organizeDevices.profile.open", { device: title })}
      onClick={onOpen}
    >
      <Pencil className="h-4 w-4" />
    </Button>
  );
}

export function DeviceProfileDialog({ device, groups, onClose }: DeviceProfileDialogProps) {
  const { t } = useI18n();

  return (
    <Dialog
      open={device !== null}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent closeLabel={t("organizeDevices.close")}>
        <DialogHeader>
          <DialogTitle>{t("organizeDevices.profile.title")}</DialogTitle>
          <DialogDescription>{t("organizeDevices.profile.description")}</DialogDescription>
        </DialogHeader>
        {device ? (
          <ProfileForm key={device.id} device={device} groups={groups} onDone={onClose} />
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
