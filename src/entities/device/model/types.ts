import type { ConnectionType, DeviceRecord, DeviceStatus } from "@/shared/proto";

export type { ConnectionType, DeviceRecord, DeviceStatus };

export type DeviceSortKey = "rank" | "number" | "status" | "latency" | "battery";

export type DeviceFilter = {
  query: string;
  groupId: string | "all";
  tag: string | "all";
  status: DeviceStatus | "all";
  connection: ConnectionType | "all";
  revealPii: boolean;
};
