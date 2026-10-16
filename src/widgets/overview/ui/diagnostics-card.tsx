import { CheckCircle2 } from "lucide-react";

import { useWorkspaceDiagnostics } from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { healthClass, textValue } from "@/shared/lib/workspace-view";
import { ConnectionHealth, enumKey } from "@/shared/proto";
import { Badge, Card, CardContent, CardHeader, CardTitle } from "@/shared/ui";

export function DiagnosticsCard() {
  const { t } = useI18n();
  const diagnostics = useWorkspaceDiagnostics();

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("overview.diagnostics.title")}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {diagnostics.length === 0 ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <CheckCircle2 className="h-4 w-4" />
            {t("overview.diagnostics.empty")}
          </div>
        ) : (
          diagnostics.map((item) => (
            <div key={item.id} className="rounded-md border border-border bg-background p-3">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">
                    {textValue(item.title, true, t)}
                  </div>
                  <p className="mt-1 text-sm leading-6 text-muted-foreground">
                    {textValue(item.recovery, true, t)}
                  </p>
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
