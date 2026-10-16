import { create } from "@bufbuild/protobuf";
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
  AndroidPlatformToolsStateSchema,
  DiagnosticCode,
  DiagnosticSignalSchema,
} from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import type { PlatformToolsCommands } from "./use-platform-tools-commands";
import { usePlatformToolsForm } from "./use-platform-tools-form";

describe("usePlatformToolsForm", () => {
  it("keeps a refusal visible after the settings snap back", async () => {
    const refused = create(AndroidPlatformToolsStateSchema, {
      diagnostics: [
        create(DiagnosticSignalSchema, {
          code: DiagnosticCode.ACTION_NOT_AUTHORIZED,
          impact: { key: "workspace.api.hostFiles" },
        }),
      ],
    });
    const commands: PlatformToolsCommands = {
      apply: async () => refused,
      install: async () => refused,
      applying: false,
      installing: false,
    };
    const { wrapper } = createWorkspaceTestHarness();
    const { result } = renderHook(() => usePlatformToolsForm(undefined, commands), { wrapper });

    await act(() => result.current.applyCurrent());

    expect(result.current.error).toBe(
      "The local API cannot use files on this computer. Run this action from the app",
    );
  });
});
