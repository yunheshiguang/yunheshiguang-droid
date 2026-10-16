import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Suspense } from "react";
import { describe, expect, it } from "vitest";

import { ErrorBoundary } from "@/shared/ui";

import { lazyModulePage } from "./lazy-page";

describe("lazyModulePage", () => {
  it("loads the page again after a failed load is retried", async () => {
    let reachable = false;
    const Page = lazyModulePage(
      async () => {
        if (!reachable) {
          throw new Error("Failed to fetch dynamically imported module");
        }
        return { Settings: () => <p>Settings page</p> };
      },
      (module) => module.Settings,
    );

    render(
      <ErrorBoundary
        fallback={(error, reset) => (
          <button type="button" onClick={reset}>
            {`Retry after ${error.message}`}
          </button>
        )}
      >
        <Suspense fallback={null}>
          <Page />
        </Suspense>
      </ErrorBoundary>,
    );

    const retry = await screen.findByRole("button", { name: /Retry after Failed/ });
    reachable = true;
    await userEvent.click(retry);

    expect(await screen.findByText("Settings page")).toBeInTheDocument();
  });
});
