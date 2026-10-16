import { useQuery } from "@connectrpc/connect-query";

import { AssistantService, type AssistantState, type UiText } from "@/shared/proto";

export type AssistantReadiness = {
  state: AssistantState | undefined;
  ready: boolean;
  problem?: UiText;
};

export function useAssistantReadiness(): AssistantReadiness {
  const { data: state } = useQuery(AssistantService.method.getAssistant, {});
  const providers = state?.providers ?? [];
  const wanted = state?.settings?.providerId ?? "";
  const candidates = wanted ? providers.filter((provider) => provider.id === wanted) : providers;
  const ready = candidates.some((provider) => provider.ready);

  return {
    state,
    ready,
    problem: ready ? undefined : candidates.at(-1)?.problem,
  };
}
