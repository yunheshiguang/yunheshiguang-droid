import { fromJson, type JsonValue, toJson } from "@bufbuild/protobuf";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { flowRunMessages } from "@/entities/flow-run";
import {
  AutomationService,
  type FlowRunJson,
  FlowRunsSchema,
  StartFlowRunRequestSchema,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { runAutomationMessages } from "../messages";
import { RunHistory } from "./run-history";

const finished: FlowRunJson = {
  id: "run-2",
  flowId: "morning",
  flowName: "Morning",
  trigger: "RUN_TRIGGER_SCHEDULE",
  actor: "schedule:Morning",
  status: "RUN_STATUS_FAILED",
  deviceIds: ["a", "b"],
  target: { groupId: "shelf-a" },
  options: { repeat: 2, retryAttempts: 1 },
  operator: { id: "alice", name: "Alice", roles: ["operator"] },
};

const legacy: FlowRunJson = {
  id: "run-1",
  flowId: "evening",
  flowName: "Evening",
  trigger: "RUN_TRIGGER_MANUAL",
  status: "RUN_STATUS_SUCCEEDED",
  deviceIds: ["a"],
};

describe("RunHistory", () => {
  it("runs a finished run again with the target and options it was started with", async () => {
    const started: JsonValue[] = [];
    installFakeServices((router) =>
      router.service(AutomationService, {
        listFlowRuns: () => fromJson(FlowRunsSchema, { runs: [finished, legacy] }),
        startFlowRun: (request) => {
          started.push(toJson(StartFlowRunRequestSchema, request));
          return { accepted: true };
        },
      }),
    );
    const { wrapper } = createWorkspaceTestHarness(runAutomationMessages, flowRunMessages);
    render(<RunHistory />, { wrapper });

    const [again, old] = await screen.findAllByRole("listitem");
    expect(again).toHaveTextContent("by Alice");
    expect(within(old).queryByRole("button", { name: /again/ })).not.toBeInTheDocument();

    await userEvent.click(
      within(again).getByRole("button", {
        name: "Run “Morning” again with the same settings",
      }),
    );

    expect(started[0]).toMatchObject({
      flowId: "morning",
      target: { groupId: "shelf-a" },
      options: { repeat: 2, retryAttempts: 1 },
    });
  });
});
