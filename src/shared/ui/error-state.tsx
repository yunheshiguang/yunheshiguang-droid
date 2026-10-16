import { AlertTriangle } from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/utils";

type ErrorStateProps = {
  title: string;
  detail?: string;
  action?: ReactNode;
  variant?: "inline" | "block";
  className?: string;
};

export function ErrorState({
  title,
  detail,
  action,
  variant = "inline",
  className,
}: ErrorStateProps) {
  if (variant === "inline") {
    return (
      <div
        role="alert"
        className={cn(
          "rounded-md border border-destructive/25 bg-destructive/10 p-3 text-sm text-destructive",
          className,
        )}
      >
        <div className="font-medium">{title}</div>
        {detail ? <p className="mt-1 break-words leading-6">{detail}</p> : null}
        {action ? <div className="pt-2">{action}</div> : null}
      </div>
    );
  }

  return (
    <div
      role="alert"
      className={cn(
        "flex min-h-[18rem] flex-col items-center justify-center gap-3 rounded-md border border-destructive/25 bg-destructive/10 p-8 text-center",
        className,
      )}
    >
      <div className="flex h-11 w-11 items-center justify-center rounded-md border border-destructive/25 bg-background">
        <AlertTriangle className="h-5 w-5 text-destructive" />
      </div>
      <div className="space-y-1">
        <h2 className="text-sm font-semibold text-destructive">{title}</h2>
        {detail ? (
          <p className="max-w-md break-words text-sm leading-6 text-destructive/80">{detail}</p>
        ) : null}
      </div>
      {action ? <div className="pt-2">{action}</div> : null}
    </div>
  );
}
