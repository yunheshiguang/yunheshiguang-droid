export interface StatusTokenSet {
  tokens: Readonly<Record<string, string>>;
  fallback: string;
}

export function resolveStatusToken(set: StatusTokenSet, status: string | undefined): string {
  if (!status || !Object.hasOwn(set.tokens, status)) {
    return set.fallback;
  }

  return set.tokens[status];
}

export type StatusTone = "positive" | "info" | "caution" | "critical" | "neutral";

const badgeTones: Readonly<Record<StatusTone, string>> = {
  positive: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
  info: "border-sky-500/25 bg-sky-500/10 text-sky-700 dark:text-sky-300",
  caution: "border-amber-500/25 bg-amber-500/10 text-amber-700 dark:text-amber-300",
  critical: "border-rose-500/25 bg-rose-500/10 text-rose-700 dark:text-rose-300",
  neutral: "border-border bg-muted text-muted-foreground",
};

export const statusTextTones: Readonly<Record<StatusTone, string>> = {
  positive: "text-emerald-700 dark:text-emerald-300",
  info: "text-sky-700 dark:text-sky-300",
  caution: "text-amber-700 dark:text-amber-300",
  critical: "text-rose-700 dark:text-rose-300",
  neutral: "text-muted-foreground",
};

const NEUTRAL_BADGE = badgeTones.neutral;
const POSITIVE_BADGE = badgeTones.positive;
const CAUTION_BADGE = badgeTones.caution;
const INFO_BADGE = badgeTones.info;
const CRITICAL_BADGE = badgeTones.critical;

export const connectionHealthTokens: StatusTokenSet = {
  tokens: {
    READY: POSITIVE_BADGE,
    DEGRADED: CAUTION_BADGE,
    RECOVERING: INFO_BADGE,
    BLOCKED: CRITICAL_BADGE,
  },
  fallback: NEUTRAL_BADGE,
};

export const installStatusTokens: StatusTokenSet = {
  tokens: {
    INSTALLED: POSITIVE_BADGE,
    INSTALLING: INFO_BADGE,
    FAILED: CRITICAL_BADGE,
  },
  fallback: NEUTRAL_BADGE,
};

export const deviceStatusTokens: StatusTokenSet = {
  tokens: {
    ONLINE: POSITIVE_BADGE,
    BUSY: INFO_BADGE,
    IDLE: NEUTRAL_BADGE,
    WARNING: CAUTION_BADGE,
    OFFLINE: CRITICAL_BADGE,
  },
  fallback: NEUTRAL_BADGE,
};

export const groupToneTokens: StatusTokenSet = {
  tokens: {
    EMERALD: "bg-emerald-500",
    SKY: "bg-sky-500",
    VIOLET: "bg-violet-500",
    AMBER: "bg-amber-500",
    ROSE: "bg-rose-500",
    SLATE: "bg-slate-500",
  },
  fallback: "bg-muted-foreground",
};

export const taskStatusTokens: StatusTokenSet = {
  tokens: {
    DONE: statusTextTones.positive,
    RUNNING: statusTextTones.info,
    RETRYING: statusTextTones.info,
    BLOCKED: statusTextTones.critical,
    FAILED: statusTextTones.critical,
  },
  fallback: statusTextTones.neutral,
};
