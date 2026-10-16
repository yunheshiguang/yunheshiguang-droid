import { create } from "@bufbuild/protobuf";
import { skipToken, useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import { type DeviceVendors, DeviceVendorsSchema, VendorService } from "@/shared/proto";

const NONE: DeviceVendors = create(DeviceVendorsSchema);

export type DeviceVendorsStatus = {
  vendors: DeviceVendors;
  loading: boolean;
  error: string | null;
};

export function useDeviceVendors(deviceIds: readonly string[]): DeviceVendorsStatus {
  const { data, isLoading, error } = useQuery(
    VendorService.method.identifyDevices,
    deviceIds.length > 0 ? { deviceIds: [...deviceIds] } : skipToken,
    { staleTime: Number.POSITIVE_INFINITY },
  );

  return {
    vendors: data ?? NONE,
    loading: isLoading,
    error: error == null ? null : errorMessage(error),
  };
}
