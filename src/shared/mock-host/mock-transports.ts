import type {
  ConnectionPlanJson,
  ConnectionTypeJson,
  DeviceDiscoveryJson,
  DiscoveredEndpointJson,
} from "@/shared/proto";

function plan(id: string, connection: ConnectionTypeJson, copy: string): ConnectionPlanJson {
  return {
    id,
    title: { key: `workspace.connection.${copy}.title` },
    connection,
    health: "CONNECTION_HEALTH_RECOVERING",
    diagnostic: "DIAGNOSTIC_CODE_NONE",
    steps: [1, 2, 3, 4].map((step) => ({ key: `workspace.connection.${copy}.step${step}` })),
  };
}

export function mockConnectionPlans(): ConnectionPlanJson[] {
  return [
    plan("core.usb", "CONNECTION_TYPE_USB", "usbFirst"),
    plan("core.wireless", "CONNECTION_TYPE_WIFI", "wifi"),
    plan("core.otg", "CONNECTION_TYPE_OTG", "otg"),
  ];
}

function wireless(
  instance: string,
  host: string,
  port: number,
  requiresPairing: boolean,
): DiscoveredEndpointJson {
  return {
    transportId: "core.wireless",
    label: {
      displayValue: instance,
      canonicalValue: instance.toLowerCase(),
      redactedValue: `${instance.slice(0, 4)}******${instance.slice(-2)}`,
      piiClass: "PII_CLASS_DEVICE_IDENTIFIER",
      visibility: "VISIBILITY_MODE_SEARCHABLE",
      searchable: true,
    },
    endpoint: { host, port },
    requiresPairing,
    connection: "CONNECTION_TYPE_WIFI",
  };
}

export function mockDiscovery(attached: ReadonlySet<string>): DeviceDiscoveryJson {
  return {
    endpoints: [
      wireless("adb-R5CT40PAIR-q1w2e3", "192.168.1.42", 41235, true),
      wireless("adb-R5CT41READY-a1s2d3", "192.168.1.43", 37115, false),
    ].filter(({ endpoint }) => !attached.has(`${endpoint?.host}:${endpoint?.port}`)),
    failures: [],
  };
}
