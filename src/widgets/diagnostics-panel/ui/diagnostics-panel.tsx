import { AlertTriangle, CheckCircle2 } from "lucide-react";
import type { ReactNode } from "react";

import { runtimeKey, useI18n } from "@/shared/i18n";
import { healthClass, textValue } from "@/shared/lib/workspace-view";
import { ConnectionHealth, type DiagnosticSignal, enumKey } from "@/shared/proto";
import { Badge, Card, CardContent, CardHeader, CardTitle } from "@/shared/ui";

type DiagnosticsPanelProps = {
  diagnostics: readonly DiagnosticSignal[];
  action?: ReactNode;
};

export function DiagnosticsPanel({ diagnostics, action }: DiagnosticsPanelProps) {
  const { t } = useI18n();

  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-4">
        <CardTitle>{t("diagnosticsPanel.title")}</CardTitle>
        {action}
      </CardHeader>
      <CardContent className="space-y-3">
        {diagnostics.length === 0 ? (
          <div className="flex items-center gap-2 rounded-md border border-border bg-background p-3 text-sm text-muted-foreground">
            <CheckCircle2 className="h-4 w-4" />
            {t("diagnosticsPanel.empty")}
          </div>
        ) : (
          diagnostics.map((item) => (
            <div key={item.id} className="rounded-md border border-border bg-background p-3">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                    <h3 className="truncate text-sm font-medium">
                      {textValue(item.title, true, t)}
                    </h3>
                  </div>
                  <p className="mt-2 text-sm leading-6 text-muted-foreground">
                    {textValue(item.impact, true, t)}
                  </p>
                  <p className="mt-1 text-sm leading-6">{textValue(item.recovery, true, t)}</p>
                </div>
                <Badge variant="outline" className={healthClass(item.health)}>
                  {t(
                    runtimeKey(`enums.connectionHealth.${enumKey(ConnectionHealth, item.health)}`),
                  )}
                </Badge>
              </div>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  );
}
