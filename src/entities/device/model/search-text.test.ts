import { create, type MessageInitShape } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import type { Translate } from "@/shared/i18n";
import {
  ConnectionHealth,
  ConnectionType,
  DeviceLabelsSchema,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  PiiClass,
  type SearchableText,
  SearchableTextSchema,
  VisibilityMode,
} from "@/shared/proto";

import { deviceSearchText } from "./search-text";
import { filterDevices } from "./selectors";
import type { DeviceRecord } from "./types";

function text(
  overrides: MessageInitShape<typeof SearchableTextSchema> & { piiClass: PiiClass },
): SearchableText {
  return create(SearchableTextSchema, {
    displayValue: "",
    canonicalValue: "",
    aliases: [],
    redactedValue: "",
    visibility: VisibilityMode.SEARCHABLE,
    searchable: true,
    ...overrides,
  });
}

function device(overrides: MessageInitShape<typeof DeviceRecordSchema> = {}): DeviceRecord {
  return create(DeviceRecordSchema, {
    id: "device-1",
    status: DeviceStatus.ONLINE,
    connection: ConnectionType.USB,
    tags: [],
    health: ConnectionHealth.READY,
    diagnostic: DiagnosticCode.NONE,
    selectedRank: 0,
    groupId: "lab",
    ...overrides,
  });
}

const serial = text({
  piiClass: PiiClass.DEVICE_IDENTIFIER,
  displayValue: "R58M20ABCDE",
  canonicalValue: "r58m20abcde",
  aliases: ["r58m20abcde"],
  redactedValue: "R5******DE",
});

describe("deviceSearchText", () => {
  it("indexes the words the operator reads rather than message keys", () => {
    const words: Record<string, string> = {
      "workspace.device.note.connected": "已连接",
      "workspace.device.group.default": "未分组",
      "enums.deviceStatus.ONLINE": "在线",
    };
    const translate = ((key: string) => words[key] ?? key) as Translate;
    const listed = device({
      note: { key: "workspace.device.note.connected" },
      group: { key: "workspace.device.group.default" },
    });

    const indexed = deviceSearchText(listed, false, translate);

    expect(indexed).toContain("已连接");
    expect(indexed).toContain("未分组");
    expect(indexed).toContain("在线");
    expect(indexed).toContain("online");
    expect(indexed).not.toContain("workspace.device");
  });

  it("keeps sensitive values out of the index while they stay masked", () => {
    const indexed = deviceSearchText(device({ name: serial }), false);

    expect(indexed).not.toContain("r58m20abcde");
    expect(indexed).toContain("r5******de");
  });

  it("does not let a masked field's aliases confirm the full value", () => {
    const hits = filterDevices([device({ name: serial })], {
      query: "R58M20ABCDE",
      groupId: "all",
      tag: "all",
      status: "all",
      connection: "all",
      revealPii: false,
    });

    expect(hits).toEqual([]);
  });

  it("indexes the operator's name, number and tags even while identifiers stay masked", () => {
    const indexed = deviceSearchText(
      device({
        name: serial,
        labels: create(DeviceLabelsSchema, { number: 12, alias: "A-12", tags: ["美区"] }),
      }),
      false,
    );

    expect(indexed).toContain("a-12");
    expect(indexed).toContain("#12");
    expect(indexed).toContain("美区");
    expect(indexed).not.toContain("r58m20abcde");
  });

  it("keeps a masked note out of the index until the operator reveals it", () => {
    const note = text({
      piiClass: PiiClass.NOTE,
      displayValue: "张三的账号",
      canonicalValue: "张三的账号",
      redactedValue: "******",
    });
    const labeled = device({
      labels: create(DeviceLabelsSchema, { number: 0, alias: "", note, tags: [] }),
    });

    expect(deviceSearchText(labeled, false)).not.toContain("张三");
    expect(deviceSearchText(labeled, true)).toContain("张三");
  });

  it("indexes the aliases of a masked field once the operator reveals it", () => {
    const withAlias = { ...serial, aliases: ["lab-bench-7"] };

    expect(deviceSearchText(device({ name: withAlias }), false)).not.toContain("lab-bench-7");
    expect(deviceSearchText(device({ name: withAlias }), true)).toContain("lab-bench-7");
  });

  it("indexes sensitive values once the operator reveals them", () => {
    const indexed = deviceSearchText(device({ name: serial }), true);

    expect(indexed).toContain("r58m20abcde");
  });

  it("always indexes fields the backend marked as carrying no personal data", () => {
    const model = text({
      piiClass: PiiClass.NONE,
      displayValue: "Pixel 8",
      canonicalValue: "pixel-8",
    });

    expect(deviceSearchText(device({ model }), false)).toContain("pixel 8");
  });

  it("skips fields the backend excluded from search entirely", () => {
    const secret = text({
      piiClass: PiiClass.ACCOUNT,
      displayValue: "operator@example.com",
      redactedValue: "o…@example.com",
      searchable: false,
    });

    const indexed = deviceSearchText(device({ name: secret }), true);

    expect(indexed).not.toContain("operator@example.com");
    expect(indexed).not.toContain("example.com");
  });

  it("indexes enum values so a status can be typed into the search box", () => {
    const indexed = deviceSearchText(device({ status: DeviceStatus.OFFLINE }), false);

    expect(indexed).toContain("offline");
  });
});
