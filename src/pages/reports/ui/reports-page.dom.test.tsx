import { createClient } from "@connectrpc/connect";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { rpcTransport } from "@/shared/api";
import { installMockHost } from "@/shared/mock-host";
import { BatchActionKind, BatchService } from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { reportsPageMessages } from "../messages";
import { ReportsPage } from "./reports-page";

beforeEach(() => {
  installMockHost({ latencyMs: 0 });
});

const batches = () => createClient(BatchService, rpcTransport());

function renderPage() {
  const { wrapper } = createWorkspaceTestHarness(reportsPageMessages);

  return render(<ReportsPage />, { wrapper });
}

describe("ReportsPage", () => {
  it("totals what was done by operator, action, and day", async () => {
    for (const deviceIds of [["R58M20ABCDE"], ["RZ8N70XYZ12", "0A1B2C3D4E"]]) {
      await batches().runBatchAction({ kind: BatchActionKind.RECOVER_CONNECTION, deviceIds });
    }
    renderPage();

    expect(await screen.findByText("2 operations in total")).toBeInTheDocument();
    expect(screen.getByRole("row", { name: /Browser preview/ })).toBeInTheDocument();
    const action = screen.getByRole("row", { name: /Run batch operation/ });
    expect(
      within(action)
        .getAllByRole("cell")
        .map((cell) => cell.textContent),
    ).toEqual(["Run batch operation", "2", expect.any(String), "0"]);
    expect(screen.getByRole("listitem", { name: /2 runs/ })).toBeInTheDocument();
  });

  it("uses the singular for a day with one run", async () => {
    await batches().runBatchAction({
      kind: BatchActionKind.RECOVER_CONNECTION,
      deviceIds: ["R58M20ABCDE"],
    });
    renderPage();

    expect(await screen.findByText("1 operation in total")).toBeInTheDocument();
    expect(screen.getByRole("listitem", { name: /: 1 run, 0 failed/ })).toBeInTheDocument();
  });

  it("starts empty and widens the period on request", async () => {
    const user = userEvent.setup();
    renderPage();

    expect(await screen.findByText("0 operations in total")).toBeInTheDocument();
    expect(screen.getAllByText("No operations in this period").length).toBeGreaterThan(0);

    await user.selectOptions(screen.getByLabelText("Period"), "quarter");
    expect(screen.getByLabelText("Period")).toHaveValue("quarter");
    expect(await screen.findByText("0 operations in total")).toBeInTheDocument();
  });
});
