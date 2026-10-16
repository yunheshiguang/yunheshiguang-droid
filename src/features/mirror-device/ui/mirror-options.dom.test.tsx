import { create, toBinary } from "@bufbuild/protobuf";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { type ScrcpyConfig, ScrcpyConfigSchema } from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { mirrorDeviceMessages } from "../messages";
import { MirrorOptions } from "./mirror-options";

describe("MirrorOptions", () => {
  it("keeps every number within what the settings can carry", () => {
    const changes: ScrcpyConfig[] = [];
    const { wrapper } = createWorkspaceTestHarness(mirrorDeviceMessages);
    render(
      <MirrorOptions
        config={create(ScrcpyConfigSchema)}
        disabled={false}
        onChange={(config) => changes.push(config)}
      />,
      { wrapper },
    );

    fireEvent.change(screen.getByLabelText("Video bit rate (Mbps, 0 for default)"), {
      target: { value: "-8" },
    });
    fireEvent.change(screen.getByLabelText("Longest side in pixels (0 for no limit)"), {
      target: { value: "99999999999" },
    });

    expect(changes[0]?.videoBitRateBps).toBe(0n);
    expect(changes[1]?.maxSize).toBe(0xffff_ffff);
    for (const change of changes) {
      expect(() => toBinary(ScrcpyConfigSchema, change)).not.toThrow();
    }
  });
});
