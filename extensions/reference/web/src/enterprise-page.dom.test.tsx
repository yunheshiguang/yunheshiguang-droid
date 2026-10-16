import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { LicenseStatus } from "yunheshiguang/sdk";
import {
  createExtensionTestHarness,
  GovernanceService,
  installFakeCommandGateway,
  installFakeServices,
  timestampFromDate,
} from "yunheshiguang/sdk/testing";

import { EnterprisePage } from "./enterprise-page";
import extension from "./index";
import { REFERENCE_PLUGIN, type ReferenceStatus } from "./reference-status";

const status: ReferenceStatus = {
  policyFile: "/config/extensions/reference/policy.cedar",
  policy: { state: "loaded" },
  policyRules: 3,
  teamFile: "/config/extensions/reference/team.json",
  team: { state: "loaded" },
  teamMembers: 2,
  licenseFile: "/config/extensions/reference/license.json",
  license: { state: "loaded" },
  auditFolder: "/data/extensions/reference/documents/audit",
  syncFile: "/config/extensions/reference/sync.json",
  sync: { state: "broken", detail: "unknown field `directory`" },
  syncFolder: "/data/extensions/reference/documents",
  assistantFile: "/config/extensions/reference/assistant.json",
  assistant: { state: "missing" },
  redroidFile: "/config/extensions/reference/redroid.json",
  redroid: { state: "missing" },
  vendorsFile: "/config/extensions/reference/vendors.json",
  vendors: { state: "missing" },
  metricsFile: "/data/extensions/reference/metrics.prom",
  apiKeysFile: "/config/extensions/reference/api-keys.json",
  apiKeys: { state: "loaded" },
};

const governance = {
  grantedCapabilities: ["reference.market", "reference.enterprise"],
  license: {
    planId: "enterprise",
    plan: { key: "reference.license.plan", params: { plan: "Enterprise" } },
    status: LicenseStatus.TRIAL,
    expiresAt: timestampFromDate(new Date("2027-01-01T12:00:00Z")),
    licensee: "Example Co.",
  },
  operator: { id: "alice", name: "Alice", roles: ["admin"] },
};

function renderPage(describeStatus: () => unknown) {
  installFakeCommandGateway({ [`plugin:${REFERENCE_PLUGIN}|describe`]: describeStatus });
  installFakeServices((router) =>
    router.service(GovernanceService, { getGovernanceState: () => governance }),
  );
  const { wrapper, queryClient } = createExtensionTestHarness(extension);
  const defaults = queryClient.getDefaultOptions();
  queryClient.setDefaultOptions({ ...defaults, queries: { ...defaults.queries, retry: false } });

  render(<EnterprisePage />, { wrapper });
}

describe("EnterprisePage", () => {
  it("shows the license and operator from the core and the team, policy and files from the extension", async () => {
    renderPage(() => status);

    expect(await screen.findByText("Enterprise plan")).toBeInTheDocument();
    expect(screen.getByText("Trial")).toBeInTheDocument();
    expect(screen.getByText("Issued to Example Co.")).toBeInTheDocument();
    expect(screen.getByText("Expires Jan 1, 2027")).toBeInTheDocument();
    expect(screen.getByText("reference.enterprise")).toBeInTheDocument();

    expect(await screen.findByText(/Operator: Alice/)).toBeInTheDocument();
    expect(screen.getByText("admin")).toBeInTheDocument();
    expect(screen.getByText("The team roster has 2 members")).toBeInTheDocument();
    expect(
      screen.getByText("The policy has 3 rules. Actions that no rule allows are refused"),
    ).toBeInTheDocument();
    expect(screen.getByText(status.auditFolder)).toBeInTheDocument();
    expect(screen.getByText(status.syncFolder)).toBeInTheDocument();
    expect(screen.getByText("Unreadable: unknown field `directory`")).toBeInTheDocument();
    expect(screen.getAllByText("In use")).toHaveLength(4);
    expect(screen.getAllByText("Not set up")).toHaveLength(3);
    expect(screen.getByText(status.redroidFile)).toBeInTheDocument();
    expect(screen.getByText(status.apiKeysFile)).toBeInTheDocument();
  });

  it("says everyone is a guest when the team roster cannot be read", async () => {
    renderPage(() => ({ ...status, team: { state: "broken", detail: "expected `,`" } }));

    expect(
      await screen.findByText(
        "Could not read the team roster; everyone is treated as a guest without roles",
      ),
    ).toBeInTheDocument();
  });

  it("keeps the license visible when the extension's own command fails", async () => {
    renderPage(() => Promise.reject("plugin reference-extension not found"));

    expect(await screen.findByText("Could not load the extension status")).toBeInTheDocument();
    expect(screen.getByText("plugin reference-extension not found")).toBeInTheDocument();
    expect(await screen.findByText("Enterprise plan")).toBeInTheDocument();
  });
});
