import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import { RunTargetSchema } from "@/shared/proto";

import { clampRunOption, runOptionLimits } from "./run-options";
import { draftFromTarget, emptyRunTarget, resolveRunTarget } from "./run-target";

describe("run options", () => {
  it("holds each option inside the limits the backend enforces", () => {
    expect(clampRunOption("repeat", 0)).toBe(1);
    expect(clampRunOption("repeat", 5000)).toBe(runOptionLimits.repeat.max);
    expect(clampRunOption("retryAttempts", 2.6)).toBe(3);
    expect(clampRunOption("retryDelaySeconds", Number.NaN)).toBe(0);
  });
});

describe("run targets", () => {
  it("resolves only a complete target", () => {
    expect(resolveRunTarget({ ...emptyRunTarget, deviceIds: ["a"] })).toEqual({
      target: { case: "devices", value: { deviceIds: ["a"] } },
    });
    expect(resolveRunTarget(emptyRunTarget)).toBeUndefined();
    expect(resolveRunTarget({ ...emptyRunTarget, mode: "group" })).toBeUndefined();
    expect(resolveRunTarget({ ...emptyRunTarget, mode: "tag", tag: "  night " })).toEqual({
      target: { case: "tag", value: "night" },
    });
  });

  it("puts a stored target back into the form", () => {
    expect(
      draftFromTarget(create(RunTargetSchema, { target: { case: "groupId", value: "shelf" } })),
    ).toMatchObject({
      mode: "group",
      groupId: "shelf",
    });
    expect(draftFromTarget(undefined)).toEqual(emptyRunTarget);
  });
});
