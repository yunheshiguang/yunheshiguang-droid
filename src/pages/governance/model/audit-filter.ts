import type { MessageInitShape } from "@bufbuild/protobuf";
import { timestampFromMs } from "@bufbuild/protobuf/wkt";

import { AuditOutcome, type AuditQuerySchema, enumKeys, GovernedAction } from "@/shared/proto";

export const auditPeriods = ["all", "day", "week", "month"] as const;
export type AuditPeriod = (typeof auditPeriods)[number];

export const governedActions = enumKeys(GovernedAction);
export const auditOutcomes = enumKeys(AuditOutcome);

const DAY_MS = 86_400_000;
const PERIOD_MS = { day: DAY_MS, week: 7 * DAY_MS, month: 30 * DAY_MS } as const;

export type AuditFilter = {
  action: (typeof governedActions)[number] | "";
  outcome: (typeof auditOutcomes)[number] | "";
  operatorId: string;
  period: AuditPeriod;
  since?: number;
};

export const emptyAuditFilter: AuditFilter = {
  action: "",
  outcome: "",
  operatorId: "",
  period: "all",
};

export function withPeriod(filter: AuditFilter, period: AuditPeriod, now: number): AuditFilter {
  return { ...filter, period, since: period === "all" ? undefined : now - PERIOD_MS[period] };
}

export function toAuditQuery(filter: AuditFilter): MessageInitShape<typeof AuditQuerySchema> {
  return {
    actions: filter.action ? [GovernedAction[filter.action]] : [],
    outcomes: filter.outcome ? [AuditOutcome[filter.outcome]] : [],
    operatorId: filter.operatorId,
    since: filter.since === undefined ? undefined : timestampFromMs(filter.since),
  };
}
