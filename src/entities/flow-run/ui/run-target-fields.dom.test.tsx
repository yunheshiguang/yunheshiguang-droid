import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { createWorkspaceTestHarness } from "@/shared/testing";

import { flowRunMessages } from "../messages";
import { emptyRunTarget, type RunTargetDraft } from "../model/run-target";
import { RunTargetFields } from "./run-target-fields";

function renderFields(value: RunTargetDraft, tags: readonly string[] = []) {
  const { wrapper } = createWorkspaceTestHarness(flowRunMessages);

  render(
    <RunTargetFields value={value} onChange={() => {}} groups={[]} tags={tags} devicesHint="" />,
    { wrapper },
  );
}

describe("RunTargetFields", () => {
  it("shows a saved group that is gone instead of pretending nothing is picked", () => {
    renderFields({ ...emptyRunTarget, mode: "group", groupId: "0b6f-gone" });

    expect(screen.getByLabelText("Group")).toHaveDisplayValue("A group that no longer exists");
  });

  it("names the ungrouped devices while none of them is connected", () => {
    renderFields({ ...emptyRunTarget, mode: "group", groupId: "ungrouped" });

    expect(screen.getByLabelText("Group")).toHaveDisplayValue("Ungrouped");
  });

  it("keeps showing a saved tag no connected device has", () => {
    renderFields({ ...emptyRunTarget, mode: "tag", tag: "night-shift" });

    expect(screen.getByLabelText("Tag")).toHaveDisplayValue("night-shift");
  });
});
