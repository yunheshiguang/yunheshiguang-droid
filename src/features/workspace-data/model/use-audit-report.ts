import { timestampFromMs } from "@bufbuild/protobuf/wkt";
import { useQuery } from "@connectrpc/connect-query";
import { useMemo } from "react";

import { errorMessage } from "@/shared/lib/error-message";
import { type AuditReport, AuditService } from "@/shared/proto";

export type AuditReportStatus = {
  report?: AuditReport;
  loading: boolean;
  error: string | null;
};

function startOfPeriod(days: number): number {
  const start = new Date();
  start.setHours(0, 0, 0, 0);
  start.setDate(start.getDate() - (days - 1));

  return start.getTime();
}

export function useAuditReport(days: number): AuditReportStatus {
  const since = useMemo(() => startOfPeriod(days), [days]);
  const { data, isPending, error } = useQuery(AuditService.method.reportAudit, {
    query: { since: timestampFromMs(since) },
    utcOffsetMinutes: -new Date().getTimezoneOffset(),
  });

  return { report: data, loading: isPending, error: error == null ? null : errorMessage(error) };
}
