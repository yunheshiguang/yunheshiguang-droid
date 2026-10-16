import { create } from "@bufbuild/protobuf";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { DeviceRecord } from "@/entities/device";
import {
  ConnectionHealth,
  ConnectionType,
  DeviceRecordSchema,
  DeviceStatus,
  DeviceVendorsSchema,
  DiagnosticCode,
  type IdentifyDevicesRequest,
  PiiClass,
  SearchableTextSchema,
  VendorService,
  VisibilityMode,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { vendorGuidanceMessages } from "../messages";
import { VendorGuidanceList } from "./vendor-guidance-list";

type Identify = (
  request: IdentifyDevicesRequest,
) => ReturnType<typeof create<typeof DeviceVendorsSchema>>;

function serveVendors(identify: Identify) {
  installFakeServices((router) => router.service(VendorService, { identifyDevices: identify }));
}

function device(id: string, status: DeviceRecord["status"]): DeviceRecord {
  return create(DeviceRecordSchema, {
    id,
    name: create(SearchableTextSchema, {
      displayValue: id,
      canonicalValue: id,
      redactedValue: `${id.slice(0, 2)}***`,
      aliases: [],
      piiClass: PiiClass.DEVICE_IDENTIFIER,
      visibility: VisibilityMode.SEARCHABLE,
      searchable: true,
    }),
    status,
    connection: ConnectionType.USB,
    tags: [],
    health: ConnectionHealth.READY,
    diagnostic: DiagnosticCode.NONE,
    selectedRank: 0,
    groupId: "",
  });
}

describe("VendorGuidanceList", () => {
  it("asks only about online devices and shows each vendor's steps for them", async () => {
    const identify = vi.fn<Identify>(() =>
      create(DeviceVendorsSchema, {
        devices: [{ deviceId: "RM-1", vendorId: "core.xiaomi" }],
        vendors: [
          {
            id: "core.xiaomi",
            label: { key: "workspace.vendors.xiaomi.label" },
            guidance: [
              { key: "workspace.vendors.xiaomi.guidance.securitySettings" },
              { key: "workspace.vendors.xiaomi.guidance.installViaUsb" },
            ],
          },
        ],
      }),
    );
    serveVendors(identify);
    const { wrapper } = createWorkspaceTestHarness(vendorGuidanceMessages);

    render(
      <VendorGuidanceList
        devices={[
          device("RM-1", DeviceStatus.ONLINE),
          device("PX-2", DeviceStatus.ONLINE),
          device("MI-3", DeviceStatus.OFFLINE),
        ]}
      />,
      { wrapper },
    );

    const xiaomi = await screen.findByRole("region", { name: "Xiaomi, Redmi, POCO" });
    expect(within(xiaomi).getByText("Devices: RM***")).toBeInTheDocument();
    expect(
      within(xiaomi)
        .getAllByRole("listitem")
        .map((step) => step.textContent),
    ).toEqual([
      "In Developer options, turn on “USB debugging (Security settings)”. This needs a SIM card and a signed-in Xiaomi account",
      "Turn on “Install via USB” to install apps over USB",
    ]);
    expect(identify.mock.calls.map(([request]) => request.deviceIds)).toEqual([["RM-1", "PX-2"]]);
  });

  it("does not ask the backend when nothing is online", () => {
    const identify = vi.fn<Identify>(() => create(DeviceVendorsSchema));
    serveVendors(identify);
    const { wrapper } = createWorkspaceTestHarness(vendorGuidanceMessages);

    render(<VendorGuidanceList devices={[device("MI-3", DeviceStatus.OFFLINE)]} />, { wrapper });

    expect(screen.getByText("No connected device needs vendor settings")).toBeInTheDocument();
    expect(identify).not.toHaveBeenCalled();
  });
});
