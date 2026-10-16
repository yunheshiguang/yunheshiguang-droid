import { FolderSync, KeyRound, ShieldCheck } from "lucide-react";
import { useMemo } from "react";
import {
  Badge,
  ErrorState,
  enumKey,
  type License,
  LicenseStatus,
  type Operator,
  SectionCard,
  timestampDate,
  useGovernanceState,
  useI18n,
} from "yunheshiguang/sdk";

import { type FileState, type ReferenceStatus, useReferenceStatus } from "./reference-status";

function FileStateText({ file }: { file: FileState }) {
  const { t } = useI18n();

  return (
    <span className={file.state === "broken" ? "text-destructive" : "text-muted-foreground"}>
      {file.state === "broken"
        ? t("enterprise.state.broken", { detail: file.detail })
        : t(`enterprise.state.${file.state}`)}
    </span>
  );
}

function PathRow({ label, path, file }: { label: string; path: string; file?: FileState }) {
  return (
    <div className="space-y-1">
      <dt className="text-xs font-medium text-muted-foreground">{label}</dt>
      <dd className="break-all font-mono text-xs">{path}</dd>
      {file ? (
        <dd className="text-xs">
          <FileStateText file={file} />
        </dd>
      ) : null}
    </div>
  );
}

function LicenseDetail({
  license,
  capabilities,
}: {
  license?: License;
  capabilities: readonly string[];
}) {
  const { t, locale } = useI18n();
  const dateFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "medium" }),
    [locale],
  );

  if (!license) {
    return <p className="text-sm text-muted-foreground">{t("enterprise.license.none")}</p>;
  }

  return (
    <div className="space-y-2 text-sm">
      <p className="font-medium">
        {license.plan ? t(license.plan.key, license.plan.params) : license.planId}{" "}
        <Badge variant="outline">
          {t(`enums.licenseStatus.${enumKey(LicenseStatus, license.status)}`)}
        </Badge>
      </p>
      {license.licensee ? (
        <p className="text-muted-foreground">
          {t("enterprise.license.licensee", { name: license.licensee })}
        </p>
      ) : null}
      <p className="text-muted-foreground">
        {license.expiresAt === undefined
          ? t("enterprise.license.perpetual")
          : t("enterprise.license.expires", {
              date: dateFormat.format(timestampDate(license.expiresAt)),
            })}
      </p>
      <ul className="flex flex-wrap gap-1">
        {capabilities.map((capability) => (
          <li key={capability}>
            <Badge variant="subtle">{capability}</Badge>
          </li>
        ))}
      </ul>
    </div>
  );
}

function TeamDetail({ status, operator }: { status: ReferenceStatus; operator?: Operator }) {
  const { t } = useI18n();

  return (
    <div className="space-y-2 text-sm">
      {operator ? (
        <p className="font-medium">
          {t("enterprise.team.operator", { name: operator.name || operator.id })}{" "}
          {operator.roles.map((role) => (
            <Badge key={role} variant="outline" className="ml-1">
              {role}
            </Badge>
          ))}
        </p>
      ) : null}
      <p className="text-muted-foreground">
        {status.team.state === "loaded"
          ? t("enterprise.team.members", { count: String(status.teamMembers) })
          : status.team.state === "missing"
            ? t("enterprise.team.none")
            : t("enterprise.team.broken")}
      </p>
      <p className="text-muted-foreground">
        {status.policy.state === "loaded"
          ? t("enterprise.policy.rules", { count: String(status.policyRules) })
          : status.policy.state === "missing"
            ? t("enterprise.policy.none")
            : t("enterprise.policy.broken")}
      </p>
    </div>
  );
}

export function EnterprisePage() {
  const { t } = useI18n();
  const governance = useGovernanceState();
  const reference = useReferenceStatus();

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <SectionCard icon={KeyRound} title={t("enterprise.license.title")}>
        <LicenseDetail
          license={governance.state?.license}
          capabilities={governance.state?.grantedCapabilities ?? []}
        />
      </SectionCard>
      {reference.error ? (
        <ErrorState title={t("enterprise.loadFailed")} detail={reference.error} />
      ) : reference.data === undefined ? (
        <p className="text-sm text-muted-foreground">{t("enterprise.loading")}</p>
      ) : (
        <>
          <SectionCard
            icon={ShieldCheck}
            title={t("enterprise.policy.title")}
            description={t("enterprise.policy.description")}
          >
            <TeamDetail status={reference.data} operator={governance.state?.operator} />
          </SectionCard>
          <SectionCard
            icon={FolderSync}
            title={t("enterprise.files.title")}
            description={t("enterprise.files.description")}
            className="xl:col-span-2"
          >
            <dl className="grid gap-4 md:grid-cols-2">
              <PathRow
                label={t("enterprise.files.team")}
                path={reference.data.teamFile}
                file={reference.data.team}
              />
              <PathRow
                label={t("enterprise.files.policy")}
                path={reference.data.policyFile}
                file={reference.data.policy}
              />
              <PathRow
                label={t("enterprise.files.license")}
                path={reference.data.licenseFile}
                file={reference.data.license}
              />
              <PathRow
                label={t("enterprise.files.sync")}
                path={reference.data.syncFile}
                file={reference.data.sync}
              />
              <PathRow label={t("enterprise.files.documents")} path={reference.data.syncFolder} />
              <PathRow label={t("enterprise.files.audit")} path={reference.data.auditFolder} />
              <PathRow
                label={t("enterprise.files.assistant")}
                path={reference.data.assistantFile}
                file={reference.data.assistant}
              />
              <PathRow
                label={t("enterprise.files.redroid")}
                path={reference.data.redroidFile}
                file={reference.data.redroid}
              />
              <PathRow
                label={t("enterprise.files.vendors")}
                path={reference.data.vendorsFile}
                file={reference.data.vendors}
              />
              <PathRow label={t("enterprise.files.metrics")} path={reference.data.metricsFile} />
              <PathRow
                label={t("enterprise.files.apiKeys")}
                path={reference.data.apiKeysFile}
                file={reference.data.apiKeys}
              />
            </dl>
          </SectionCard>
        </>
      )}
    </div>
  );
}
