import { FolderCog, Plus, Trash2 } from "lucide-react";
import { useState } from "react";

import { type DeviceGroup, groupTones, operatorGroups } from "@/entities/group";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { groupToneClass, textValue } from "@/shared/lib/workspace-view";
import { GroupTone, type OrganizeResult } from "@/shared/proto";
import {
  Button,
  ConfirmButton,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Input,
  Select,
} from "@/shared/ui";

import { useOrganizeOutcome } from "../model/outcome";
import { useOrganizeDevices } from "../model/use-organize-devices";

type Track = (operation: Promise<OrganizeResult>) => Promise<boolean>;

type Tone = (typeof groupTones)[number];

function ToneSelect({
  value,
  label,
  onChange,
}: {
  value: Tone;
  label: string;
  onChange: (tone: Tone) => void;
}) {
  const { t } = useI18n();

  return (
    <Select
      className="w-28"
      value={value}
      aria-label={label}
      onChange={(event) => onChange(event.currentTarget.value as Tone)}
    >
      {groupTones.map((tone) => (
        <option key={tone} value={tone}>
          {t(`enums.groupTone.${tone}`)}
        </option>
      ))}
    </Select>
  );
}

function GroupRow({ group, track }: { group: DeviceGroup; track: Track }) {
  const { t } = useI18n();
  const { saveGroup, removeGroup, pending } = useOrganizeDevices();
  const current = textValue(group.name, false, t);
  const initialTone: Tone = groupTones.find((name) => GroupTone[name] === group.tone) ?? "EMERALD";
  const [name, setName] = useState(current);
  const [tone, setTone] = useState(initialTone);
  const changed = name.trim() !== current || tone !== initialTone;

  return (
    <li className="flex items-center gap-2">
      <span aria-hidden className={cn("h-2.5 w-2.5 shrink-0 rounded-full", groupToneClass(tone))} />
      <Input
        value={name}
        aria-label={t("organizeDevices.groups.name")}
        onChange={(event) => setName(event.currentTarget.value)}
      />
      <ToneSelect value={tone} label={t("organizeDevices.groups.tone")} onChange={setTone} />
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={pending || !changed || !name.trim()}
        onClick={() =>
          void track(saveGroup({ id: group.id, name: name.trim(), tone: GroupTone[tone] }))
        }
      >
        {t("organizeDevices.groups.rename")}
      </Button>
      <ConfirmButton
        type="button"
        variant="ghost"
        size="icon"
        disabled={pending}
        aria-label={t("organizeDevices.groups.remove", { name: current })}
        title={t("organizeDevices.groups.confirmRemove", { name: current })}
        description={t("organizeDevices.groups.description")}
        confirmLabel={t("organizeDevices.groups.delete")}
        cancelLabel={t("common.cancel")}
        onConfirm={() => void track(removeGroup(group.id))}
      >
        <Trash2 className="h-4 w-4" />
      </ConfirmButton>
    </li>
  );
}

function GroupList({ groups }: { groups: readonly DeviceGroup[] }) {
  const { t } = useI18n();
  const { saveGroup, pending } = useOrganizeDevices();
  const { problem, track } = useOrganizeOutcome();
  const [name, setName] = useState("");
  const [tone, setTone] = useState<Tone>("EMERALD");

  const create = async () => {
    if (
      await track(saveGroup({ id: crypto.randomUUID(), name: name.trim(), tone: GroupTone[tone] }))
    ) {
      setName("");
    }
  };

  return (
    <div className="space-y-4">
      {groups.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("organizeDevices.groups.empty")}</p>
      ) : (
        <ul aria-label={t("organizeDevices.groups.title")} className="space-y-2">
          {groups.map((group) => (
            <GroupRow key={group.id} group={group} track={track} />
          ))}
        </ul>
      )}
      <form
        className="flex items-center gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          void create();
        }}
      >
        <Input
          value={name}
          aria-label={t("organizeDevices.groups.namePlaceholder")}
          placeholder={t("organizeDevices.groups.namePlaceholder")}
          onChange={(event) => setName(event.currentTarget.value)}
        />
        <ToneSelect value={tone} label={t("organizeDevices.groups.tone")} onChange={setTone} />
        <Button type="submit" size="sm" disabled={pending || !name.trim()}>
          <Plus className="h-4 w-4" />
          {t("organizeDevices.groups.create")}
        </Button>
      </form>
      {problem ? <p className="text-xs text-destructive">{problem}</p> : null}
    </div>
  );
}

export function GroupManager({ groups }: { groups: readonly DeviceGroup[] }) {
  const { t } = useI18n();

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button type="button" variant="outline" size="sm">
          <FolderCog className="h-4 w-4" />
          {t("organizeDevices.groups.manage")}
        </Button>
      </DialogTrigger>
      <DialogContent closeLabel={t("organizeDevices.close")}>
        <DialogHeader>
          <DialogTitle>{t("organizeDevices.groups.title")}</DialogTitle>
          <DialogDescription>{t("organizeDevices.groups.description")}</DialogDescription>
        </DialogHeader>
        <GroupList groups={operatorGroups(groups)} />
      </DialogContent>
    </Dialog>
  );
}
