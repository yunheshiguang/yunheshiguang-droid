import { create } from "@bufbuild/protobuf";
import { Code, ConnectError, type ServiceImpl } from "@connectrpc/connect";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { DeviceRecord } from "@/entities/device";
import {
  type ConnectDeviceRequest,
  ConnectionHealth,
  ConnectionOutcome,
  ConnectionService,
  ConnectionType,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  type DisconnectDeviceRequest,
  type PairDeviceRequest,
  PiiClass,
  SearchableTextSchema,
  VisibilityMode,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { connectDeviceMessages } from "../messages";
import { ConnectDevicePanel } from "./connect-device-panel";
import { NetworkDeviceList } from "./network-device-list";

function renderWith(
  ui: React.ReactElement,
  connection: Partial<ServiceImpl<typeof ConnectionService>>,
) {
  installFakeServices((router) => router.service(ConnectionService, connection));
  const { wrapper } = createWorkspaceTestHarness(connectDeviceMessages);

  render(ui, { wrapper });
}

const connected = {
  outcome: ConnectionOutcome.CONNECTED,
  deviceId: "192.168.1.20:5555",
  detail: {
    key: "workspace.connection.result.connected",
    params: { device: "192.168.1.20:5555" },
  },
};

describe("ConnectDevicePanel", () => {
  it("connects to the typed address on the default port", async () => {
    const connect = vi.fn((_request: ConnectDeviceRequest) => connected);
    renderWith(<ConnectDevicePanel />, { connectDevice: connect });

    const form = screen.getByRole("form", { name: "Connect" });
    await userEvent.type(within(form).getByLabelText("IP address and port"), "192.168.1.20");
    await userEvent.click(within(form).getByRole("button", { name: "Connect" }));

    expect(await within(form).findByRole("status")).toHaveTextContent(
      "Connected Connected to 192.168.1.20:5555",
    );
    expect(connect.mock.calls[0]?.[0]).toMatchObject({
      endpoint: { host: "192.168.1.20", port: 5555 },
    });
  });

  it("explains a malformed address without calling the backend", async () => {
    const connect = vi.fn((_request: ConnectDeviceRequest) => connected);
    renderWith(<ConnectDevicePanel />, { connectDevice: connect });

    const form = screen.getByRole("form", { name: "Connect" });
    const address = within(form).getByLabelText("IP address and port");
    await userEvent.type(address, "host/path");
    await userEvent.click(within(form).getByRole("button", { name: "Connect" }));

    expect(address).toHaveAttribute("aria-invalid", "true");
    expect(address).toHaveAccessibleDescription("Enter an address like 192.168.1.20:5555");
    expect(connect).not.toHaveBeenCalled();
  });

  it("requires the pairing port and reports a rejected code", async () => {
    const pair = vi.fn((_request: PairDeviceRequest) => ({
      outcome: ConnectionOutcome.REJECTED,
      detail: { key: "workspace.connection.result.pairingRejected" },
    }));
    renderWith(<ConnectDevicePanel />, { pairDevice: pair });

    const form = screen.getByRole("form", { name: "Pair (Android 11 and later)" });
    const address = within(form).getByLabelText("Pairing address");
    await userEvent.type(address, "192.168.1.20");
    await userEvent.type(within(form).getByLabelText("Pairing code"), "123456");
    await userEvent.click(within(form).getByRole("button", { name: "Pair" }));

    expect(address).toHaveAttribute("aria-invalid", "true");
    expect(pair).not.toHaveBeenCalled();

    await userEvent.type(address, ":37123");
    await userEvent.click(within(form).getByRole("button", { name: "Pair" }));

    expect(await within(form).findByRole("status")).toHaveTextContent(
      "Refused Pairing failed. Check the pairing code, then try again",
    );
    expect(pair.mock.calls[0]?.[0]).toMatchObject({
      endpoint: { host: "192.168.1.20", port: 37123 },
      pairingCode: "123456",
    });
  });

  it("shows a request that never reached the backend as an error", async () => {
    renderWith(<ConnectDevicePanel />, {
      connectDevice: () => Promise.reject(new ConnectError("ipc closed", Code.Unavailable)),
    });

    const form = screen.getByRole("form", { name: "Connect" });
    await userEvent.type(within(form).getByLabelText("IP address and port"), "10.0.0.2:5555");
    await userEvent.click(within(form).getByRole("button", { name: "Connect" }));

    const alert = await within(form).findByRole("alert");
    expect(alert).toHaveTextContent("Request failed");
    expect(alert).toHaveTextContent("ipc closed");
  });
});

function device(id: string, connection: DeviceRecord["connection"], name: string): DeviceRecord {
  const text = (value: string) =>
    create(SearchableTextSchema, {
      displayValue: value,
      canonicalValue: value.toLowerCase(),
      redactedValue: `${value.slice(0, 2)}***`,
      piiClass: PiiClass.DEVICE_IDENTIFIER,
      visibility: VisibilityMode.SEARCHABLE,
      searchable: true,
    });

  return create(DeviceRecordSchema, {
    id,
    name: text(name),
    model: text("Pixel 8"),
    status: DeviceStatus.ONLINE,
    connection,
    tags: [],
    health: ConnectionHealth.READY,
    diagnostic: DiagnosticCode.NONE,
    selectedRank: 0,
    groupId: "",
  });
}

describe("NetworkDeviceList", () => {
  const devices = [
    device("192.168.1.20:5555", ConnectionType.WIFI, "Lab phone"),
    device("R58M20ABCDE", ConnectionType.USB, "Desk phone"),
    device("emulator-5554", ConnectionType.WIFI, "Emulator"),
  ];

  it("lists only network devices, masked, and disconnects the chosen one", async () => {
    const disconnect = vi.fn((_request: DisconnectDeviceRequest) => ({
      outcome: ConnectionOutcome.DISCONNECTED,
      deviceId: "192.168.1.20:5555",
      detail: {
        key: "workspace.connection.result.disconnected",
        params: { device: "192.168.1.20:5555" },
      },
    }));
    renderWith(<NetworkDeviceList devices={devices} />, { disconnectDevice: disconnect });

    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(1);
    expect(items[0]).toHaveTextContent("La***");
    expect(items[0]).not.toHaveTextContent("192.168.1.20");

    await userEvent.click(screen.getByRole("button", { name: "Disconnect La***" }));

    expect(await screen.findByRole("status")).toHaveTextContent("Disconnected");
    expect(disconnect.mock.calls[0]?.[0]).toMatchObject({ deviceId: "192.168.1.20:5555" });
  });

  it("says so when nothing is attached over the network", () => {
    renderWith(<NetworkDeviceList devices={[devices[1]]} />, {});

    expect(screen.getByText("No network devices yet")).toBeInTheDocument();
  });
});
