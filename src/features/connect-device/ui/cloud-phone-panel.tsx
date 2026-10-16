import { Cloud, Play, Power, Trash2 } from "lucide-react";
import { useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import {
  type CloudPhone,
  type CloudPhoneResult,
  CloudPhoneStatus,
  type CloudProviderInfo,
  enumKey,
} from "@/shared/proto";
import { Button, ConfirmButton, SectionCard } from "@/shared/ui";

import { useCloudPhones } from "../model/use-cloud-phones";
import { RegisterCloudPhoneForm } from "./register-cloud-phone-form";

const STATUS_TONES = {
  [CloudPhoneStatus.UNSPECIFIED]: "neutral",
  [CloudPhoneStatus.RUNNING]: "positive",
  [CloudPhoneStatus.STOPPED]: "neutral",
  [CloudPhoneStatus.STARTING]: "info",
  [CloudPhoneStatus.STOPPING]: "info",
  [CloudPhoneStatus.FAILED]: "critical",
} as const satisfies Record<CloudPhoneStatus, StatusTone>;

type PhoneRowProps = {
  phone: CloudPhone;
  provider?: CloudProviderInfo;
  busy: boolean;
  onAct: (action: "start" | "stop" | "remove", phone: CloudPhone) => void;
};

function PhoneRow({ phone, provider, busy, onAct }: PhoneRowProps) {
  const { t } = useI18n();
  const address = phone.endpoint ? `${phone.endpoint.host}:${phone.endpoint.port}` : undefined;

  return (
    <li className="flex items-start justify-between gap-3 rounded-md border border-border p-3">
      <div className="min-w-0 space-y-1">
        <p className="truncate text-sm font-medium">{phone.name || phone.id}</p>
        <p className="text-xs text-muted-foreground">
          {textValue(provider?.label, true, t) || phone.providerId}
          {address ? ` · ${address}` : ` · ${t("connectDevice.cloud.noAddress")}`}
        </p>
        {phone.detail ? (
          <p className="text-xs text-muted-foreground">{textValue(phone.detail, true, t)}</p>
        ) : null}
      </div>
      <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
        <span
          className={cn(
            "text-xs font-medium",
            statusTextTones[STATUS_TONES[phone.status] ?? "neutral"],
          )}
        >
          {t(`enums.cloudPhoneStatus.${enumKey(CloudPhoneStatus, phone.status)}`)}
        </span>
        {provider?.canStart && phone.status !== CloudPhoneStatus.RUNNING ? (
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => onAct("start", phone)}
          >
            <Play className="h-4 w-4" />
            {t("connectDevice.cloud.start")}
          </Button>
        ) : null}
        {provider?.canStop && phone.status === CloudPhoneStatus.RUNNING ? (
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => onAct("stop", phone)}
          >
            <Power className="h-4 w-4" />
            {t("connectDevice.cloud.stop")}
          </Button>
        ) : null}
        {provider?.canRegister ? (
          <ConfirmButton
            type="button"
            size="sm"
            variant="ghost"
            aria-label={t("connectDevice.cloud.remove", { name: phone.name || phone.id })}
            disabled={busy}
            title={t("connectDevice.cloud.confirmRemove", { name: phone.name || phone.id })}
            confirmLabel={t("connectDevice.cloud.removeAction")}
            cancelLabel={t("common.cancel")}
            onConfirm={() => onAct("remove", phone)}
          >
            <Trash2 className="h-4 w-4" />
          </ConfirmButton>
        ) : null}
      </div>
    </li>
  );
}

export function CloudPhonePanel() {
  const { t } = useI18n();
  const { phones, loading, error, start, stop, remove, register, busy } = useCloudPhones();
  const [outcome, setOutcome] = useState<{ ok: boolean; text: string } | null>(null);
  const providers = new Map((phones?.providers ?? []).map((provider) => [provider.id, provider]));
  const canRegister = phones?.providers.some((provider) => provider.canRegister) ?? false;

  const report = (result: CloudPhoneResult) =>
    setOutcome(
      result.accepted
        ? { ok: true, text: t("connectDevice.cloud.done") }
        : { ok: false, text: textValue(result.detail, true, t) },
    );
  const act = (action: "start" | "stop" | "remove", phone: CloudPhone) => {
    const run = { start, stop, remove }[action];
    void run({ providerId: phone.providerId, phoneId: phone.id })
      .then(report)
      .catch((failure: unknown) => setOutcome({ ok: false, text: errorMessage(failure) }));
  };

  return (
    <SectionCard icon={Cloud} title={t("connectDevice.cloud.title")} contentClassName="space-y-4">
      {error ? (
        <p role="alert" className="text-xs text-destructive">
          {errorMessage(error)}
        </p>
      ) : loading ? (
        <p role="status" className="text-sm text-muted-foreground">
          {t("connectDevice.cloud.loading")}
        </p>
      ) : phones && phones.phones.length > 0 ? (
        <ul aria-label={t("connectDevice.cloud.list")} className="space-y-2">
          {phones.phones.map((phone) => (
            <PhoneRow
              key={`${phone.providerId}/${phone.id}`}
              phone={phone}
              provider={providers.get(phone.providerId)}
              busy={busy}
              onAct={act}
            />
          ))}
        </ul>
      ) : (
        <p className="text-sm text-muted-foreground">{t("connectDevice.cloud.empty")}</p>
      )}
      {(phones?.problems ?? []).map((problem) => {
        const provider =
          textValue(providers.get(problem.providerId)?.label, true, t) || problem.providerId;
        const reason = textValue(problem.problem, true, t);

        return (
          <p key={problem.providerId} className="text-xs text-muted-foreground">
            {t("connectDevice.cloud.problem", { provider, reason })}
          </p>
        );
      })}
      {canRegister ? (
        <RegisterCloudPhoneForm busy={busy} onRegister={register} onResult={report} />
      ) : null}
      {outcome ? (
        <p
          role={outcome.ok ? "status" : "alert"}
          className={outcome.ok ? "text-xs text-muted-foreground" : "text-xs text-destructive"}
        >
          {outcome.text}
        </p>
      ) : null}
    </SectionCard>
  );
}
