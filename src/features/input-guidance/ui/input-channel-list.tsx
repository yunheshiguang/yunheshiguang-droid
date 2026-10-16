import { Keyboard } from "lucide-react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import { enumKey, InputKind } from "@/shared/proto";
import { Badge, ErrorState } from "@/shared/ui";

import { useInputChannels } from "../model/use-input-channels";

export function InputChannelList() {
  const { t } = useI18n();
  const { data: channels, isPending, isError, error } = useInputChannels();

  if (isPending) {
    return (
      <p role="status" className="text-xs text-muted-foreground">
        {t("inputGuidance.loading")}
      </p>
    );
  }

  if (isError) {
    return <ErrorState title={t("inputGuidance.failed")} detail={errorMessage(error)} />;
  }

  return (
    <section
      aria-label={t("inputGuidance.title")}
      className="space-y-3 rounded-md border border-border bg-background p-3"
    >
      <div>
        <h3 className="flex items-center gap-2 text-sm font-medium">
          <Keyboard className="h-4 w-4 text-muted-foreground" />
          {t("inputGuidance.title")}
        </h3>
        <p className="mt-1 text-xs leading-5 text-muted-foreground">
          {t("inputGuidance.description")}
        </p>
      </div>
      {channels.length === 0 ? (
        <p className="text-xs text-destructive">{t("inputGuidance.none")}</p>
      ) : (
        <ol className="space-y-3">
          {channels.map((channel) => (
            <li key={channel.id} className="space-y-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="text-sm">{textValue(channel.label, true, t) || channel.id}</span>
                {channel.kinds.map((kind) => (
                  <Badge key={kind} variant="outline">
                    {t(runtimeKey(`enums.inputKind.${enumKey(InputKind, kind)}`))}
                  </Badge>
                ))}
              </div>
              {channel.description ? (
                <p className="text-xs leading-5 text-muted-foreground">
                  {textValue(channel.description, true, t)}
                </p>
              ) : null}
              {channel.setup.length > 0 ? (
                <ol className="list-decimal space-y-0.5 pl-5 text-xs leading-5">
                  {channel.setup.map((step) => (
                    <li key={step.key}>{textValue(step, true, t)}</li>
                  ))}
                </ol>
              ) : null}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
