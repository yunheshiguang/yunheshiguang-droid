import { ShieldAlert, ShieldCheck } from "lucide-react";
import { useId, useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import {
  ConflictPolicy,
  type PackageItem,
  type PackagePreview,
  type PackageSigner,
} from "@/shared/proto";
import { Badge, Button, Label, Select } from "@/shared/ui";

import {
  type PackageOrigin,
  useInstallPackage,
  useTrustPackageSigner,
} from "../model/use-template-packages";

const POLICIES = ["KEEP_BOTH", "REPLACE", "SKIP"] as const;

type Policy = (typeof POLICIES)[number];

function ItemList({ title, items }: { title: string; items: readonly PackageItem[] }) {
  const { t } = useI18n();

  return items.length === 0 ? null : (
    <div className="space-y-1">
      <p className="text-xs font-medium text-muted-foreground">{title}</p>
      <ul aria-label={title} className="space-y-1">
        {items.map((item) => (
          <li key={item.id} className="flex items-center gap-2 text-sm">
            <span className="truncate">{item.name || item.id}</span>
            {item.exists ? (
              <Badge variant="outline">{t("sharePackages.preview.exists")}</Badge>
            ) : null}
          </li>
        ))}
      </ul>
    </div>
  );
}

function SignerLine({ signer, onTrusted }: { signer?: PackageSigner; onTrusted: () => void }) {
  const { t } = useI18n();
  const { trust, trusting } = useTrustPackageSigner();
  const [trusted, setTrusted] = useState(signer?.trusted ?? false);
  const [refusal, setRefusal] = useState<string | null>(null);

  if (!signer) {
    return <p className="text-xs text-muted-foreground">{t("sharePackages.preview.unsigned")}</p>;
  }

  return (
    <div className="flex flex-wrap items-center gap-2 text-xs">
      <span>{t("sharePackages.preview.signer", { name: signer.name, keyId: signer.keyId })}</span>
      <Badge variant={trusted ? "default" : "outline"}>
        {trusted ? t("sharePackages.preview.trusted") : t("sharePackages.preview.untrusted")}
      </Badge>
      {trusted ? null : (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={trusting}
          onClick={() =>
            void trust({ publicKey: signer.publicKey, name: signer.name })
              .then((result) => {
                if (result.accepted) {
                  setTrusted(true);
                  onTrusted();
                } else {
                  setRefusal(textValue(result.detail, true, t));
                }
              })
              .catch((error: unknown) => setRefusal(errorMessage(error)))
          }
        >
          {t("sharePackages.preview.trust")}
        </Button>
      )}
      {refusal ? (
        <span role="alert" className="text-destructive">
          {refusal}
        </span>
      ) : null}
    </div>
  );
}

type PackagePreviewCardProps = {
  origin: PackageOrigin;
  preview: PackagePreview;
  onTrusted: () => void;
  onInstalled: (message: string) => void;
  onDone: () => void;
};

export function PackagePreviewCard({
  origin,
  preview,
  onTrusted,
  onInstalled,
  onDone,
}: PackagePreviewCardProps) {
  const { t } = useI18n();
  const id = useId();
  const { install, installing } = useInstallPackage();
  const [policy, setPolicy] = useState<Policy>("KEEP_BOTH");
  const [failure, setFailure] = useState<string | null>(null);
  const conflicts = [...preview.flows, ...preview.scripts].some((item) => item.exists);
  const manifest = preview.manifest;

  return (
    <section
      aria-label={t("sharePackages.preview.title")}
      className="space-y-3 rounded-md border border-border p-3"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-sm font-medium">
            {manifest?.name || manifest?.id} {manifest?.version ? `v${manifest.version}` : null}
          </p>
          {manifest?.author ? (
            <p className="text-xs text-muted-foreground">
              {t("sharePackages.preview.author", { author: manifest.author })}
            </p>
          ) : null}
          {manifest?.description ? (
            <p className="text-xs leading-5 text-muted-foreground">{manifest.description}</p>
          ) : null}
        </div>
        <Badge variant={preview.verified ? "default" : "outline"}>
          {preview.verified ? (
            <ShieldCheck className="mr-1 h-3 w-3" />
          ) : (
            <ShieldAlert className="mr-1 h-3 w-3" />
          )}
          {preview.verified
            ? t("sharePackages.preview.verified")
            : t("sharePackages.preview.unverified")}
        </Badge>
      </div>
      <SignerLine signer={preview.signer} onTrusted={onTrusted} />
      <ItemList title={t("sharePackages.preview.flows")} items={preview.flows} />
      <ItemList title={t("sharePackages.preview.scripts")} items={preview.scripts} />
      {preview.problems.length > 0 ? (
        <ul aria-label={t("sharePackages.preview.problems")} className="list-disc space-y-1 pl-4">
          {preview.problems.map((problem) => (
            <li key={JSON.stringify(problem)} className="text-xs text-muted-foreground">
              {textValue(problem, true, t)}
            </li>
          ))}
        </ul>
      ) : null}
      {conflicts ? (
        <div className="space-y-1">
          <Label htmlFor={`${id}-policy`}>{t("sharePackages.preview.onConflict")}</Label>
          <Select
            id={`${id}-policy`}
            value={policy}
            onChange={(event) => setPolicy(event.currentTarget.value as Policy)}
          >
            {POLICIES.map((option) => (
              <option key={option} value={option}>
                {t(`enums.conflictPolicy.${option}`)}
              </option>
            ))}
          </Select>
        </div>
      ) : null}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          type="button"
          size="sm"
          disabled={installing}
          onClick={() => {
            setFailure(null);
            void install({ origin, policy: ConflictPolicy[policy] })
              .then((result) => {
                const message = textValue(result.detail, true, t);

                if (result.accepted) {
                  onInstalled(message);
                } else {
                  setFailure(message);
                }
              })
              .catch((error: unknown) => setFailure(errorMessage(error)));
          }}
        >
          {t("sharePackages.preview.install")}
        </Button>
        <Button type="button" size="sm" variant="ghost" onClick={onDone}>
          {t("sharePackages.preview.close")}
        </Button>
        {failure ? (
          <span role="alert" className="text-xs text-destructive">
            {failure}
          </span>
        ) : null}
      </div>
    </section>
  );
}
