import { Plus } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import type { CloudPhoneResult } from "@/shared/proto";
import { Button, Input, Label } from "@/shared/ui";

type RegisterCloudPhoneFormProps = {
  busy: boolean;
  onRegister: (phone: {
    name: string;
    endpoint: { host: string; port: number };
  }) => Promise<CloudPhoneResult>;
  onResult: (result: CloudPhoneResult) => void;
};

const ADB_PORT = 5555;
const MAX_PORT = 65_535;

function parsePort(text: string): number | undefined {
  const digits = text.trim();

  if (!/^\d{1,5}$/.test(digits)) {
    return undefined;
  }

  const port = Number(digits);

  return port >= 1 && port <= MAX_PORT ? port : undefined;
}

export function RegisterCloudPhoneForm({
  busy,
  onRegister,
  onResult,
}: RegisterCloudPhoneFormProps) {
  const { t } = useI18n();
  const id = useId();
  const [name, setName] = useState("");
  const [host, setHost] = useState("");
  const [port, setPort] = useState(String(ADB_PORT));
  const [failure, setFailure] = useState<string | null>(null);
  const parsedPort = parsePort(port);

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (parsedPort === undefined) {
      return;
    }

    setFailure(null);
    void onRegister({ name: name.trim(), endpoint: { host: host.trim(), port: parsedPort } })
      .then((result) => {
        onResult(result);
        if (result.accepted) {
          setName("");
          setHost("");
        }
      })
      .catch((error: unknown) => setFailure(errorMessage(error)));
  };

  return (
    <form onSubmit={submit} aria-labelledby={`${id}-title`} className="space-y-2">
      <h3 id={`${id}-title`} className="text-sm font-medium">
        {t("connectDevice.cloud.register.title")}
      </h3>
      <div className="grid gap-3 sm:grid-cols-[1fr_1fr_6rem_auto] sm:items-end">
        <div className="space-y-1">
          <Label htmlFor={`${id}-name`}>{t("connectDevice.cloud.register.name")}</Label>
          <Input
            id={`${id}-name`}
            value={name}
            onChange={(event) => setName(event.currentTarget.value)}
            placeholder={t("connectDevice.cloud.register.namePlaceholder")}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-host`}>{t("connectDevice.cloud.register.host")}</Label>
          <Input
            id={`${id}-host`}
            value={host}
            onChange={(event) => setHost(event.currentTarget.value)}
            placeholder="10.20.0.11"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor={`${id}-port`}>{t("connectDevice.cloud.register.port")}</Label>
          <Input
            id={`${id}-port`}
            inputMode="numeric"
            value={port}
            aria-invalid={parsedPort === undefined || undefined}
            aria-describedby={parsedPort === undefined ? `${id}-port-issue` : undefined}
            onChange={(event) => setPort(event.currentTarget.value)}
          />
        </div>
        <Button
          type="submit"
          size="sm"
          disabled={busy || !name.trim() || !host.trim() || parsedPort === undefined}
        >
          <Plus className="h-4 w-4" />
          {t("connectDevice.cloud.register.submit")}
        </Button>
      </div>
      {parsedPort === undefined ? (
        <p id={`${id}-port-issue`} className="text-xs text-destructive">
          {t("connectDevice.cloud.register.portInvalid")}
        </p>
      ) : null}
      {failure ? (
        <p role="alert" className="break-words text-xs text-destructive">
          {failure}
        </p>
      ) : null}
    </form>
  );
}
