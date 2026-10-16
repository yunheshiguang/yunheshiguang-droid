import { ShieldCheck } from "lucide-react";

import { useWorkspaceSnapshot } from "@/features/workspace-data";
import {
  assessSchemaCompatibility,
  EXPECTED_SCHEMA_VERSION,
  schemaCompatibilityMessageKeys,
} from "@/shared/api";
import { type PlainMessageKey, useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import type { WorkspaceSnapshot } from "@/shared/proto";
import { SectionCard } from "@/shared/ui";

const selectProtocolFacts = (snapshot: WorkspaceSnapshot) => ({
  schemaVersion: snapshot.schemaVersion,
  hostPlatform: snapshot.hostPlatform,
  adbPath: snapshot.adbPath,
  fastbootPath: snapshot.fastbootPath,
});

export function ProtocolPage() {
  const { t } = useI18n();
  const snapshot = useWorkspaceSnapshot(selectProtocolFacts);
  const compatibility = assessSchemaCompatibility(snapshot?.schemaVersion);
  const runtimeFacts: { labelKey: PlainMessageKey; value?: string }[] = [
    { labelKey: "protocol.hostPlatform", value: snapshot?.hostPlatform },
    { labelKey: "protocol.adbPath", value: snapshot?.adbPath },
    { labelKey: "protocol.fastbootPath", value: snapshot?.fastbootPath },
  ];

  return (
    <div className="space-y-4 p-6">
      <SectionCard icon={ShieldCheck} title={t("protocol.title")} />

      <div className="grid gap-3 md:grid-cols-2">
        <SectionCard
          title={snapshot?.schemaVersion || "-"}
          description={t("protocol.schemaVersion")}
        >
          <p
            className={cn(
              "text-xs",
              compatibility === "compatible"
                ? "text-muted-foreground"
                : "text-amber-600 dark:text-amber-400",
            )}
          >
            {t(schemaCompatibilityMessageKeys[compatibility], {
              expected: EXPECTED_SCHEMA_VERSION,
            })}
          </p>
        </SectionCard>
        {runtimeFacts.map((fact) => (
          <SectionCard
            key={fact.labelKey}
            title={fact.value || "-"}
            description={t(fact.labelKey)}
          />
        ))}
      </div>
    </div>
  );
}
