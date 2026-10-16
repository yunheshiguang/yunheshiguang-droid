import { timestampDate } from "@bufbuild/protobuf/wkt";
import { Copy, KeyRound, Trash2 } from "lucide-react";
import { useId, useMemo, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { ApiGatewayResult, ApiToken, UiText } from "@/shared/proto";
import { Button, ConfirmButton, Input, Label } from "@/shared/ui";

type ApiTokenListProps = {
  tokens: readonly ApiToken[];
  createToken: (name: string) => Promise<ApiGatewayResult>;
  revokeToken: (id: string) => Promise<ApiGatewayResult>;
  pending: boolean;
};

export function ApiTokenList({ tokens, createToken, revokeToken, pending }: ApiTokenListProps) {
  const { t, locale } = useI18n();
  const id = useId();
  const [name, setName] = useState("");
  const [secret, setSecret] = useState("");
  const [refusal, setRefusal] = useState<UiText | string | undefined>();
  const timeFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "short" }),
    [locale],
  );

  const create = async () => {
    try {
      const result = await createToken(name);

      setRefusal(result.accepted ? undefined : result.detail);
      if (result.accepted) {
        setSecret(result.secret);
        setName("");
      }
    } catch (error) {
      setRefusal(errorMessage(error));
    }
  };

  const revoke = async (tokenId: string) => {
    try {
      const result = await revokeToken(tokenId);

      setRefusal(result.accepted ? undefined : result.detail);
    } catch (error) {
      setRefusal(errorMessage(error));
    }
  };

  return (
    <div className="space-y-3">
      <form
        className="flex flex-wrap items-end gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          void create();
        }}
      >
        <div className="min-w-0 flex-1 space-y-1">
          <Label htmlFor={`${id}-name`}>{t("manageApiGateway.tokens.name")}</Label>
          <Input
            id={`${id}-name`}
            value={name}
            maxLength={32}
            onChange={(event) => setName(event.currentTarget.value)}
          />
        </div>
        <Button type="submit" disabled={pending || !name.trim()}>
          <KeyRound className="h-4 w-4" />
          {t("manageApiGateway.tokens.create")}
        </Button>
      </form>
      {refusal ? (
        <p role="alert" className="text-xs text-destructive">
          {typeof refusal === "string" ? refusal : textValue(refusal, true, t)}
        </p>
      ) : null}

      {secret ? (
        <div className="space-y-2 rounded-md border border-amber-500/40 bg-amber-500/10 p-3">
          <p className="text-xs font-medium">{t("manageApiGateway.tokens.secretOnce")}</p>
          <div className="flex gap-2">
            <Input
              readOnly
              aria-label={t("manageApiGateway.tokens.secret")}
              className="font-mono text-xs"
              value={secret}
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              aria-label={t("manageApiGateway.tokens.copy")}
              onClick={() => void navigator.clipboard?.writeText(secret)}
            >
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>
      ) : null}

      {tokens.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("manageApiGateway.tokens.empty")}</p>
      ) : (
        <ul className="space-y-2">
          {tokens.map((token) => (
            <li
              key={token.id}
              className="flex items-center justify-between gap-3 rounded-md border border-border p-3"
            >
              <div className="min-w-0">
                <div className="truncate text-sm font-medium">{token.name}</div>
                <p className="text-xs text-muted-foreground">
                  {token.createdAt === undefined
                    ? ""
                    : t("manageApiGateway.tokens.created", {
                        time: timeFormat.format(timestampDate(token.createdAt)),
                      })}
                  {" · "}
                  {token.lastUsedAt === undefined
                    ? t("manageApiGateway.tokens.neverUsed")
                    : t("manageApiGateway.tokens.lastUsed", {
                        time: timeFormat.format(timestampDate(token.lastUsedAt)),
                      })}
                </p>
              </div>
              <ConfirmButton
                type="button"
                variant="ghost"
                size="icon"
                disabled={pending}
                aria-label={t("manageApiGateway.tokens.revoke", { name: token.name })}
                title={t("manageApiGateway.tokens.confirmRevoke", { name: token.name })}
                description={t("manageApiGateway.tokens.confirmRevokeBody")}
                confirmLabel={t("manageApiGateway.tokens.revokeAction")}
                cancelLabel={t("common.cancel")}
                onConfirm={() => void revoke(token.id)}
              >
                <Trash2 className="h-4 w-4" />
              </ConfirmButton>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
