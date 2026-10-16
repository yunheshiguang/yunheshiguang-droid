import { create } from "@bufbuild/protobuf";
import { Code, ConnectError } from "@connectrpc/connect";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { taskMessages } from "@/entities/task";
import {
  BatchActionKind,
  GovernedAction,
  TelemetryService,
  WorkspaceHealthSchema,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { healthPageMessages } from "../messages";
import { HealthPage } from "./health-page";

describe("HealthPage", () => {
  it("shows availability, each operation's record and every telemetry destination", async () => {
    installFakeServices((router) =>
      router.service(TelemetryService, {
        getWorkspaceHealth: () =>
          create(WorkspaceHealthSchema, {
            probes: [
              { online: 3, devices: 4, unauthorized: 1, durationMs: 120 },
              { online: 4, devices: 4, durationMs: 90 },
            ],
            probeCapacity: 60,
            operations: [
              {
                action: GovernedAction.RUN_BATCH_ACTION,
                batchAction: BatchActionKind.SCREENSHOT,
                runs: 3,
                failures: 1,
                averageMs: 800,
                slowestMs: 1500,
              },
              { action: GovernedAction.CONNECT_DEVICE, runs: 1, averageMs: 40, slowestMs: 40 },
            ],
            sinks: [
              {
                id: "core.health",
                label: { key: "workspace.telemetry.health.label" },
                description: { key: "workspace.telemetry.health.description" },
              },
            ],
          }),
      }),
    );
    const { wrapper } = createWorkspaceTestHarness(taskMessages, healthPageMessages);

    render(<HealthPage />, { wrapper });

    expect(await screen.findByText("3 of 4 online")).toBeInTheDocument();
    expect(
      screen.getByText("1 waiting for authorization · 0 offline · Last probe took 120 ms"),
    ).toBeInTheDocument();
    expect(screen.getByText("Availability over the last 2 probes")).toBeInTheDocument();

    const rows = within(screen.getByRole("table")).getAllByRole("row");
    expect(rows.map((row) => row.textContent)).toEqual([
      "OperationRunsFailedAverageSlowest",
      "Run batch operation · Screenshot31800 ms1500 ms",
      "Connect device1040 ms40 ms",
    ]);
    expect(screen.getByText("Health summary")).toBeInTheDocument();
  });

  it("says so when the health data cannot be read", async () => {
    installFakeServices((router) =>
      router.service(TelemetryService, {
        getWorkspaceHealth: () => {
          throw new ConnectError("host gone", Code.Unavailable);
        },
      }),
    );
    const { wrapper, queryClient } = createWorkspaceTestHarness(healthPageMessages);
    const defaults = queryClient.getDefaultOptions();
    queryClient.setDefaultOptions({ ...defaults, queries: { ...defaults.queries, retry: false } });

    render(<HealthPage />, { wrapper });

    expect(await screen.findByText("Could not load health data")).toBeInTheDocument();
    expect(screen.getByText("host gone")).toBeInTheDocument();
  });
});
