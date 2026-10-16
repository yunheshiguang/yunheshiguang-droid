import { UserRound } from "lucide-react";

import { useCurrentOperator } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";

export function OperatorBadge() {
  const { t } = useI18n();
  const operator = useCurrentOperator();

  if (!operator) {
    return null;
  }

  const name = operator.name || operator.id;
  const roles = operator.roles.join(", ") || t("windowChrome.noRoles");

  return (
    <span
      className="hidden max-w-[12rem] items-center gap-1.5 truncate rounded-md border border-border px-2 py-1 text-xs text-muted-foreground md:inline-flex"
      title={t("windowChrome.operator", { name, roles })}
    >
      <UserRound className="h-3.5 w-3.5 shrink-0" />
      <span className="truncate">{name}</span>
    </span>
  );
}
