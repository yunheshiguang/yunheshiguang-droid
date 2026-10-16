import { Globe, Radio, TerminalSquare } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { SectionCard } from "@/shared/ui";

import { httpEndpoints, procedures, pushedEvents, windowProcedures } from "../model/http-endpoints";

export function ExternalApiPage() {
  const { t } = useI18n();

  return (
    <div className="space-y-4 p-6">
      <SectionCard
        icon={TerminalSquare}
        title={t("externalApi.title")}
        body={t("externalApi.body")}
      />

      <SectionCard
        icon={Globe}
        title={t("externalApi.http.title")}
        description={t("externalApi.http.description")}
        contentClassName="space-y-2"
      >
        <ul className="space-y-2">
          {httpEndpoints.map((endpoint) => (
            <li key={endpoint.id} className="rounded-md border border-border p-3">
              <code className="font-mono text-xs font-medium">
                {endpoint.method} {endpoint.path}
              </code>
              <p className="mt-1 text-xs text-muted-foreground">
                {t(`externalApi.endpoints.${endpoint.id}`)}
              </p>
            </li>
          ))}
          {procedures.map((procedure) => (
            <li key={procedure.path} className="rounded-md border border-border p-3">
              <code className="break-all font-mono text-xs font-medium">
                {procedure.read ? "GET · POST" : "POST"} {procedure.path}
              </code>
              <p className="mt-1 text-xs text-muted-foreground">
                {t(runtimeKey(`externalApi.commands.${procedure.id}`))}
              </p>
            </li>
          ))}
        </ul>
      </SectionCard>

      <SectionCard
        icon={Radio}
        title={t("externalApi.stream.title")}
        description={t("externalApi.stream.description")}
        contentClassName="space-y-2"
      >
        <ul className="space-y-2">
          {pushedEvents.map((event) => (
            <li key={event.name} className="rounded-md border border-border p-3">
              <code className="font-mono text-xs font-medium">{event.name}</code>
              <p className="mt-1 text-xs text-muted-foreground">
                {t(`externalApi.events.${event.id}`)}
              </p>
            </li>
          ))}
        </ul>
      </SectionCard>

      <h2 className="text-sm font-medium">{t("externalApi.commandsTitle")}</h2>
      <div className="grid gap-3 md:grid-cols-2">
        {windowProcedures.map((procedure) => (
          <SectionCard
            key={procedure.path}
            title={procedure.name}
            description={t(runtimeKey(`externalApi.commands.${procedure.id}`))}
          />
        ))}
      </div>
    </div>
  );
}
