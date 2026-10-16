import type { AdbDeviceActionResult } from "@/shared/proto";

export type BatchRunSummary = {
  total: number;
  succeeded: number;
  failed: number;
};

export function summarizeDeviceResults(results: readonly AdbDeviceActionResult[]): BatchRunSummary {
  const succeeded = results.filter((result) => result.success).length;

  return { total: results.length, succeeded, failed: results.length - succeeded };
}

export function orderDeviceResults(
  results: readonly AdbDeviceActionResult[],
): AdbDeviceActionResult[] {
  return [
    ...results.filter((result) => !result.success),
    ...results.filter((result) => result.success),
  ];
}
