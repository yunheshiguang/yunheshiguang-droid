import { useWorkspaceConnectionPlans } from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import { ConnectionType, enumKey } from "@/shared/proto";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui";

export function ConnectionPlanSummary({ className }: { className?: string }) {
  const { t } = useI18n();
  const plans = useWorkspaceConnectionPlans();

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{t("workspace.connection.sectionTitle")}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {plans.map((plan) => (
          <div key={plan.id} className="rounded-md border border-border bg-background p-3">
            <div className="text-sm font-medium">{textValue(plan.title, true, t)}</div>
            <p className="mt-1 text-xs text-muted-foreground">
              {t(runtimeKey(`enums.connectionType.${enumKey(ConnectionType, plan.connection)}`))}
            </p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
