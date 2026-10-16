import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import { type FlowRun, FlowRunSchema, RunStatus, RunTrigger } from "@/shared/proto";

import { pickCurrentRun } from "./pick-run";

function run(id: string, flowId: string, status: FlowRun["status"]): FlowRun {
  return create(FlowRunSchema, {
    id,
    flowId,
    flowName: flowId,
    trigger: RunTrigger.MANUAL,
    actor: "main",
    status,
    deviceIds: [],
    iteration: 1,
    iterations: 1,
    steps: [],
    scheduleId: "",
  });
}

describe("pickCurrentRun", () => {
  it("follows the flow's run in progress, else its latest run", () => {
    const runs = [
      run("3", "other", RunStatus.RUNNING),
      run("2", "morning", RunStatus.FAILED),
      run("1", "morning", RunStatus.SUCCEEDED),
    ];

    expect(pickCurrentRun(runs, "morning")?.id).toBe("2");
    expect(pickCurrentRun([run("4", "morning", RunStatus.RUNNING), ...runs], "morning")?.id).toBe(
      "4",
    );
    expect(pickCurrentRun(runs, "absent")).toBeUndefined();
  });
});
