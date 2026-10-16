import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { createWorkspaceTestHarness } from "@/shared/testing";

import { flowRunMessages } from "../messages";
import { defaultRunOptions, type RunOptionValues } from "../model/run-options";
import { RunOptionsFields } from "./run-options-fields";

function Harness({ onChange }: { onChange: (value: RunOptionValues) => void }) {
  const [value, setValue] = useState(defaultRunOptions);

  return (
    <RunOptionsFields
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange(next);
      }}
    />
  );
}

function renderFields() {
  const onChange = vi.fn<(value: RunOptionValues) => void>();
  const { wrapper } = createWorkspaceTestHarness(flowRunMessages);
  render(<Harness onChange={onChange} />, { wrapper });
  return onChange;
}

describe("RunOptionsFields", () => {
  it("lets a field be cleared and retyped", async () => {
    const onChange = renderFields();
    const repeat = screen.getByLabelText("Times to run");

    await userEvent.clear(repeat);
    await userEvent.type(repeat, "5");

    expect(repeat).toHaveValue(5);
    expect(onChange).toHaveBeenLastCalledWith({ ...defaultRunOptions, repeat: 5 });
  });

  it("brings a value back within its limits when the field loses focus", async () => {
    const onChange = renderFields();
    const retries = screen.getByLabelText("Retries for failed devices");

    await userEvent.clear(retries);
    await userEvent.type(retries, "99");
    await userEvent.tab();

    expect(retries).toHaveValue(10);
    expect(onChange).toHaveBeenLastCalledWith({ ...defaultRunOptions, retryAttempts: 10 });
  });
});
