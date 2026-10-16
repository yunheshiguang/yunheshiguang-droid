import { DeviceStatus } from "@/shared/proto";

import type { DeviceRecord } from "./types";

const ONLINE_STATUSES = new Set<DeviceStatus>([
  DeviceStatus.ONLINE,
  DeviceStatus.BUSY,
  DeviceStatus.IDLE,
]);

export function isDeviceOnline(device: Pick<DeviceRecord, "status">): boolean {
  return ONLINE_STATUSES.has(device.status);
}

export function countOnlineDevices(devices: readonly DeviceRecord[]): number {
  return devices.reduce((total, device) => total + (isDeviceOnline(device) ? 1 : 0), 0);
}
