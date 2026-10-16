import type { DeviceRecordJson, DeviceVendorsJson, VendorProfileJson } from "@/shared/proto";

const PROFILES: Record<string, VendorProfileJson> = {
  "core.xiaomi": {
    id: "core.xiaomi",
    label: { key: "workspace.vendors.xiaomi.label" },
    guidance: [
      { key: "workspace.vendors.xiaomi.guidance.securitySettings" },
      { key: "workspace.vendors.xiaomi.guidance.installViaUsb" },
    ],
  },
  "core.samsung": {
    id: "core.samsung",
    label: { key: "workspace.vendors.samsung.label" },
    guidance: [{ key: "workspace.vendors.samsung.guidance.autoBlocker" }],
  },
};

function vendorOf(device: DeviceRecordJson): string | undefined {
  const model = device.model?.displayValue ?? "";

  if (/^(Redmi|Mi |POCO)/i.test(model)) {
    return "core.xiaomi";
  }

  return /^Galaxy/i.test(model) ? "core.samsung" : undefined;
}

export function identifyMockDevices(
  deviceIds: readonly string[],
  devices: readonly DeviceRecordJson[],
): DeviceVendorsJson {
  const online = new Map(
    devices
      .filter((device) => device.status === "DEVICE_STATUS_ONLINE")
      .map((device) => [device.id ?? "", device]),
  );
  const matched = deviceIds.flatMap((deviceId) => {
    const device = online.get(deviceId);
    const vendorId = device ? vendorOf(device) : undefined;

    return vendorId ? [{ deviceId, vendorId }] : [];
  });

  return {
    devices: matched,
    vendors: [...new Set(matched.map((device) => device.vendorId))].map((id) => PROFILES[id]),
  };
}
