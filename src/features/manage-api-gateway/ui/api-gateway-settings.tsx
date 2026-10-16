import { Webhook } from "lucide-react";
import { useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import { type ApiGatewayState, ApiGatewayStatus, type UiText } from "@/shared/proto";
import { Button, Input, Label, SectionCard } from "@/shared/ui";

import { useApiGateway } from "../model/use-api-gateway";
import { ApiTokenList } from "./api-token-list";

const DEFAULT_PORT = 47831;

function StatusLine({ state }: { state: ApiGatewayState }) {
  const { t } = useI18n();

  switch (state.status) {
    case ApiGatewayStatus.RUNNING:
      return (
        <p role="status" className="text-sm text-emerald-600 dark:text-emerald-400">
          {t("manageApiGateway.status.running", { address: state.address })}
        </p>
      );
    case ApiGatewayStatus.FAILED:
      return (
        <p role="status" className="break-words text-sm text-destructive">
          {t("manageApiGateway.status.failed")}
          {state.problem ? ` · ${textValue(state.problem, true, t)}` : ""}
        </p>
      );
    default:
      return (
        <p role="status" className="text-sm text-muted-foreground">
          {t("manageApiGateway.status.stopped")}
        </p>
      );
  }
}

export function ApiGatewaySettings() {
  const { t } = useI18n();
  const id = useId();
  const { state, update, createToken, revokeToken, pending } = useApiGateway();
  const [edits, setEdits] = useState<{ enabled?: boolean; port?: string }>({});
  const [refusal, setRefusal] = useState<UiText | string | undefined>();
  const enabled = edits.enabled ?? state?.config?.enabled ?? false;
  const port = edits.port ?? (state?.config?.port ? String(state.config.port) : "");

  const apply = async () => {
    try {
      const result = await update({ enabled, port: Number(port) || 0 });

      setRefusal(result.accepted ? undefined : result.detail);
      if (result.accepted) {
        setEdits({});
      }
    } catch (error) {
      setRefusal(errorMessage(error));
    }
  };

  return (
    <SectionCard
      icon={Webhook}
      title={t("manageApiGateway.title")}
      description={t("manageApiGateway.description")}
      contentClassName="space-y-5"
    >
      {state ? <StatusLine state={state} /> : null}

      <form
        className="flex flex-wrap items-end gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          void apply();
        }}
      >
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(event) => setEdits({ ...edits, enabled: event.currentTarget.checked })}
          />
          {t("manageApiGateway.enable")}
        </label>
        <div className="space-y-1">
          <Label htmlFor={`${id}-port`}>{t("manageApiGateway.port")}</Label>
          <Input
            id={`${id}-port`}
            className="w-32"
            inputMode="numeric"
            placeholder={String(DEFAULT_PORT)}
            value={port}
            onChange={(event) =>
              setEdits({ ...edits, port: event.currentTarget.value.replace(/\D/g, "") })
            }
          />
        </div>
        <Button type="submit" disabled={pending}>
          {t("manageApiGateway.apply")}
        </Button>
      </form>
      {refusal ? (
        <p role="alert" className="break-words text-xs text-destructive">
          {typeof refusal === "string" ? refusal : textValue(refusal, true, t)}
        </p>
      ) : null}

      <section className="space-y-2">
        <h3 className="text-sm font-medium">{t("manageApiGateway.tokens.title")}</h3>
        <p className="text-xs text-muted-foreground">{t("manageApiGateway.tokens.description")}</p>
        <ApiTokenList
          tokens={state?.tokens ?? []}
          createToken={createToken}
          revokeToken={revokeToken}
          pending={pending}
        />
      </section>

      <section className="space-y-2">
        <h3 className="text-sm font-medium">{t("manageApiGateway.usage.title")}</h3>
        <pre
          className={cn(
            "overflow-x-auto rounded bg-muted p-3 font-mono text-xs",
            !state?.address && "text-muted-foreground",
          )}
        >
          {`curl -H "Authorization: Bearer <token>" -H "Content-Type: application/json" -d '{}' ${state?.address || `http://127.0.0.1:${port || DEFAULT_PORT}`}/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot`}
        </pre>
      </section>

      {state && state.authenticators.length > 0 ? (
        <section className="space-y-2">
          <h3 className="text-sm font-medium">{t("manageApiGateway.authenticators")}</h3>
          <ul className="space-y-2">
            {state.authenticators.map((authenticator) => (
              <li key={authenticator.id} className="text-xs leading-5">
                <span className="font-medium">
                  {textValue(authenticator.label, true, t) || authenticator.id}
                </span>
                {authenticator.description
                  ? ` · ${textValue(authenticator.description, true, t)}`
                  : ""}
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </SectionCard>
  );
}
