import { timestampDate } from "@bufbuild/protobuf/wkt";
import { type ReactNode, useMemo } from "react";

import { useI18n } from "@/shared/i18n";
import { type StatusTone, statusTextTones } from "@/shared/lib/status-tokens";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import {
  enumKey,
  type GovernanceProvider,
  type GovernanceState,
  type License,
  LicenseStatus,
  type Operator,
} from "@/shared/proto";
import { Badge } from "@/shared/ui";

function ProviderDetail({ provider }: { provider?: GovernanceProvider }) {
  const { t } = useI18n();

  if (!provider) {
    return <p className="text-sm text-muted-foreground">{t("governance.policy.unknown")}</p>;
  }

  return (
    <div className="space-y-1">
      <p className="text-sm font-medium">{textValue(provider.label, true, t) || provider.id}</p>
      <p className="text-xs leading-5 text-muted-foreground">
        {textValue(provider.description, true, t)}
      </p>
    </div>
  );
}

function PolicyRow({ term, children }: { term: string; children: ReactNode }) {
  return (
    <div className="space-y-2">
      <dt className="text-xs font-medium uppercase tracking-wide text-muted-foreground">{term}</dt>
      <dd className="space-y-2">{children}</dd>
    </div>
  );
}

const LICENSE_TONES = {
  [LicenseStatus.UNSPECIFIED]: "neutral",
  [LicenseStatus.ACTIVE]: "positive",
  [LicenseStatus.TRIAL]: "info",
  [LicenseStatus.EXPIRED]: "critical",
} as const satisfies Record<LicenseStatus, StatusTone>;

function LicenseDetail({ license }: { license?: License }) {
  const { t, locale } = useI18n();
  const dateFormat = useMemo(
    () => new Intl.DateTimeFormat(locale, { dateStyle: "medium" }),
    [locale],
  );

  if (!license) {
    return <p className="text-xs text-muted-foreground">{t("governance.policy.noLicense")}</p>;
  }

  return (
    <div className="space-y-1">
      <p className="text-sm font-medium">
        {textValue(license.plan, true, t) || license.planId}{" "}
        <span
          className={cn("text-xs", statusTextTones[LICENSE_TONES[license.status] ?? "neutral"])}
        >
          {t(`enums.licenseStatus.${enumKey(LicenseStatus, license.status)}`)}
        </span>
      </p>
      {license.licensee ? (
        <p className="text-xs text-muted-foreground">
          {t("governance.policy.licensee", { name: license.licensee })}
        </p>
      ) : null}
      {license.expiresAt === undefined ? null : (
        <p className="text-xs text-muted-foreground">
          {t("governance.policy.expires", {
            date: dateFormat.format(timestampDate(license.expiresAt)),
          })}
        </p>
      )}
    </div>
  );
}

function OperatorDetail({ operator }: { operator?: Operator }) {
  const { t } = useI18n();

  if (!operator) {
    return <p className="text-xs text-muted-foreground">{t("governance.policy.unknown")}</p>;
  }

  return (
    <div className="space-y-1">
      <p className="text-sm font-medium">
        {t("governance.policy.operator", { name: operator.name || operator.id })}
      </p>
      {operator.roles.length === 0 ? (
        <p className="text-xs text-muted-foreground">{t("governance.policy.noRoles")}</p>
      ) : (
        <ul aria-label={t("governance.policy.roles")} className="flex flex-wrap gap-1">
          {operator.roles.map((role) => (
            <li key={role}>
              <Badge variant="outline">{role}</Badge>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function GovernancePolicy({ state }: { state: GovernanceState }) {
  const { t } = useI18n();

  return (
    <dl className="space-y-5">
      <PolicyRow term={t("governance.policy.identity")}>
        <ProviderDetail provider={state.identity} />
        <OperatorDetail operator={state.operator} />
      </PolicyRow>
      <PolicyRow term={t("governance.policy.authorization")}>
        <ProviderDetail provider={state.authorization} />
      </PolicyRow>
      <PolicyRow term={t("governance.policy.entitlements")}>
        <ProviderDetail provider={state.entitlements} />
        {state.grantedCapabilities.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t("governance.policy.noCapabilities")}</p>
        ) : (
          <ul className="flex flex-wrap gap-1">
            {state.grantedCapabilities.map((capability) => (
              <li key={capability}>
                <Badge variant="outline">{capability}</Badge>
              </li>
            ))}
          </ul>
        )}
      </PolicyRow>
      <PolicyRow term={t("governance.policy.license")}>
        <LicenseDetail license={state.license} />
      </PolicyRow>
      <PolicyRow term={t("governance.policy.auditStore")}>
        <ProviderDetail provider={state.auditStore} />
      </PolicyRow>
      <PolicyRow term={t("governance.policy.auditSinks")}>
        {state.auditSinks.map((sink) => (
          <ProviderDetail key={sink.id} provider={sink} />
        ))}
      </PolicyRow>
    </dl>
  );
}
