import { create } from "@bufbuild/protobuf";
import { timestampFromMs } from "@bufbuild/protobuf/wkt";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { type AuditEntry, AuditEntrySchema, AuditOutcome, GovernedAction } from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { auditMessages } from "../messages";
import { AuditTrail } from "./audit-trail";

const occurredAt = Date.parse("2026-09-25T10:00:00Z");

const entries: AuditEntry[] = [
  create(AuditEntrySchema, {
    id: "audit-2",
    occurredAt: timestampFromMs(occurredAt),
    action: GovernedAction.RUN_BATCH_ACTION,
    outcome: AuditOutcome.DENIED,
    actor: "main",
    operator: { id: "alice", name: "Alice", roles: ["operator"] },
    targets: ["***BCDE", "***5555"],
    groups: ["shelf-a"],
    detail: { key: "workspace.task.title.screenshot" },
  }),
  create(AuditEntrySchema, {
    id: "audit-1",
    action: GovernedAction.CONNECT_DEVICE,
    outcome: AuditOutcome.SUCCEEDED,
    actor: "workspace-2",
  }),
];

function renderTrail(list: AuditEntry[]) {
  const { wrapper } = createWorkspaceTestHarness(auditMessages);

  render(<AuditTrail entries={list} capacity={200} />, { wrapper });
}

describe("AuditTrail", () => {
  it("lists each action with its outcome, origin and masked targets", () => {
    renderTrail(entries);

    const [denied, connected] = screen.getAllByRole("listitem");

    expect(denied).toHaveTextContent("Run batch operation · Screenshot");
    expect(denied).toHaveTextContent("Denied");
    expect(denied).toHaveTextContent("By Alice");
    expect(denied).toHaveTextContent("From main");
    expect(denied).toHaveTextContent("On ***BCDE, ***5555");
    expect(denied).toHaveTextContent("In shelf-a");
    expect(within(denied).getByRole("time")).toHaveAttribute(
      "datetime",
      "2026-09-25T10:00:00.000Z",
    );

    expect(connected).toHaveTextContent("Connect device");
    expect(connected).toHaveTextContent("Succeeded");
    expect(connected).toHaveTextContent("Time unknown");
    expect(connected).not.toHaveTextContent("On ");
    expect(connected).not.toHaveTextContent("By ");
    expect(
      screen.getByText("Keeps the latest 200 actions. Older ones are in the log"),
    ).toBeInTheDocument();
  });

  it("says when nothing has been recorded yet", () => {
    renderTrail([]);

    expect(screen.getByText("No actions recorded since the app started")).toBeInTheDocument();
    expect(screen.queryByRole("listitem")).not.toBeInTheDocument();
  });
});
