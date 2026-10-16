import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { createWorkspaceTestHarness, installFakeCommandGateway } from "@/shared/testing";

import { useExtensionQuery } from "./extension-commands";
import { useI18n } from "./extension-i18n";

describe("extension hooks", () => {
  it("hands extensions the locale and translation only", () => {
    const { wrapper } = createWorkspaceTestHarness();

    const { result } = renderHook(() => useI18n(), { wrapper });

    expect(Object.keys(result.current).sort()).toEqual(["locale", "t"]);
  });

  it("lets an extension ask for its command's answer again", async () => {
    let answers = 0;
    installFakeCommandGateway({
      "plugin:sample|describe": () => {
        answers += 1;
        return { answer: answers };
      },
    });
    const { wrapper } = createWorkspaceTestHarness();

    const { result } = renderHook(
      () => useExtensionQuery<{ answer: number }>("sample", "describe"),
      {
        wrapper,
      },
    );
    await waitFor(() => expect(result.current.data).toEqual({ answer: 1 }));
    act(() => result.current.refetch());

    await waitFor(() => expect(result.current.data).toEqual({ answer: 2 }));
  });
});
