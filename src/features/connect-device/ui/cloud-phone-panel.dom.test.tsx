import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import {
  type CloudPhoneRequest,
  CloudPhoneStatus,
  CloudService,
  type RegisterCloudPhoneRequest,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { connectDeviceMessages } from "../messages";
import { CloudPhonePanel } from "./cloud-phone-panel";

const providerCopy = {
  "zh-CN": { test: { redroid: "Redroid" } },
  "en-US": { test: { redroid: "Redroid" } },
};

const phones = {
  providers: [
    { id: "test.redroid", label: { key: "test.redroid" }, canStart: true, canStop: true },
    {
      id: "core.registered",
      label: { key: "workspace.cloud.registered.label" },
      canRegister: true,
    },
  ],
  phones: [
    {
      providerId: "test.redroid",
      id: "c1",
      name: "redroid-1",
      status: CloudPhoneStatus.STOPPED,
      endpoint: { host: "10.0.0.5", port: 40001 },
    },
    {
      providerId: "core.registered",
      id: "r1",
      name: "Cloud 01",
      endpoint: { host: "10.20.0.11", port: 5555 },
    },
  ],
  problems: [{ providerId: "test.redroid", problem: { key: "workspace.cloud.timedOut" } }],
};

describe("CloudPhonePanel", () => {
  it("starts phones where the provider can and registers new ones by address", async () => {
    const start = vi.fn((_request: CloudPhoneRequest) => ({ accepted: true }));
    const register = vi.fn((_request: RegisterCloudPhoneRequest) => ({ accepted: true }));
    installFakeServices((router) =>
      router.service(CloudService, {
        listCloudPhones: () => phones,
        startCloudPhone: start,
        registerCloudPhone: register,
      }),
    );
    const { wrapper } = createWorkspaceTestHarness(connectDeviceMessages, providerCopy);
    render(<CloudPhonePanel />, { wrapper });

    const [redroid, registered] = within(
      await screen.findByRole("list", { name: "Cloud phones" }),
    ).getAllByRole("listitem");
    expect(redroid).toHaveTextContent("redroid-1");
    expect(redroid).toHaveTextContent("10.0.0.5:40001");
    expect(redroid).toHaveTextContent("Stopped");
    expect(registered).toHaveTextContent("Unknown");
    expect(within(registered).queryByRole("button", { name: "Start" })).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Could not list cloud phones from Redroid: The cloud phone service timed out",
      ),
    ).toBeInTheDocument();

    await userEvent.click(within(redroid).getByRole("button", { name: "Start" }));
    expect(start.mock.calls[0]?.[0]).toMatchObject({
      providerId: "test.redroid",
      phoneId: "c1",
    });

    await userEvent.type(screen.getByLabelText("Name"), "Cloud 02");
    await userEvent.type(screen.getByLabelText("Host"), "10.20.0.12");
    await userEvent.click(screen.getByRole("button", { name: "Register" }));
    expect(register.mock.calls[0]?.[0]).toMatchObject({
      name: "Cloud 02",
      endpoint: { host: "10.20.0.12", port: 5555 },
    });
    expect(await screen.findByText("Done")).toBeInTheDocument();
  });

  it("holds a registration with a port adb cannot use, and says why", async () => {
    const register = vi.fn((_request: RegisterCloudPhoneRequest) => ({ accepted: true }));
    installFakeServices((router) =>
      router.service(CloudService, { listCloudPhones: () => phones, registerCloudPhone: register }),
    );
    const { wrapper } = createWorkspaceTestHarness(connectDeviceMessages, providerCopy);
    render(<CloudPhonePanel />, { wrapper });

    await userEvent.type(await screen.findByLabelText("Name"), "Cloud 03");
    await userEvent.type(screen.getByLabelText("Host"), "10.20.0.13");
    await userEvent.clear(screen.getByLabelText("Port"));
    await userEvent.type(screen.getByLabelText("Port"), "70000");

    expect(screen.getByLabelText("Port")).toHaveAccessibleDescription(
      "Enter a whole number from 1 to 65535",
    );
    expect(screen.getByRole("button", { name: "Register" })).toBeDisabled();
    expect(register).not.toHaveBeenCalled();
  });
});
