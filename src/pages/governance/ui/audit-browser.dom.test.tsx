import type { MessageInitShape } from "@bufbuild/protobuf";
import { timestampFromDate } from "@bufbuild/protobuf/wkt";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { auditMessages } from "@/entities/audit";
import { setDialogGateway } from "@/shared/api";
import {
  AuditExportFormat,
  AuditOutcome,
  type AuditPageSchema,
  AuditService,
  type ExportAuditRequest,
  GovernedAction,
  type QueryAuditRequest,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { governancePageMessages } from "../messages";
import { AuditBrowser } from "./audit-browser";

type AuditPageInit = MessageInitShape<typeof AuditPageSchema>;

function entry(id: string, action: GovernedAction) {
  return {
    id,
    occurredAt: timestampFromDate(new Date("2026-09-26T08:00:00Z")),
    action,
    outcome: AuditOutcome.SUCCEEDED,
    actor: "main",
    operator: { id: "alice", name: "Alice", roles: ["admin"] },
    targets: ["***5555"],
    groups: ["shelf-a"],
  };
}

function renderBrowser(
  queryAudit: (request: QueryAuditRequest) => AuditPageInit,
  operators: readonly { key: string; label: string }[] = [],
) {
  const exportAudit = vi.fn((_request: ExportAuditRequest) => ({
    accepted: true,
    detail: { key: "workspace.audit.exported", params: { count: "2", path: "/tmp/audit.csv" } },
    exported: 2,
  }));
  installFakeServices((router) =>
    router.service(AuditService, {
      queryAudit,
      exportAudit,
      reportAudit: () => ({ byOperator: [...operators] }),
    }),
  );
  const { wrapper } = createWorkspaceTestHarness(governancePageMessages, auditMessages);

  render(<AuditBrowser groupName={(id) => (id === "shelf-a" ? "Shelf A" : id)} />, { wrapper });
  return { exportAudit };
}

describe("AuditBrowser", () => {
  it("pages back through the history and filters it by action", async () => {
    const queryAudit = vi.fn(
      (request: QueryAuditRequest): AuditPageInit =>
        request.query?.beforeId
          ? { entries: [entry("a-1", GovernedAction.PAIR_DEVICE)], nextBeforeId: "" }
          : { entries: [entry("a-2", GovernedAction.CONNECT_DEVICE)], nextBeforeId: "a-2" },
    );
    renderBrowser(queryAudit);

    const [newest] = await screen.findAllByRole("listitem");
    expect(newest).toHaveTextContent("Connect device");
    expect(newest).toHaveTextContent("By Alice");
    expect(newest).toHaveTextContent("In Shelf A");
    await userEvent.click(screen.getByRole("button", { name: "Load older entries" }));
    await waitFor(() => expect(screen.getAllByRole("listitem")).toHaveLength(2));
    expect(screen.getAllByRole("listitem")[1]).toHaveTextContent("Pair device");
    expect(screen.queryByRole("button", { name: "Load older entries" })).not.toBeInTheDocument();

    await userEvent.selectOptions(screen.getByLabelText("Action"), "PAIR_DEVICE");
    expect(queryAudit.mock.calls.at(-1)?.[0]).toMatchObject({
      query: { actions: [GovernedAction.PAIR_DEVICE], beforeId: "" },
    });
  });

  it("exports what the filter shows to the file the operator picks", async () => {
    const pickPath = vi.fn(async () => "/tmp/audit.csv");
    setDialogGateway({ pickPath });
    const { exportAudit } = renderBrowser(() => ({
      entries: [entry("a-1", GovernedAction.PAIR_DEVICE)],
    }));

    await screen.findAllByRole("listitem");
    await userEvent.selectOptions(screen.getByLabelText("Outcome"), "SUCCEEDED");
    await userEvent.click(screen.getByRole("button", { name: "Export CSV" }));

    expect(pickPath).toHaveBeenCalledWith(
      expect.objectContaining({ target: "save", extensions: ["csv"] }),
    );
    expect(exportAudit.mock.calls[0]?.[0]).toMatchObject({
      format: AuditExportFormat.CSV,
      path: "/tmp/audit.csv",
      query: { outcomes: [AuditOutcome.SUCCEEDED] },
    });
    expect(
      await screen.findByText("Exported 2 audit entries to /tmp/audit.csv"),
    ).toBeInTheDocument();
  });

  it("offers every operator in the trail, not only those on the loaded page", async () => {
    renderBrowser(
      () => ({ entries: [entry("a-1", GovernedAction.PAIR_DEVICE)] }),
      [
        { key: "alice", label: "Alice" },
        { key: "bob", label: "Bob" },
      ],
    );

    await screen.findAllByRole("listitem");
    const operator = screen.getByLabelText("Operator");
    expect(await within(operator).findByRole("option", { name: "Bob" })).toBeInTheDocument();
    await userEvent.selectOptions(operator, "alice");
    expect(within(operator).getByRole("option", { name: "Bob" })).toBeInTheDocument();
  });

  it("says why the history is unavailable", async () => {
    renderBrowser(() => ({ entries: [], problem: { key: "workspace.audit.unavailable" } }));

    expect(await screen.findByText("Could not load audit entries")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Audit history is unavailable. Only actions since the app started are shown",
      ),
    ).toBeInTheDocument();
  });
});
