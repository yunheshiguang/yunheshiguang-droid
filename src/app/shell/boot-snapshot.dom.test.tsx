import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";

import { createWorkspaceTestHarness } from "@/shared/testing";
import { ThemeProvider } from "@/shared/theme";

import { BOOT_SNAPSHOT_KEY, type BootSnapshot, useBootSnapshot } from "./boot-snapshot";

function readSnapshot(): BootSnapshot | null {
  const stored = window.localStorage.getItem(BOOT_SNAPSHOT_KEY);

  return stored ? (JSON.parse(stored) as BootSnapshot) : null;
}

describe("useBootSnapshot", () => {
  it("records the language and theme the shell actually rendered", async () => {
    window.localStorage.setItem("yunheshiguang.theme", "dark");
    const { wrapper: I18n } = createWorkspaceTestHarness();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <ThemeProvider>
        <I18n>{children}</I18n>
      </ThemeProvider>
    );

    renderHook(() => useBootSnapshot(), { wrapper });

    await waitFor(() =>
      expect(readSnapshot()).toEqual({
        lang: "en-US",
        loadingLabel: "Loading…",
        themeClass: "dark",
        colorScheme: "dark",
      }),
    );
  });
});
