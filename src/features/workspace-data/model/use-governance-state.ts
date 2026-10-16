import { useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import { GovernanceService, type GovernanceState, type Operator } from "@/shared/proto";

const GOVERNANCE_POLL_INTERVAL_MS = 15_000;
const NO_CAPABILITIES: readonly string[] = [];
const getGovernanceState = GovernanceService.method.getGovernanceState;

export type GovernanceStatus = {
  state?: GovernanceState;
  loading: boolean;
  error: string | null;
};

export function useGovernanceState(): GovernanceStatus {
  const { data, isPending, error } = useQuery(
    getGovernanceState,
    {},
    { refetchInterval: GOVERNANCE_POLL_INTERVAL_MS },
  );

  return { state: data, loading: isPending, error: error == null ? null : errorMessage(error) };
}

const selectCapabilities = (state: GovernanceState) => state.grantedCapabilities;

export function useGrantedCapabilities(): readonly string[] {
  return (
    useQuery(getGovernanceState, {}, { refetchInterval: false, select: selectCapabilities }).data ??
    NO_CAPABILITIES
  );
}

const selectOperator = (state: GovernanceState) => state.operator;

export function useCurrentOperator(): Operator | undefined {
  return useQuery(getGovernanceState, {}, { refetchInterval: false, select: selectOperator }).data;
}
