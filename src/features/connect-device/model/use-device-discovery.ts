import { create } from "@bufbuild/protobuf";
import { useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import { type DeviceDiscovery, DeviceDiscoverySchema, DiscoveryService } from "@/shared/proto";

const DISCOVERY_POLL_INTERVAL_MS = 5_000;

const NOTHING: DeviceDiscovery = create(DeviceDiscoverySchema);

export type DeviceDiscoveryStatus = {
  discovery: DeviceDiscovery;
  loading: boolean;
  error: string | null;
};

export function useDeviceDiscovery(): DeviceDiscoveryStatus {
  const { data, isPending, error } = useQuery(
    DiscoveryService.method.discoverDevices,
    {},
    { refetchInterval: DISCOVERY_POLL_INTERVAL_MS },
  );

  return {
    discovery: data ?? NOTHING,
    loading: isPending,
    error: error == null ? null : errorMessage(error),
  };
}
