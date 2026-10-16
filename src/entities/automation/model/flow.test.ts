import { describe, expect, it } from "vitest";

import { BatchActionKind } from "@/shared/proto";

import {
  type AutomationStep,
  createAutomationFlow,
  createAutomationStep,
  moveAutomationStep,
  readAutomationFlow,
  sortAutomationFlows,
} from "./flow";
import { readFlowDocuments, toFlowDocumentBody } from "./flow-documents";

const steps: AutomationStep[] = [
  { id: "a", kind: BatchActionKind.INSTALL_APK, value: "/tmp/app.apk" },
  { id: "b", kind: BatchActionKind.LAUNCH_APP, value: "com.example" },
  { id: "c", kind: BatchActionKind.SCREENSHOT, value: "/tmp/shots" },
];

describe("automation flow model", () => {
  it("creates flows and steps with their own ids", () => {
    const flow = createAutomationFlow("Morning check");
    const step = createAutomationStep(BatchActionKind.INPUT_TEXT, "hello");

    expect(flow).toMatchObject({ name: "Morning check", steps: [] });
    expect(step).toMatchObject({ kind: BatchActionKind.INPUT_TEXT, value: "hello" });
    expect(createAutomationStep(BatchActionKind.INPUT_TEXT).id).not.toBe(step.id);
  });

  it("swaps a step with its neighbour and stops at either end", () => {
    expect(moveAutomationStep(steps, 1, -1).map((step) => step.id)).toEqual(["b", "a", "c"]);
    expect(moveAutomationStep(steps, 1, 1).map((step) => step.id)).toEqual(["a", "c", "b"]);
    expect(moveAutomationStep(steps, 0, -1)).toBe(steps);
    expect(moveAutomationStep(steps, 2, 1)).toBe(steps);
  });

  it("reads flows in the contract format and the earlier format, and rejects broken ones", () => {
    const contract = JSON.stringify({
      name: "Works",
      steps: steps.map((step) => ({
        ...step,
        kind: `BATCH_ACTION_KIND_${BatchActionKind[step.kind]}`,
      })),
    });

    expect(readAutomationFlow("ok", contract)).toEqual({ id: "ok", name: "Works", steps });
    const earlier = steps.map((step) => ({ ...step, kind: BatchActionKind[step.kind] }));
    expect(
      readAutomationFlow("old", JSON.stringify({ version: 1, name: "Old", steps: earlier })),
    ).toEqual({
      id: "old",
      name: "Old",
      steps,
    });
    expect(readAutomationFlow("x", JSON.stringify({ steps }))).toBeUndefined();
    expect(
      readAutomationFlow("x", JSON.stringify({ name: "Broken", steps: [{ id: "s" }] })),
    ).toBeUndefined();
    expect(readAutomationFlow("x", "not a flow")).toBeUndefined();
  });

  it("reads flows written without empty fields or with numeric kinds, as the backend does", () => {
    expect(readAutomationFlow("bare", JSON.stringify({ name: "Bare" }))).toEqual({
      id: "bare",
      name: "Bare",
      steps: [],
    });
    expect(
      readAutomationFlow(
        "packed",
        JSON.stringify({
          name: "Packed",
          steps: [{ kind: BatchActionKind.LAUNCH_APP, value: "a" }],
        }),
      ),
    ).toEqual({
      id: "packed",
      name: "Packed",
      steps: [{ id: "step-1", kind: BatchActionKind.LAUNCH_APP, value: "a" }],
    });
  });

  it("sorts flows by name, then by id", () => {
    const sorted = sortAutomationFlows([
      { id: "2", name: "b", steps: [] },
      { id: "3", name: "a", steps: [] },
      { id: "1", name: "b", steps: [] },
    ]);

    expect(sorted.map((flow) => flow.id)).toEqual(["3", "1", "2"]);
  });
});

describe("flow documents", () => {
  it("round-trips a flow through its document body and drops documents it cannot read", () => {
    const flow = { id: "flow-1", name: "Install and launch", steps };

    expect(
      readFlowDocuments({
        documents: [
          { id: "flow-1", body: toFlowDocumentBody(flow) },
          { id: "garbled", body: "{" },
          { id: "conflict", body: JSON.stringify({ name: 3 }) },
        ],
      }),
    ).toEqual([flow]);
    expect(JSON.parse(toFlowDocumentBody(flow)).steps[0].kind).toBe(
      "BATCH_ACTION_KIND_INSTALL_APK",
    );
  });
});
