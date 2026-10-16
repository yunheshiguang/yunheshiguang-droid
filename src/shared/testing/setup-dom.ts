import "@testing-library/jest-dom/vitest";
import { cleanup, configure } from "@testing-library/react";
import { afterEach, vi } from "vitest";

import { resetGateways } from "@/shared/api";

import { resetFakeServices } from "./fake-services";
import { takeMissingCopy } from "./missing-copy";

configure({ asyncUtilTimeout: 3_000 });

vi.mock("zustand", async () => (await import("./zustand-reset")).zustandWithReset);

afterEach(() => {
  cleanup();
  resetGateways();
  resetFakeServices();
  window.localStorage.clear();

  const missingCopy = takeMissingCopy();

  if (missingCopy.length > 0) {
    throw new Error(`rendered copy that no locale defines: ${missingCopy.join(", ")}`);
  }
});
