import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent, { type UserEvent } from "@testing-library/user-event";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";

import { setLogGateway, setWindowGateway } from "@/shared/api";
import { installMockHost } from "@/shared/mock-host";
import { idleWindowGateway } from "@/shared/testing";
import { preloadModulePage } from "@/shared/workspace-module";

import { fullAssembly } from "./registry/full-assembly";
import { WorkspaceShell } from "./workspace-shell";

let logged: string[] = [];

beforeAll(async () => {
  await Promise.all(
    fullAssembly().registry.modules.map((module) => preloadModulePage(module.component)),
  );
}, 120_000);

beforeEach(() => {
  logged = [];
  installMockHost({ latencyMs: 0 });
  setLogGateway({
    error: (message) => logged.push(message),
    warn: (message) => logged.push(message),
  });
  setWindowGateway(idleWindowGateway);
  vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(800);
  vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(800);
});

afterEach(() => {
  vi.restoreAllMocks();
  expect(logged).toEqual([]);
});

async function openModule(user: UserEvent, label: string) {
  const navigation = await screen.findByRole("navigation", { name: "Modules" });
  const entry = within(navigation).getByRole("button", { name: new RegExp(`^${label}`) });

  await user.click(entry);
  await waitFor(() => expect(entry).toHaveAttribute("aria-current", "page"));

  return screen.getByRole("main");
}

async function selectOnDeviceWall(user: UserEvent, names: readonly string[]) {
  const wall = await openModule(user, "Device wall");

  for (const name of names) {
    const card = await within(wall).findByRole("article", { name });
    await user.click(within(card).getByRole("button", { name: "Select" }));
  }
}

describe("workspace flows", { timeout: 30_000 }, () => {
  it("filters the device wall, and searches hidden fields only once they are shown", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    const wall = await openModule(user, "Device wall");
    await within(wall).findByRole("article", { name: "R5******DE" });
    expect(within(wall).getAllByRole("article")).toHaveLength(4);

    await user.selectOptions(within(wall).getByLabelText("Status"), "Offline");
    await waitFor(() => expect(within(wall).getAllByRole("article")).toHaveLength(1));
    expect(within(wall).getByRole("article", { name: "0A******4E" })).toBeInTheDocument();

    await user.selectOptions(within(wall).getByLabelText("Status"), "All statuses");
    await user.type(within(wall).getByRole("textbox", { name: "Search devices" }), "galaxy");
    await waitFor(() => expect(within(wall).queryAllByRole("article")).toHaveLength(0));

    const reveal = within(wall).getByRole("button", { name: "Show sensitive data" });
    expect(reveal).toHaveAttribute("aria-pressed", "false");
    await user.click(reveal);
    expect(reveal).toHaveAttribute("aria-pressed", "true");
    expect(await within(wall).findByRole("article", { name: "RZ8N70XYZ12" })).toBeInTheDocument();
    expect(within(wall).getAllByRole("article")).toHaveLength(1);
  });

  it("runs a batch action on the devices picked on the device wall", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    await selectOnDeviceWall(user, ["R5******DE", "RZ******12"]);
    const batch = await openModule(user, "Batch");

    expect(await within(batch).findByText("2 devices")).toBeInTheDocument();
    await user.click(within(batch).getByRole("button", { name: "Recover connection" }));
    await user.click(within(batch).getByRole("button", { name: "Run" }));

    expect(await within(batch).findByText("Completed on 2 devices")).toBeInTheDocument();
    expect(within(batch).getAllByText("Succeeded")).toHaveLength(2);
  });

  it("saves a new flow and runs it on the device picked on the device wall", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    await selectOnDeviceWall(user, ["R5******DE"]);
    const automation = await openModule(user, "Automation");

    await user.click(await within(automation).findByRole("button", { name: "New flow" }));
    const editor = within(automation).getByRole("form", { name: "Edit flow" });
    await user.type(within(editor).getByLabelText("Flow name"), "Morning check");
    await user.click(within(editor).getByRole("button", { name: "Add step" }));
    await user.selectOptions(within(editor).getByLabelText("Step 1"), "RECOVER_CONNECTION");
    await user.click(within(editor).getByRole("button", { name: "Save flow" }));
    expect(await within(editor).findByRole("status")).toHaveTextContent("Saved");

    await user.selectOptions(await within(automation).findByLabelText("Flow"), "Morning check");
    expect(
      within(automation).getByText("1 device selected on the device wall"),
    ).toBeInTheDocument();
    await user.click(within(automation).getByRole("button", { name: "Run" }));

    const progress = await within(automation).findByRole("region", { name: "Run progress" });
    await waitFor(() => expect(within(progress).getByRole("listitem")).toHaveTextContent("Done"), {
      timeout: 4_000,
    });
  });

  it("runs a new shell script on the device picked on the device wall", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    await selectOnDeviceWall(user, ["R5******DE"]);
    const scripts = await openModule(user, "Scripts");

    await user.click(await within(scripts).findByRole("button", { name: "Shell" }));
    await user.click(within(scripts).getByRole("button", { name: "Run" }));

    const results = await within(scripts).findByRole("region", { name: "Run results" });
    expect(within(results).getByText("Exit code 0")).toBeInTheDocument();
    expect(
      within(results).getByRole("figure", { name: "Output from R5******DE" }),
    ).toHaveTextContent("$ getprop ro.product.model");
  });

  it("connects a network device and disconnects it from the list beside the form", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    const connect = await openModule(user, "Connect");
    const form = await within(connect).findByRole("form", { name: "Connect" });
    await user.type(within(form).getByLabelText("IP address and port"), "10.0.0.8:5555");
    await user.click(within(form).getByRole("button", { name: "Connect" }));
    expect(await within(form).findByRole("status")).toHaveTextContent("Connected to");

    await user.click(await within(connect).findByRole("button", { name: "Disconnect 10******55" }));
    await waitFor(() =>
      expect(
        within(connect).queryByRole("button", { name: "Disconnect 10******55" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("switches the whole workspace to Chinese from the settings page", async () => {
    const user = userEvent.setup();
    render(<WorkspaceShell />);

    const settings = await openModule(user, "Settings");
    await user.click(within(settings).getByRole("button", { name: "Switch to 中文" }));

    const navigation = await screen.findByRole("navigation", { name: "模块" });
    expect(within(navigation).getByRole("button", { name: /^设备墙/ })).toBeInTheDocument();
  });
});
