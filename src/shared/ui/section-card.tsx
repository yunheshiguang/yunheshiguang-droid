import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/utils";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./card";

type SectionCardProps = {
  title: string;
  icon?: LucideIcon;
  description?: string;
  body?: string;
  children?: ReactNode;
  className?: string;
  contentClassName?: string;
};

export function SectionCard({
  title,
  icon: Icon,
  description,
  body,
  children,
  className,
  contentClassName,
}: SectionCardProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className={cn(Icon && "flex items-center gap-2")}>
          {Icon ? <Icon className="h-4 w-4" /> : null}
          {title}
        </CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : null}
      </CardHeader>
      {body || children ? (
        <CardContent className={contentClassName}>
          {body ? <p className="text-sm leading-6 text-muted-foreground">{body}</p> : null}
          {children}
        </CardContent>
      ) : null}
    </Card>
  );
}
