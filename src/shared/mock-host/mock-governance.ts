import { toJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import { compareIds } from "@/shared/lib/text";
import {
  type AuditEntryJson,
  AuditExportFormatSchema,
  AuditPageSchema,
  type AuditQuery,
  type AuditQueryJson,
  AuditQuerySchema,
  AuditReportSchema,
  type AuditService,
  type AuditTallyJson,
  type GovernanceProviderJson,
  type GovernanceService,
  GovernanceStateSchema,
  type GovernedActionJson,
  type OperatorJson,
  type UiTextJson,
} from "@/shared/proto";

import { fromMockJson } from "./mock-context";

const AUDIT_CAPACITY = 200;
const DEFAULT_PAGE_SIZE = 100;

export const MOCK_OPERATOR: OperatorJson = {
  id: "local:browser",
  name: "Browser preview",
  roles: ["owner"],
};

function tally(entries: readonly AuditEntryJson[], utcOffsetMinutes: number) {
  const tables = {
    operators: new Map(),
    groups: new Map(),
    actions: new Map(),
    days: new Map(),
  } as Record<"operators" | "groups" | "actions" | "days", Map<string, AuditTallyJson>>;
  const add = (
    table: Map<string, AuditTallyJson>,
    key: string,
    entry: AuditEntryJson,
    label = "",
  ) => {
    const row = table.get(key) ?? { key, label, total: 0, succeeded: 0, failed: 0, denied: 0 };
    row.total = (row.total ?? 0) + 1;
    if (entry.outcome === "AUDIT_OUTCOME_SUCCEEDED") row.succeeded = (row.succeeded ?? 0) + 1;
    if (entry.outcome === "AUDIT_OUTCOME_FAILED") row.failed = (row.failed ?? 0) + 1;
    if (entry.outcome === "AUDIT_OUTCOME_DENIED") row.denied = (row.denied ?? 0) + 1;
    table.set(key, row);
  };

  for (const entry of entries) {
    add(tables.operators, entry.operator?.id ?? "", entry, entry.operator?.name ?? "");
    for (const group of new Set(entry.groups ?? [])) {
      add(tables.groups, group, entry);
    }
    add(tables.actions, entry.action ?? "GOVERNED_ACTION_UNSPECIFIED", entry);
    const local = new Date(Date.parse(entry.occurredAt ?? "") + utcOffsetMinutes * 60_000);
    add(tables.days, local.toISOString().slice(0, 10), entry);
  }

  const busiest = (table: Map<string, AuditTallyJson>) =>
    [...table.values()].sort(
      (left, right) =>
        (right.total ?? 0) - (left.total ?? 0) || compareIds(left.key ?? "", right.key ?? ""),
    );

  return {
    byOperator: busiest(tables.operators),
    byGroup: busiest(tables.groups),
    byAction: busiest(tables.actions),
    byDay: [...tables.days.values()].sort((left, right) =>
      compareIds(left.key ?? "", right.key ?? ""),
    ),
    entries: entries.length,
  };
}

function maskDeviceId(id: string): string {
  return id.length <= 4 ? "****" : `***${id.slice(-4)}`;
}

function governanceProvider(id: string, name: string): GovernanceProviderJson {
  return {
    id,
    label: { key: `workspace.governance.provider.${name}.label` },
    description: { key: `workspace.governance.provider.${name}.description` },
  };
}

export type MockAudit = (
  action: GovernedActionJson,
  succeeded: boolean,
  targets: readonly string[],
  detail?: UiTextJson,
) => void;

export function createMockGovernance(settle: <T>(value: T) => Promise<T>): {
  audit: MockAudit;
  governance: ServiceImpl<typeof GovernanceService>;
  auditing: ServiceImpl<typeof AuditService>;
} {
  const history: AuditEntryJson[] = [];

  const audit: MockAudit = (action, succeeded, targets, detail) => {
    history.unshift({
      id: crypto.randomUUID(),
      occurredAt: new Date().toISOString(),
      action,
      outcome: succeeded ? "AUDIT_OUTCOME_SUCCEEDED" : "AUDIT_OUTCOME_FAILED",
      actor: "main",
      operator: MOCK_OPERATOR,
      targets: targets.map(maskDeviceId),
      groups: [],
      detail,
    });
  };

  const matches = (request?: AuditQuery) => {
    const query: AuditQueryJson = request ? toJson(AuditQuerySchema, request) : {};
    const since = query.since ? Date.parse(query.since) : undefined;
    const until = query.until ? Date.parse(query.until) : undefined;

    return history.filter((entry) => {
      const at = Date.parse(entry.occurredAt ?? "");

      return (
        (!query.actions?.length ||
          query.actions.includes(entry.action ?? "GOVERNED_ACTION_UNSPECIFIED")) &&
        (!query.outcomes?.length ||
          query.outcomes.includes(entry.outcome ?? "AUDIT_OUTCOME_UNSPECIFIED")) &&
        (!query.operatorId || entry.operator?.id === query.operatorId) &&
        (!query.actor || entry.actor === query.actor) &&
        (since === undefined || at >= since) &&
        (until === undefined || at < until)
      );
    });
  };

  return {
    audit,
    governance: {
      getGovernanceState: () =>
        settle(
          fromMockJson(GovernanceStateSchema, {
            authorization: governanceProvider("core.local-operator", "localOperator"),
            entitlements: governanceProvider("core.open-source", "coreEntitlements"),
            auditSinks: [
              governanceProvider("core.session-trail", "sessionTrail"),
              governanceProvider("core.run-log", "runLog"),
            ],
            auditTrail: history.slice(0, AUDIT_CAPACITY),
            auditCapacity: AUDIT_CAPACITY,
            identity: governanceProvider("core.local-user", "localUser"),
            operator: MOCK_OPERATOR,
            auditStore: governanceProvider("core.sqlite-audit", "sqliteAudit"),
          }),
        ),
    },
    auditing: {
      queryAudit: ({ query }) => {
        const size = query?.limit || DEFAULT_PAGE_SIZE;
        const selected = matches(query);
        const start = query?.beforeId
          ? selected.findIndex((entry) => entry.id === query.beforeId) + 1
          : 0;
        const page = selected.slice(start, start + size);
        const more = start + size < selected.length;

        return settle(
          fromMockJson(AuditPageSchema, {
            entries: page,
            nextBeforeId: more ? (page.at(-1)?.id ?? "") : "",
          }),
        );
      },
      reportAudit: ({ query, utcOffsetMinutes }) =>
        settle(fromMockJson(AuditReportSchema, tally(matches(query), utcOffsetMinutes))),
      exportAudit: ({ query, format, path }) => {
        if (!path.startsWith("/")) {
          return settle({ accepted: false, detail: { key: "workspace.audit.invalid.path" } });
        }
        const exported = matches(query).length;
        audit("GOVERNED_ACTION_EXPORT_AUDIT", true, [], {
          key: "workspace.audit.audit.exported",
          params: {
            count: String(exported),
            format: AuditExportFormatSchema.value[format]?.name ?? "",
          },
        });

        return settle({
          accepted: true,
          detail: { key: "workspace.audit.exported", params: { count: String(exported), path } },
          exported,
        });
      },
    },
  };
}
