import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import {
  ConnectionHealth,
  ConnectionType,
  DeviceLabelsSchema,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
} from "@/shared/proto";

import { collectDeviceTags } from "./facets";
import { filterDevices, withKnownFacets } from "./selectors";
import { sortDevices } from "./sorters";
import type { DeviceFilter, DeviceRecord } from "./types";

function device(id: string, labels?: DeviceRecord["labels"]): DeviceRecord {
  return create(DeviceRecordSchema, {
    id,
    status: DeviceStatus.ONLINE,
    connection: ConnectionType.USB,
    tags: [],
    health: ConnectionHealth.READY,
    diagnostic: DiagnosticCode.NONE,
    selectedRank: Number(id.replace(/\D/g, "")) || 0,
    groupId: "ungrouped",
    labels,
  });
}

const all: DeviceFilter = {
  query: "",
  groupId: "all",
  tag: "all",
  status: "all",
  connection: "all",
  revealPii: false,
};

const devices = [
  device("d1", create(DeviceLabelsSchema, { number: 3, alias: "", tags: ["美区", "TikTok"] })),
  device("d2"),
  device("d3", create(DeviceLabelsSchema, { number: 1, alias: "", tags: ["TikTok"] })),
];

describe("device labels", () => {
  it("lists every tag once, in order", () => {
    expect(collectDeviceTags(devices)).toEqual(["TikTok", "美区"]);
  });

  it("narrows the wall to one tag", () => {
    const hits = filterDevices(devices, { ...all, tag: "美区" });

    expect(hits.map((item) => item.id)).toEqual(["d1"]);
  });

  it("sorts numbered devices first, by number", () => {
    expect(sortDevices(devices, "number").map((item) => item.id)).toEqual(["d3", "d1", "d2"]);
  });
});

describe("withKnownFacets", () => {
  it("lets go of a group or tag that no longer exists", () => {
    const filter = { ...all, groupId: "gone", tag: "old" };

    expect(withKnownFacets(filter, ["shelf-a"], ["美区"])).toEqual({
      ...filter,
      groupId: "all",
      tag: "all",
    });
  });

  it("keeps the same filter while its group and tag still exist", () => {
    const filter = { ...all, groupId: "shelf-a", tag: "美区" };

    expect(withKnownFacets(filter, ["shelf-a"], ["美区"])).toBe(filter);
  });
});
