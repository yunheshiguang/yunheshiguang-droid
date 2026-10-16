import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { mirrorDeviceMessages } from "@/features/mirror-device";
import { installMockHost } from "@/shared/mock-host";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { mirrorPageMessages } from "../messages";
import { MirrorPage } from "./mirror-page";

beforeEach(() => {
  installMockHost({ latencyMs: 0 });
});

function renderPage() {
  const { wrapper } = createWorkspaceTestHarness(mirrorDeviceMessages, mirrorPageMessages);

  return render(<MirrorPage />, { wrapper });
}

describe("MirrorPage", () => {
  it("downloads scrcpy first, then opens and closes a device's mirroring window", async () => {
    const user = userEvent.setup();
    renderPage();

    expect(await screen.findByText("scrcpy not found")).toBeInTheDocument();
    const pixel = await screen.findByRole("listitem", { name: "R5******DE" });
    expect(within(pixel).getByRole("button", { name: "Mirror" })).toBeDisabled();

    await user.click(screen.getByRole("button", { name: "Download the official scrcpy" }));
    expect(await screen.findByText("v5.0 installed")).toBeInTheDocument();
    expect(screen.queryByText("scrcpy not found")).not.toBeInTheDocument();

    await user.click(within(pixel).getByRole("button", { name: "Mirror" }));
    expect(await within(pixel).findByText("Mirroring")).toBeInTheDocument();

    await user.click(within(pixel).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(within(pixel).queryByText("Mirroring")).not.toBeInTheDocument());
  });

  it("does not offer to mirror a device that is offline", async () => {
    const user = userEvent.setup();
    renderPage();

    await screen.findByText("scrcpy not found");
    await user.click(screen.getByRole("button", { name: "Download the official scrcpy" }));
    await screen.findByText("v5.0 installed");

    const offline = screen.getByRole("listitem", { name: "0A******4E" });
    expect(within(offline).getByRole("button", { name: "Mirror" })).toBeDisabled();
  });

  it("saves the mirroring options", async () => {
    const user = userEvent.setup();
    renderPage();

    await screen.findByText("scrcpy not found");
    const audio = screen.getByRole("checkbox", { name: "Audio (Android 11 and later)" });
    expect(screen.getByRole("button", { name: "Save settings" })).toBeDisabled();

    await user.click(audio);
    await user.click(screen.getByRole("button", { name: "Save settings" }));

    await waitFor(() =>
      expect(
        screen.getByRole("checkbox", { name: "Audio (Android 11 and later)" }),
      ).not.toBeChecked(),
    );
    expect(screen.getByRole("button", { name: "Save settings" })).toBeDisabled();
  });
});
