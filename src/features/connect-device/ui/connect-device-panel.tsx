import { Wifi } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { Button, Input, Label, SectionCard } from "@/shared/ui";

import { DEFAULT_DEBUG_PORT, parseEndpointInput } from "../model/endpoint-input";
import { useConnectDevice, usePairDevice } from "../model/use-connection-commands";
import { ConnectionOutcome } from "./connection-outcome";

type AddressFieldProps = {
  id: string;
  label: string;
  hint: string;
  value: string;
  onChange: (value: string) => void;
  invalid: boolean;
  placeholder: string;
};

function AddressField({
  id,
  label,
  hint,
  value,
  onChange,
  invalid,
  placeholder,
}: AddressFieldProps) {
  const { t } = useI18n();

  return (
    <div className="space-y-2">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
        placeholder={placeholder}
        autoComplete="off"
        spellCheck={false}
        aria-invalid={invalid || undefined}
        aria-describedby={`${id}-hint`}
      />
      <p id={`${id}-hint`} className="text-xs leading-5 text-muted-foreground">
        {invalid ? t("connectDevice.address.invalid") : hint}
      </p>
    </div>
  );
}

function ConnectForm() {
  const { t } = useI18n();
  const id = useId();
  const [address, setAddress] = useState("");
  const [invalid, setInvalid] = useState(false);
  const connect = useConnectDevice();

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const endpoint = parseEndpointInput(address, DEFAULT_DEBUG_PORT);

    setInvalid(!endpoint);

    if (endpoint) {
      connect.run({ endpoint });
    }
  };

  return (
    <form onSubmit={submit} aria-labelledby={`${id}-title`} className="space-y-3">
      <h3 id={`${id}-title`} className="text-sm font-medium">
        {t("connectDevice.connect.title")}
      </h3>
      <AddressField
        id={`${id}-address`}
        label={t("connectDevice.address.label")}
        hint={t("connectDevice.connect.hint")}
        value={address}
        onChange={setAddress}
        invalid={invalid}
        placeholder="192.168.1.20:5555"
      />
      <Button type="submit" disabled={connect.pending}>
        {connect.pending ? t("connectDevice.pending") : t("connectDevice.connect.submit")}
      </Button>
      <ConnectionOutcome result={connect.result} error={connect.error} />
    </form>
  );
}

function PairForm() {
  const { t } = useI18n();
  const id = useId();
  const [address, setAddress] = useState("");
  const [code, setCode] = useState("");
  const [invalid, setInvalid] = useState(false);
  const pair = usePairDevice();

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const endpoint = parseEndpointInput(address);

    setInvalid(!endpoint);

    if (endpoint) {
      pair.run({ endpoint, pairingCode: code });
    }
  };

  return (
    <form onSubmit={submit} aria-labelledby={`${id}-title`} className="space-y-3">
      <h3 id={`${id}-title`} className="text-sm font-medium">
        {t("connectDevice.pair.title")}
      </h3>
      <AddressField
        id={`${id}-address`}
        label={t("connectDevice.pair.address")}
        hint={t("connectDevice.pair.hint")}
        value={address}
        onChange={setAddress}
        invalid={invalid}
        placeholder="192.168.1.20:37123"
      />
      <div className="space-y-2">
        <Label htmlFor={`${id}-code`}>{t("connectDevice.pair.code")}</Label>
        <Input
          id={`${id}-code`}
          value={code}
          onChange={(event) => setCode(event.currentTarget.value)}
          inputMode="numeric"
          autoComplete="one-time-code"
          maxLength={6}
        />
      </div>
      <Button type="submit" disabled={pair.pending}>
        {pair.pending ? t("connectDevice.pending") : t("connectDevice.pair.submit")}
      </Button>
      <ConnectionOutcome result={pair.result} error={pair.error} />
    </form>
  );
}

export function ConnectDevicePanel() {
  const { t } = useI18n();

  return (
    <SectionCard
      icon={Wifi}
      title={t("connectDevice.title")}
      description={t("connectDevice.description")}
      contentClassName="grid gap-6 lg:grid-cols-2"
    >
      <ConnectForm />
      <PairForm />
    </SectionCard>
  );
}
