import type { LucideIcon } from "lucide-react";

import { cn } from "@/shared/lib/utils";

type EmptyStateProps = {
  icon: LucideIcon;
  title: string;
  description?: string;
  className?: string;
  action?: React.ReactNode;
};

export function EmptyState({ icon: Icon, title, description, className, action }: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex min-h-[18rem] flex-col items-center justify-center gap-3 rounded-md border border-dashed border-border bg-muted/25 p-8 text-center",
        className,
      )}
    >
      <div className="flex h-11 w-11 items-center justify-center rounded-md border border-border bg-background">
        <Icon className="h-5 w-5 text-muted-foreground" />
      </div>
      <div className="space-y-1">
        <h2 className="text-sm font-semibold">{title}</h2>
        {description ? (
          <p className="max-w-md text-sm leading-6 text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {action ? <div className="pt-2">{action}</div> : null}
    </div>
  );
}
