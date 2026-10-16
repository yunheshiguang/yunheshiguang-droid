import type {
  AndroidPlatformToolsStateJson,
  ConnectionTypeJson,
  DeviceRecordJson,
  DeviceStatusJson,
  PiiClassJson,
  SearchableTextJson,
} from "@/shared/proto";

function redact(value: string): string {
  if (value.length <= 4) {
    return `${value.charAt(0)}***`;
  }

  return `${value.slice(0, 2)}${"*".repeat(Math.min(value.length - 4, 6))}${value.slice(-2)}`;
}

function searchable(value: string, piiClass: PiiClassJson): SearchableTextJson {
  return {
    displayValue: value,
    canonicalValue: value.toLowerCase(),
    redactedValue: redact(value),
    piiClass,
    visibility: "VISIBILITY_MODE_SEARCHABLE",
    searchable: true,
  };
}

export function mockDevice(
  id: string,
  model: string,
  connection: ConnectionTypeJson,
  status: DeviceStatusJson = "DEVICE_STATUS_ONLINE",
): DeviceRecordJson {
  const online = status === "DEVICE_STATUS_ONLINE";

  return {
    id,
    name: searchable(id, "PII_CLASS_DEVICE_IDENTIFIER"),
    model: searchable(model, "PII_CLASS_DEVICE_MODEL"),
    status,
    connection,
    note: online
      ? { key: "workspace.device.note.connected" }
      : { key: "workspace.device.note.offline" },
    tags: [],
    health: online ? "CONNECTION_HEALTH_READY" : "CONNECTION_HEALTH_BLOCKED",
    diagnostic: online ? "DIAGNOSTIC_CODE_NONE" : "DIAGNOSTIC_CODE_DEVICE_OFFLINE",
  };
}

export function initialMockDevices(): DeviceRecordJson[] {
  return [
    mockDevice("R58M20ABCDE", "Pixel 8", "CONNECTION_TYPE_USB"),
    mockDevice("RZ8N70XYZ12", "Galaxy S23", "CONNECTION_TYPE_USB"),
    mockDevice("192.168.1.20:5555", "Redmi Note 12", "CONNECTION_TYPE_WIFI"),
    mockDevice("0A1B2C3D4E", "Mi 11", "CONNECTION_TYPE_USB", "DEVICE_STATUS_OFFLINE"),
  ];
}

export const MOCK_ADB_PATH = "/mock/platform-tools/adb";
export const MOCK_FASTBOOT_PATH = "/mock/platform-tools/fastboot";

export function mockPlatformTools(): AndroidPlatformToolsStateJson {
  return {
    config: { source: "ANDROID_PLATFORM_TOOLS_SOURCE_APP_PRIVATE" },
    adbPath: MOCK_ADB_PATH,
    fastbootPath: MOCK_FASTBOOT_PATH,
    binaries: [
      {
        kind: "ANDROID_PLATFORM_TOOLS_BINARY_KIND_ADB",
        resolvedPath: MOCK_ADB_PATH,
        available: true,
      },
      {
        kind: "ANDROID_PLATFORM_TOOLS_BINARY_KIND_FASTBOOT",
        resolvedPath: MOCK_FASTBOOT_PATH,
        available: true,
      },
    ],
    health: "CONNECTION_HEALTH_READY",
  };
}
