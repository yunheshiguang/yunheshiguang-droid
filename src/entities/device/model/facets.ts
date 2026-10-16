import { compareNames } from "@/shared/lib/text";
import { ConnectionType, DeviceStatus } from "@/shared/proto";

import type { DeviceRecord } from "./types";

export const deviceStatusFacets = [
  DeviceStatus.ONLINE,
  DeviceStatus.BUSY,
  DeviceStatus.IDLE,
  DeviceStatus.WARNING,
  DeviceStatus.OFFLINE,
] as const;

export const connectionTypeFacets = [
  ConnectionType.USB,
  ConnectionType.WIFI,
  ConnectionType.OTG,
  ConnectionType.NETWORK_SCAN,
  ConnectionType.CLOUD,
] as const;

export function collectDeviceTags(devices: readonly DeviceRecord[]): string[] {
  const tags = new Set(devices.flatMap((device) => device.labels?.tags ?? []));

  return [...tags].sort(compareNames);
}
