import { create, fromJson, type JsonValue } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import {
  type ConnectDeviceRequest,
  ConnectionHealth,
  ConnectionOutcome,
  type ConnectionPlan,
  ConnectionPlanSchema,
  ConnectionService,
  ConnectionType,
  DeviceDiscoverySchema,
  DiagnosticCode,
  DiscoveryService,
  type PairDeviceRequest,
  UiTextSchema,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { connectDeviceMessages } from "../messages";
import { DiscoveredDeviceList } from "./discovered-device-list";

const transports: ConnectionPlan[] = [
  create(ConnectionPlanSchema, {
    id: "core.wireless",
    title: create(UiTextSchema, {
      key: "workspace.connection.wifi.title",
      params: {},
      fallback: "",
    }),
    connection: ConnectionType.WIFI,
    health: ConnectionHealth.RECOVERING,
    diagnostic: DiagnosticCode.NONE,
    steps: [],
  }),
];

function service(instance: string, host: string, port: number, requiresPairing: boolean) {
  return {
    transportId: "core.wireless",
    label: {
      displayValue: instance,
      canonicalValue: instance,
      redactedValue: `${instance.slice(0, 4)}***`,
      piiClass: "PII_CLASS_DEVICE_IDENTIFIER",
      visibility: "VISIBILITY_MODE_SEARCHABLE",
    },
    endpoint: { host, port },
    requiresPairing,
    connection: "CONNECTION_TYPE_WIFI",
  };
}

function renderList(
  discovery: JsonValue,
  connection: Partial<ServiceImpl<typeof ConnectionService>> = {},
) {
  installFakeServices((router) => {
    router.service(DiscoveryService, {
      discoverDevices: () => fromJson(DeviceDiscoverySchema, discovery),
    });
    router.service(ConnectionService, connection);
  });
  const { wrapper } = createWorkspaceTestHarness(connectDeviceMessages);

  render(<DiscoveredDeviceList transports={transports} />, { wrapper });
}

describe("DiscoveredDeviceList", () => {
  it("connects a paired phone and pairs a phone showing its pairing code", async () => {
    const connect = vi.fn((_request: ConnectDeviceRequest) => ({
      outcome: ConnectionOutcome.CONNECTED,
    }));
    const pair = vi.fn((_request: PairDeviceRequest) => ({ outcome: ConnectionOutcome.PAIRED }));
    renderList(
      {
        endpoints: [
          service("adb-READY-1", "192.168.1.43", 37115, false),
          service("adb-PAIR-2", "192.168.1.42", 41235, true),
        ],
      },
      { connectDevice: connect, pairDevice: pair },
    );

    const [ready, pairing] = await screen.findAllByRole("listitem");
    expect(ready).toHaveTextContent("adb-***");
    expect(ready).toHaveTextContent("Wireless debugging");
    expect(ready).not.toHaveTextContent("adb-READY-1");
    expect(pairing).toHaveTextContent("Needs pairing");

    await userEvent.click(within(ready).getByRole("button", { name: "Connect adb-***" }));
    expect(connect.mock.calls[0]?.[0]).toMatchObject({
      endpoint: { host: "192.168.1.43", port: 37115 },
    });

    const pairButton = within(pairing).getByRole("button", { name: "Pair with adb-***" });
    expect(pairButton).toBeDisabled();
    await userEvent.type(within(pairing).getByLabelText("Pairing code"), "482913");
    await userEvent.click(pairButton);
    expect(pair.mock.calls[0]?.[0]).toMatchObject({
      endpoint: { host: "192.168.1.42", port: 41235 },
      pairingCode: "482913",
    });
  });

  it("names the transport whose discovery failed and keeps the others", async () => {
    renderList({
      endpoints: [],
      failures: [
        {
          transportId: "core.wireless",
          reason: {
            key: "workspace.connection.wifi.discoveryFailed",
            params: { message: "mdns unavailable" },
          },
        },
      ],
    });

    expect(
      await screen.findByText(
        "Wireless debugging: Could not look for nearby devices: mdns unavailable",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No devices to connect yet. Turn on Wireless debugging on a device"),
    ).toBeInTheDocument();
  });
});
