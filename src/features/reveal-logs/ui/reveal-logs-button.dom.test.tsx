import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { setCommandGateway, WORKSPACE_COMMANDS } from "@/shared/api";
import { createFakeCommandGateway, createWorkspaceTestHarness } from "@/shared/testing";

import { revealLogsMessages } from "../messages";
import { RevealLogsButton } from "./reveal-logs-button";

function renderButton(reveal: () => unknown) {
  setCommandGateway(createFakeCommandGateway({ [WORKSPACE_COMMANDS.revealLogDirectory]: reveal }));
  const { wrapper } = createWorkspaceTestHarness(revealLogsMessages);

  render(<RevealLogsButton />, { wrapper });
}

describe("RevealLogsButton", () => {
  it("asks the backend to open the log folder", async () => {
    const reveal = vi.fn(() => undefined);
    renderButton(reveal);

    await userEvent.click(screen.getByRole("button", { name: "Open log folder" }));

    expect(reveal).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("says why the folder could not be opened", async () => {
    renderButton(() => Promise.reject("no file manager is registered"));

    await userEvent.click(screen.getByRole("button", { name: "Open log folder" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Could not open the log folder");
    expect(alert).toHaveTextContent("no file manager is registered");
  });
});
