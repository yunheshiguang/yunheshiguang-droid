import {
  createConnectQueryKey,
  useMutation,
  useQuery,
  useTransport,
} from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { workspaceMutationKeys } from "@/shared/api";
import { type ApiGatewayResult, ApiGatewayService, type ApiGatewayState } from "@/shared/proto";

export type ApiGateway = {
  state: ApiGatewayState | undefined;
  update: (config: { enabled: boolean; port: number }) => Promise<ApiGatewayResult>;
  createToken: (name: string) => Promise<ApiGatewayResult>;
  revokeToken: (id: string) => Promise<ApiGatewayResult>;
  pending: boolean;
};

const getApiGateway = ApiGatewayService.method.getApiGateway;

export function useApiGateway(): ApiGateway {
  const queryClient = useQueryClient();
  const transport = useTransport();
  const { data: state } = useQuery(getApiGateway, {});
  const adopt = (result: ApiGatewayResult) => {
    if (result.state) {
      queryClient.setQueryData(
        createConnectQueryKey({
          schema: getApiGateway,
          input: {},
          transport,
          cardinality: "finite",
        }),
        result.state,
      );
    }
  };
  const options = { mutationKey: workspaceMutationKeys.apiGateway(), onSuccess: adopt };
  const update = useMutation(ApiGatewayService.method.updateApiGateway, options);
  const create = useMutation(ApiGatewayService.method.createApiToken, options);
  const revoke = useMutation(ApiGatewayService.method.revokeApiToken, options);

  return {
    state,
    update: (config) => update.mutateAsync({ config }),
    createToken: (name) => create.mutateAsync({ name }),
    revokeToken: (id) => revoke.mutateAsync({ id }),
    pending: update.isPending || create.isPending || revoke.isPending,
  };
}
