import { describe, expect, it } from "vitest";

import { BatchActionKind } from "@/shared/proto";

import { flowIssues, hasFlowIssues } from "./flow-issues";

describe("flowIssues", () => {
  it("flags a nameless, empty flow", () => {
    const issues = flowIssues({ id: "f", name: "  ", steps: [] });

    expect(issues.name).toBe(true);
    expect(issues.empty).toBe(true);
    expect(hasFlowIssues(issues)).toBe(true);
  });

  it("flags only the steps whose action needs a value they lack", () => {
    const issues = flowIssues({
      id: "f",
      name: "Install and launch",
      steps: [
        { id: "install", kind: BatchActionKind.INSTALL_APK, value: "" },
        { id: "recover", kind: BatchActionKind.RECOVER_CONNECTION, value: "" },
        { id: "launch", kind: BatchActionKind.LAUNCH_APP, value: "com.example" },
      ],
    });

    expect([...issues.stepsMissingValue]).toEqual(["install"]);
    expect(hasFlowIssues(issues)).toBe(true);
  });

  it("accepts a complete flow", () => {
    const issues = flowIssues({
      id: "f",
      name: "Recover",
      steps: [{ id: "recover", kind: BatchActionKind.RECOVER_CONNECTION, value: "" }],
    });

    expect(hasFlowIssues(issues)).toBe(false);
  });
});
