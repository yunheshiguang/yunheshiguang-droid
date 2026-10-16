import type {
  AdbDeviceActionResultJson,
  BatchActionKindJson,
  DeviceInputRouteJson,
  InputChannelsJson,
} from "@/shared/proto";

const YADB_LABEL = { key: "workspace.input.yadb.label" };

export function mockInputChannels(): InputChannelsJson {
  return {
    channels: [
      {
        id: "core.yadb",
        label: YADB_LABEL,
        description: { key: "workspace.input.yadb.description" },
        kinds: ["INPUT_KIND_TEXT", "INPUT_KIND_CLIPBOARD"],
        setup: [],
      },
    ],
  };
}

const INPUT_ACTIONS = new Set<BatchActionKindJson>([
  "BATCH_ACTION_KIND_INPUT_TEXT",
  "BATCH_ACTION_KIND_CLIPBOARD_WRITE",
]);

export function mockInputRoutes(
  kind: BatchActionKindJson,
  results: readonly AdbDeviceActionResultJson[],
): DeviceInputRouteJson[] {
  if (!INPUT_ACTIONS.has(kind)) {
    return [];
  }

  return results.map((result) => ({
    deviceId: result.deviceId,
    channelId: "core.yadb",
    channel: YADB_LABEL,
    failure: result.success
      ? undefined
      : { key: "workspace.deviceShell.failed", params: { message: result.message ?? "" } },
  }));
}
