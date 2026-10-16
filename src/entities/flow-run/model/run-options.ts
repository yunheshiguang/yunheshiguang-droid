import type { RunOptions } from "@/shared/proto";

export type RunOptionValues = Pick<
  RunOptions,
  "repeat" | "repeatIntervalSeconds" | "retryAttempts" | "retryDelaySeconds"
>;

export const runOptionLimits = {
  repeat: { min: 1, max: 1000 },
  repeatIntervalSeconds: { min: 0, max: 86_400 },
  retryAttempts: { min: 0, max: 10 },
  retryDelaySeconds: { min: 0, max: 3_600 },
} satisfies Record<keyof RunOptionValues, { min: number; max: number }>;

export const defaultRunOptions: RunOptionValues = {
  repeat: 1,
  repeatIntervalSeconds: 0,
  retryAttempts: 0,
  retryDelaySeconds: 5,
};

export function runOptionValues(options: RunOptions | undefined): RunOptionValues {
  return options
    ? {
        repeat: options.repeat,
        repeatIntervalSeconds: options.repeatIntervalSeconds,
        retryAttempts: options.retryAttempts,
        retryDelaySeconds: options.retryDelaySeconds,
      }
    : defaultRunOptions;
}

export function clampRunOption(key: keyof RunOptionValues, value: number): number {
  const { min, max } = runOptionLimits[key];

  return Number.isFinite(value) ? Math.min(max, Math.max(min, Math.round(value))) : min;
}
