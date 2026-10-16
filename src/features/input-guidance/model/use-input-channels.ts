import { useQuery } from "@connectrpc/connect-query";

import { InputService } from "@/shared/proto";

export function useInputChannels() {
  return useQuery(
    InputService.method.listInputChannels,
    {},
    { staleTime: Number.POSITIVE_INFINITY, select: (result) => result.channels },
  );
}
