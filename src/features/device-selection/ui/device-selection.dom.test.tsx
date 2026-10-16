import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { createWorkspaceTestHarness } from "@/shared/testing";

import { deviceSelectionMessages } from "../messages";
import { DeviceSelectToggle } from "./device-select-toggle";
import { DeviceSelectionControls } from "./device-selection-controls";

function renderSelection(visibleIds: readonly string[] = ["a", "b"]) {
  const { wrapper } = createWorkspaceTestHarness(deviceSelectionMessages);

  return render(
    <>
      <DeviceSelectionControls availableIds={["a", "b", "c"]} visibleIds={visibleIds} />
      <DeviceSelectToggle deviceId="a" />
      <DeviceSelectToggle deviceId="b" />
    </>,
    { wrapper },
  );
}

const pressed = () => screen.queryAllByRole("button", { pressed: true });

describe("device selection", () => {
  it("toggles one card and counts it", async () => {
    const user = userEvent.setup();
    renderSelection();

    expect(screen.getByText("0 selected")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear selection" })).toBeDisabled();

    await user.click(screen.getAllByRole("button", { name: "Select" })[0]);

    expect(pressed()).toHaveLength(1);
    expect(screen.getByText("1 selected")).toBeInTheDocument();
  });

  it("selects everything shown, then clears it", async () => {
    const user = userEvent.setup();
    renderSelection();

    await user.click(screen.getByRole("button", { name: "Select all shown" }));
    expect(pressed().map((button) => button.textContent)).toEqual(["Selected", "Selected"]);
    expect(screen.getByText("2 selected")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Clear selection" }));
    expect(pressed()).toEqual([]);
    expect(screen.getByText("0 selected")).toBeInTheDocument();
  });

  it("has nothing to select when the filter hides every device", () => {
    renderSelection([]);

    expect(screen.getByRole("button", { name: "Select all shown" })).toBeDisabled();
  });
});
