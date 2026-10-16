import type { LucideIcon } from "lucide-react";

import { formatCompactNumber } from "@/shared/lib/text";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/shared/ui";

export type MetricCardProps = {
  icon: LucideIcon;
  label: string;
  value: number;
  note: string;
  locale: string;
};

export function MetricCard({ icon: Icon, label, value, note, locale }: MetricCardProps) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardDescription className="flex items-center gap-2">
          <Icon className="h-4 w-4" />
          {label}
        </CardDescription>
        <CardTitle className="text-3xl font-semibold tracking-normal">
          {formatCompactNumber(value, locale)}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <p className="truncate text-sm text-muted-foreground">{note}</p>
      </CardContent>
    </Card>
  );
}
