import { Loader2 } from "lucide-react";

import { cn } from "@/shared/lib/utils";

type LoadingScreenProps = {
  label: string;
  className?: string;
};

export function LoadingScreen({ label, className }: LoadingScreenProps) {
  return (
    <div
      role="status"
      className={cn(
        "absolute inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-sm",
        className,
      )}
    >
      <div className="flex items-center gap-3 rounded-md border border-border bg-card px-4 py-3 shadow-sm">
        <Loader2 aria-hidden="true" className="h-5 w-5 animate-spin text-muted-foreground" />
        <span className="text-sm font-medium text-foreground">{label}</span>
      </div>
    </div>
  );
}
