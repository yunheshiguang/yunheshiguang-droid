import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import { ScheduleSchema } from "@/shared/proto";

import { draftFromSchedule, newScheduleDraft, toSchedule } from "./schedule-draft";

describe("schedule drafts", () => {
  it("saves a complete draft with a trimmed name and its resolved target", () => {
    const draft = { ...newScheduleDraft("morning", ["a", "b"]), name: "  Morning  " };

    expect(toSchedule(draft)).toMatchObject({
      name: "Morning",
      flowId: "morning",
      target: { target: { case: "devices", value: { deviceIds: ["a", "b"] } } },
      cron: "0 9 * * *",
      enabled: true,
    });
  });

  it("refuses drafts that miss a name, a flow, a target or a time", () => {
    const draft = { ...newScheduleDraft("morning", ["a"]), name: "Morning" };

    expect(toSchedule({ ...draft, name: " " })).toBeUndefined();
    expect(toSchedule({ ...draft, flowId: "" })).toBeUndefined();
    expect(toSchedule({ ...draft, cron: "" })).toBeUndefined();
    expect(toSchedule({ ...draft, target: { ...draft.target, deviceIds: [] } })).toBeUndefined();
    expect(toSchedule({ ...draft, target: { ...draft.target, mode: "group" } })).toBeUndefined();
  });

  it("puts a saved schedule back into the form as it was", () => {
    const saved = toSchedule({
      ...newScheduleDraft("morning", []),
      name: "Shelf",
      target: { mode: "group", groupId: "shelf-a", deviceIds: [], tag: "" },
    });

    expect(saved).toBeDefined();
    if (saved) {
      expect(toSchedule(draftFromSchedule(create(ScheduleSchema, saved)))).toEqual(saved);
    }
  });
});
