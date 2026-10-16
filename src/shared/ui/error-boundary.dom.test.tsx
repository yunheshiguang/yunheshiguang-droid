import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { ErrorBoundary } from "./error-boundary";

let broken = true;

function Module() {
  if (broken) {
    throw new Error("module crashed");
  }

  return <p>module content</p>;
}

function renderBoundary(resetKey: string) {
  return (
    <ErrorBoundary
      resetKey={resetKey}
      fallback={(error, reset) => (
        <div role="alert">
          {error.message}
          <button type="button" onClick={reset}>
            retry
          </button>
        </div>
      )}
    >
      <Module />
    </ErrorBoundary>
  );
}

beforeEach(() => {
  broken = true;
  vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("ErrorBoundary", () => {
  it("replaces a crashed module with the fallback instead of unmounting everything", () => {
    render(renderBoundary("devices"));

    expect(screen.getByRole("alert")).toHaveTextContent("module crashed");
  });

  it("keeps the fallback while the parent re-renders with the same key", () => {
    const { rerender } = render(renderBoundary("devices"));

    broken = false;
    rerender(renderBoundary("devices"));

    expect(screen.getByRole("alert")).toBeInTheDocument();
  });

  it("clears the error once the key changes", () => {
    const { rerender } = render(renderBoundary("devices"));

    broken = false;
    rerender(renderBoundary("batch"));

    expect(screen.getByText("module content")).toBeInTheDocument();
  });

  it("retries on demand", async () => {
    const user = userEvent.setup();
    render(renderBoundary("devices"));

    broken = false;
    await user.click(screen.getByRole("button", { name: "retry" }));

    expect(screen.getByText("module content")).toBeInTheDocument();
  });
});
