import { CheckCircle2 } from "lucide-react";

import { useWorkspaceConnectionPlans } from "@/features/workspace-data";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { healthClass, textValue } from "@/shared/lib/workspace-view";
import { ConnectionHealth, ConnectionType, enumKey } from "@/shared/proto";
import {
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Separator,
} from "@/shared/ui";

export function ConnectionPlanGrid() {
  const { t } = useI18n();
  const plans = useWorkspaceConnectionPlans();

  return (
    <section className="grid gap-4 lg:grid-cols-3">
      {plans.map((plan) => (
        <Card key={plan.id}>
          <CardHeader>
            <CardTitle>{textValue(plan.title, true, t)}</CardTitle>
            <CardDescription>
              {t(runtimeKey(`enums.connectionType.${enumKey(ConnectionType, plan.connection)}`))}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Badge variant="outline" className={healthClass(plan.health)}>
              {t(runtimeKey(`enums.connectionHealth.${enumKey(ConnectionHealth, plan.health)}`))}
            </Badge>
            <Separator />
            <div className="space-y-2">
              {plan.steps.map((step) => (
                <div key={step.key} className="flex items-start gap-2 text-sm">
                  <CheckCircle2 className="mt-0.5 h-4 w-4 text-muted-foreground" />
                  <span>{textValue(step, true, t)}</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ))}
    </section>
  );
}
