import { useI18n } from "@/shared/i18n";
import type { AuditEntry } from "@/shared/proto";

import { AuditEntryList } from "./audit-entry-list";

type AuditTrailProps = {
  entries: readonly AuditEntry[];
  capacity: number;
  groupName?: (id: string) => string;
};

export function AuditTrail({ entries, capacity, groupName }: AuditTrailProps) {
  const { t } = useI18n();

  return (
    <div className="space-y-3">
      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("audit.empty")}</p>
      ) : (
        <AuditEntryList entries={entries} groupName={groupName} />
      )}
      <p className="text-xs text-muted-foreground">
        {t("audit.capacity", { count: String(capacity) })}
      </p>
    </div>
  );
}
