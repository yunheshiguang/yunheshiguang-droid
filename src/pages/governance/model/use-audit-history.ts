import {
  callUnaryMethod,
  createConnectQueryKey,
  useQuery,
  useTransport,
} from "@connectrpc/connect-query";
import { useInfiniteQuery, useMutation } from "@tanstack/react-query";

import { dialogGateway, workspaceMutationKeys } from "@/shared/api";
import {
  type AuditEntry,
  AuditExportFormat,
  AuditService,
  type AuditTally,
  type ExportAuditResult,
  type UiText,
} from "@/shared/proto";

import { type AuditFilter, toAuditQuery } from "./audit-filter";

const AUDIT_PAGE_SIZE = 50;
const queryAudit = AuditService.method.queryAudit;

export type AuditHistory = {
  entries: readonly AuditEntry[];
  problem?: UiText;
  loading: boolean;
  error: unknown;
  hasMore: boolean;
  loadingMore: boolean;
  loadMore: () => void;
};

export function useAuditHistory(filter: AuditFilter): AuditHistory {
  const transport = useTransport();
  const query = toAuditQuery(filter);
  const history = useInfiniteQuery({
    queryKey: createConnectQueryKey({
      schema: queryAudit,
      input: { query },
      transport,
      cardinality: "infinite",
    }),
    queryFn: ({ pageParam }) =>
      callUnaryMethod(transport, queryAudit, {
        query: { ...query, beforeId: pageParam, limit: AUDIT_PAGE_SIZE },
      }),
    initialPageParam: "",
    getNextPageParam: (page) => page.nextBeforeId || undefined,
  });
  const pages = history.data?.pages ?? [];

  return {
    entries: pages.flatMap((page) => page.entries),
    problem: pages.find((page) => page.problem)?.problem,
    loading: history.isPending,
    error: history.error,
    hasMore: history.hasNextPage,
    loadingMore: history.isFetchingNextPage,
    loadMore: () => void history.fetchNextPage(),
  };
}

const EXTENSIONS = {
  [AuditExportFormat.CSV]: "csv",
  [AuditExportFormat.JSONL]: "jsonl",
  [AuditExportFormat.UNSPECIFIED]: "csv",
} as const satisfies Record<AuditExportFormat, string>;

export function useExportAudit() {
  const transport = useTransport();
  const mutation = useMutation({
    mutationKey: workspaceMutationKeys.exportAudit(),
    mutationFn: async ({
      filter,
      format,
    }: {
      filter: AuditFilter;
      format: AuditExportFormat;
    }): Promise<ExportAuditResult | undefined> => {
      const extension = EXTENSIONS[format];
      const day = new Date().toISOString().slice(0, 10);
      const path = await dialogGateway().pickPath({
        target: "save",
        suggestedName: `audit-${day}.${extension}`,
        extensions: [extension],
      });

      if (!path) {
        return undefined;
      }
      return callUnaryMethod(transport, AuditService.method.exportAudit, {
        query: toAuditQuery(filter),
        format,
        path,
      });
    },
  });

  return {
    exportAs: mutation.mutateAsync,
    exporting: mutation.isPending,
    result: mutation.data,
    error: mutation.error,
  };
}

const NO_OPERATORS: readonly AuditTally[] = [];

export function useAuditOperators(): readonly AuditTally[] {
  const { data } = useQuery(AuditService.method.reportAudit, { query: {} });

  return data?.byOperator ?? NO_OPERATORS;
}
