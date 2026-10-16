import { fromJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";
import { act, render, renderHook, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { useAutomationFlowActions } from "@/entities/automation";
import { flowRunMessages } from "@/entities/flow-run";
import {
  AutomationService,
  type RemoveScheduleRequest,
  type SaveScheduleRequest,
  type ScheduleStateJson,
  SchedulesSchema,
} from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { scheduleAutomationMessages } from "../messages";
import { SchedulePanel } from "./schedule-panel";

const stored: ScheduleStateJson = {
  schedule: {
    id: "s-1",
    name: "Nightly",
    flowId: "flow-1",
    target: { groupId: "shelf-a" },
    cron: "0 2 * * *",
    enabled: true,
    options: { repeat: 1 },
  },
  nextRunAt: "2026-09-27T02:00:00Z",
  lastRunId: "run-1",
  lastStatus: "RUN_STATUS_FAILED",
  lastRunAt: "2026-09-26T02:00:00Z",
  problem: { key: "workspace.automation.noDevices" },
};

async function renderPanel(automation: Partial<ServiceImpl<typeof AutomationService>>) {
  installFakeServices(createFakeDocuments().routes, (router) =>
    router.service(AutomationService, automation),
  );
  const { wrapper } = createWorkspaceTestHarness(flowRunMessages, scheduleAutomationMessages);
  const { result } = renderHook(() => useAutomationFlowActions(), { wrapper });
  await act(() => result.current.save({ id: "flow-1", name: "Morning check", steps: [] }));

  render(<SchedulePanel selectedDeviceIds={["serial-a", "serial-b"]} groups={[]} tags={[]} />, {
    wrapper,
  });
}

describe("SchedulePanel", () => {
  it("creates a schedule for the selected devices at a preset time", async () => {
    const save = vi.fn((_request: SaveScheduleRequest) => ({ accepted: true }));
    await renderPanel({ listSchedules: () => ({ schedules: [] }), saveSchedule: save });

    expect(await screen.findByText("No schedules yet")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "New schedule" }));
    const editor = screen.getByRole("form", { name: "Edit schedule" });
    expect(within(editor).getByText("2 devices selected when saved")).toBeInTheDocument();
    expect(within(editor).getByRole("button", { name: "Save" })).toBeDisabled();

    await userEvent.type(within(editor).getByLabelText("Schedule name"), "Weekday check");
    await userEvent.selectOptions(within(editor).getByLabelText("Common times"), "0 9 * * 1-5");
    await userEvent.click(within(editor).getByRole("button", { name: "Save" }));

    expect(save.mock.calls[0]?.[0]).toMatchObject({
      schedule: {
        name: "Weekday check",
        flowId: "flow-1",
        target: { target: { case: "devices", value: { deviceIds: ["serial-a", "serial-b"] } } },
        cron: "0 9 * * 1-5",
        enabled: true,
      },
    });
    expect(screen.queryByRole("form", { name: "Edit schedule" })).toBeNull();
  });

  it("lists schedules with their next run and last problem, and pauses or deletes them", async () => {
    const save = vi.fn((_request: SaveScheduleRequest) => ({ accepted: true }));
    const remove = vi.fn((_request: RemoveScheduleRequest) => ({ accepted: true }));
    await renderPanel({
      listSchedules: () => fromJson(SchedulesSchema, { schedules: [stored] }),
      saveSchedule: save,
      removeSchedule: remove,
    });

    const item = (await screen.findByText("Nightly")).closest("li");
    expect(item).not.toBeNull();
    if (!item) {
      return;
    }
    expect(within(item).getByText(/Morning check/)).toBeInTheDocument();
    expect(within(item).getByText(/^Next /)).toBeInTheDocument();
    expect(within(item).getByText("Failed")).toBeInTheDocument();
    expect(within(item).getByText("No device in the target is online")).toBeInTheDocument();

    await userEvent.click(within(item).getByRole("button", { name: "Pause" }));
    expect(save.mock.calls[0]?.[0]).toMatchObject({
      schedule: {
        id: "s-1",
        enabled: false,
        target: { target: { case: "groupId", value: "shelf-a" } },
      },
    });

    await userEvent.click(within(item).getByRole("button", { name: "Delete schedule “Nightly”" }));
    await userEvent.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "Delete" }),
    );
    expect(remove.mock.calls[0]?.[0]).toMatchObject({ id: "s-1" });
  });
});
