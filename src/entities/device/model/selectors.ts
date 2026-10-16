import Fuse from "fuse.js";

import type { Translate } from "@/shared/i18n";

import { deviceSearchText } from "./search-text";
import type { DeviceFilter, DeviceRecord } from "./types";

const FUSE_OPTIONS = {
  keys: ["searchText"],
  threshold: 0.3,
  ignoreLocation: true,
  minMatchCharLength: 1,
};

function matchesFacets(device: DeviceRecord, filter: DeviceFilter): boolean {
  return (
    (filter.groupId === "all" || device.groupId === filter.groupId) &&
    (filter.tag === "all" || (device.labels?.tags ?? []).includes(filter.tag)) &&
    (filter.status === "all" || device.status === filter.status) &&
    (filter.connection === "all" || device.connection === filter.connection)
  );
}

export function filterDevices(
  devices: readonly DeviceRecord[],
  filter: DeviceFilter,
  translate?: Translate,
): DeviceRecord[] {
  const filtered = devices.filter((device) => matchesFacets(device, filter));
  const query = filter.query.trim();

  if (!query) {
    return filtered;
  }

  const indexed = filtered.map((device) => ({
    device,
    searchText: deviceSearchText(device, filter.revealPii, translate),
  }));

  return new Fuse(indexed, FUSE_OPTIONS).search(query).map((result) => result.item.device);
}

export function withKnownFacets(
  filter: DeviceFilter,
  groupIds: readonly string[],
  tags: readonly string[],
): DeviceFilter {
  const groupId =
    filter.groupId === "all" || groupIds.includes(filter.groupId) ? filter.groupId : "all";
  const tag = filter.tag === "all" || tags.includes(filter.tag) ? filter.tag : "all";

  return groupId === filter.groupId && tag === filter.tag ? filter : { ...filter, groupId, tag };
}
