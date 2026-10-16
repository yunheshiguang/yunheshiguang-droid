import { EyeOff, History, Scale, ShieldCheck } from "lucide-react";

import { AuditTrail } from "@/entities/audit";
import { RevealLogsButton } from "@/features/reveal-logs";
import { useGovernanceState, useWorkspaceGroups } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import { groupLabel } from "@/shared/lib/workspace-view";
import { ErrorState, SectionCard } from "@/shared/ui";

import { AuditBrowser } from "./audit-browser";
import { GovernancePolicy } from "./governance-policy";

export function GovernancePage() {
  const { t } = useI18n();
  const { state, loading, error } = useGovernanceState();
  const groups = useWorkspaceGroups();
  const groupName = (id: string) => groupLabel(groups, id, t) ?? id;

  if (error && !state) {
    return (
      <div className="p-6">
        <ErrorState title={t("governance.loadFailed")} detail={error} />
      </div>
    );
  }

  return (
    <div className="grid gap-6 p-6 xl:grid-cols-[1.2fr_0.8fr]">
      <SectionCard
        icon={History}
        title={t("governance.audit.title")}
        description={t("governance.audit.description")}
        contentClassName="space-y-4"
      >
        {!state ? (
          <p role="status" className="text-sm text-muted-foreground">
            {loading ? t("governance.loading") : null}
          </p>
        ) : state.auditStore ? (
          <AuditBrowser groupName={groupName} />
        ) : (
          <AuditTrail
            entries={state.auditTrail}
            capacity={state.auditCapacity}
            groupName={groupName}
          />
        )}
        <RevealLogsButton />
      </SectionCard>

      <div className="space-y-6">
        <SectionCard icon={Scale} title={t("governance.policy.title")}>
          {state ? <GovernancePolicy state={state} /> : null}
        </SectionCard>
        <SectionCard
          icon={EyeOff}
          title={t("governance.privacy.title")}
          description={t("governance.privacy.description")}
          body={t("governance.privacy.body")}
        />
        <SectionCard
          icon={ShieldCheck}
          title={t("governance.boundary.title")}
          description={t("governance.boundary.description")}
          body={t("governance.boundary.body")}
        />
      </div>
    </div>
  );
}
