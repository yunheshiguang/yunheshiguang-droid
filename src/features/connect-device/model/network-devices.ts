import type { DeviceRecord } from "@/entities/device";
import { ConnectionType } from "@/shared/proto";

export function disconnectableDevices(devices: readonly DeviceRecord[]): DeviceRecord[] {
  return devices.filter(
    (device) => device.connection === ConnectionType.WIFI && /:\d+$/.test(device.id),
  );
}
