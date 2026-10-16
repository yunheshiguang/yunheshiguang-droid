import { Radar } from "lucide-react";
import { useId, useState } from "react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import type { ConnectionPlan } from "@/shared/proto";
import { ConnectionType, type DiscoveredEndpoint, enumKey } from "@/shared/proto";
import { Button, ErrorState, Input, Label, SectionCard } from "@/shared/ui";

import { useConnectDevice, usePairDevice } from "../model/use-connection-commands";
import { useDeviceDiscovery } from "../model/use-device-discovery";
import { ConnectionOutcome } from "./connection-outcome";

type DiscoveredDeviceListProps = {
  transports: readonly ConnectionPlan[];
};

function ConnectAction({ endpoint, name }: { endpoint: DiscoveredEndpoint; name: string }) {
  const { t } = useI18n();
  const connect = useConnectDevice();

  return (
    <div className="space-y-2">
      <Button
        size="sm"
        variant="outline"
        disabled={connect.pending}
        onClick={() => connect.run({ endpoint: endpoint.endpoint })}
        aria-label={t("connectDevice.discovery.connectNamed", { name })}
      >
        {t("connectDevice.discovery.connect")}
      </Button>
      <ConnectionOutcome result={connect.result} error={connect.error} />
    </div>
  );
}

function PairAction({ endpoint, name }: { endpoint: DiscoveredEndpoint; name: string }) {
  const { t } = useI18n();
  const id = useId();
  const pair = usePairDevice();
  const [code, setCode] = useState("");

  return (
    <form
      className="space-y-2"
      onSubmit={(event) => {
        event.preventDefault();
        pair.run({ endpoint: endpoint.endpoint, pairingCode: code.trim() });
      }}
    >
      <div className="flex items-end gap-2">
        <div className="space-y-1">
          <Label htmlFor={id} className="text-xs">
            {t("connectDevice.discovery.code")}
          </Label>
          <Input
            id={id}
            value={code}
            onChange={(event) => setCode(event.currentTarget.value)}
            inputMode="numeric"
            autoComplete="one-time-code"
            className="h-8 w-28"
          />
        </div>
        <Button
          type="submit"
          size="sm"
          disabled={pair.pending || code.trim().length === 0}
          aria-label={t("connectDevice.discovery.pairNamed", { name })}
        >
          {t("connectDevice.discovery.pair")}
        </Button>
      </div>
      <ConnectionOutcome result={pair.result} error={pair.error} />
    </form>
  );
}

export function DiscoveredDeviceList({ transports }: DiscoveredDeviceListProps) {
  const { t } = useI18n();
  const { discovery, loading, error } = useDeviceDiscovery();
  const titles = new Map(transports.map((plan) => [plan.id, textValue(plan.title, true, t)]));

  return (
    <SectionCard
      icon={Radar}
      title={t("connectDevice.discovery.title")}
      contentClassName="space-y-3"
    >
      {error ? (
        <ErrorState title={t("connectDevice.discovery.failed")} detail={error} />
      ) : loading ? (
        <p className="text-sm text-muted-foreground">{t("connectDevice.discovery.loading")}</p>
      ) : discovery.endpoints.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("connectDevice.discovery.empty")}</p>
      ) : (
        <ul className="space-y-2">
          {discovery.endpoints.map((endpoint) => {
            const name = textValue(endpoint.label, false, t);

            return (
              <li
                key={`${endpoint.transportId}/${endpoint.endpoint?.host}:${endpoint.endpoint?.port}`}
                className="flex flex-wrap items-start justify-between gap-3 rounded-md border border-border bg-background p-3"
              >
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{name}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {titles.get(endpoint.transportId) ??
                      t(
                        runtimeKey(
                          `enums.connectionType.${enumKey(ConnectionType, endpoint.connection)}`,
                        ),
                      )}
                    {endpoint.requiresPairing
                      ? ` · ${t("connectDevice.discovery.needsPairing")}`
                      : ""}
                  </div>
                </div>
                {endpoint.requiresPairing ? (
                  <PairAction endpoint={endpoint} name={name} />
                ) : (
                  <ConnectAction endpoint={endpoint} name={name} />
                )}
              </li>
            );
          })}
        </ul>
      )}
      {discovery.failures.map((failure) => (
        <p key={failure.transportId} className="text-xs leading-5 text-destructive">
          {titles.get(failure.transportId) ?? failure.transportId}:{" "}
          {textValue(failure.reason, true, t)}
        </p>
      ))}
    </SectionCard>
  );
}
