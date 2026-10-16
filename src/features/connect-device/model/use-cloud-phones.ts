import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { discoveryKey, workspaceMutationKeys } from "@/shared/api";
import { CloudService } from "@/shared/proto";

const CLOUD_PHONES_POLL_INTERVAL_MS = 30_000;

export function useCloudPhones() {
  const queryClient = useQueryClient();
  const { data, isPending, error } = useQuery(
    CloudService.method.listCloudPhones,
    {},
    { refetchInterval: CLOUD_PHONES_POLL_INTERVAL_MS },
  );
  const refresh = () =>
    Promise.all(
      [createConnectQueryKey({ schema: CloudService, cardinality: undefined }), discoveryKey()].map(
        (queryKey) => queryClient.invalidateQueries({ queryKey }),
      ),
    );
  const options = { mutationKey: workspaceMutationKeys.cloudPhones(), onSuccess: refresh };
  const start = useMutation(CloudService.method.startCloudPhone, options);
  const stop = useMutation(CloudService.method.stopCloudPhone, options);
  const remove = useMutation(CloudService.method.removeCloudPhone, options);
  const register = useMutation(CloudService.method.registerCloudPhone, options);

  return {
    phones: data,
    loading: isPending,
    error,
    start: start.mutateAsync,
    stop: stop.mutateAsync,
    remove: remove.mutateAsync,
    register: register.mutateAsync,
    busy: start.isPending || stop.isPending || remove.isPending || register.isPending,
  };
}
