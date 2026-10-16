import type { PlainMessageKey } from "@/shared/i18n";
import { DeviceStatus } from "@/shared/proto";

import type { DeviceRecord, DeviceSortKey } from "./types";

type DeviceComparator = (left: DeviceRecord, right: DeviceRecord) => number;

interface DeviceSorter {
  key: DeviceSortKey;
  labelKey: PlainMessageKey;
  order: number;
  compare: DeviceComparator;
}

const statusOrder: Readonly<Partial<Record<DeviceStatus, number>>> = {
  [DeviceStatus.ONLINE]: 0,
  [DeviceStatus.BUSY]: 1,
  [DeviceStatus.IDLE]: 2,
  [DeviceStatus.WARNING]: 3,
  [DeviceStatus.OFFLINE]: 4,
};

const UNRANKED_STATUS = Number.MAX_SAFE_INTEGER;

function statusRank(status: DeviceStatus) {
  return statusOrder[status] ?? UNRANKED_STATUS;
}

function bySelectedRank(left: DeviceRecord, right: DeviceRecord) {
  return left.selectedRank - right.selectedRank;
}

const deviceSorters: readonly DeviceSorter[] = [
  {
    key: "rank",
    labelKey: "device.sort.rank",
    order: 10,
    compare: bySelectedRank,
  },
  {
    key: "number",
    labelKey: "device.sort.number",
    order: 15,
    compare: (left, right) =>
      (left.labels?.number || Number.POSITIVE_INFINITY) -
      (right.labels?.number || Number.POSITIVE_INFINITY),
  },
  {
    key: "status",
    labelKey: "device.sort.status",
    order: 20,
    compare: (left, right) => statusRank(left.status) - statusRank(right.status),
  },
  {
    key: "latency",
    labelKey: "device.sort.latency",
    order: 30,
    compare: (left, right) =>
      (left.latencyMs ?? Number.POSITIVE_INFINITY) - (right.latencyMs ?? Number.POSITIVE_INFINITY),
  },
  {
    key: "battery",
    labelKey: "device.sort.battery",
    order: 40,
    compare: (left, right) =>
      (right.batteryPercent ?? Number.NEGATIVE_INFINITY) -
      (left.batteryPercent ?? Number.NEGATIVE_INFINITY),
  },
];

export function listDeviceSorters(): readonly DeviceSorter[] {
  return [...deviceSorters].sort((left, right) => left.order - right.order);
}

export function resolveDeviceSorter(key: DeviceSortKey): DeviceSorter | undefined {
  return deviceSorters.find((sorter) => sorter.key === key) ?? deviceSorters[0];
}

export function sortDevices(devices: readonly DeviceRecord[], sortKey: DeviceSortKey) {
  const compare = resolveDeviceSorter(sortKey)?.compare ?? bySelectedRank;

  return [...devices].sort((left, right) => compare(left, right) || bySelectedRank(left, right));
}

export type { DeviceSorter };
