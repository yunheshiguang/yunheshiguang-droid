import { create, type MessageInitShape } from "@bufbuild/protobuf";
import { timestampFromMs } from "@bufbuild/protobuf/wkt";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { GovernanceStateSchema, LicenseStatus } from "@/shared/proto";
import { createWorkspaceTestHarness } from "@/shared/testing";

import { governancePageMessages } from "../messages";
import { GovernancePolicy } from "./governance-policy";

type State = MessageInitShape<typeof GovernanceStateSchema>;

const openSource: State = { auditCapacity: 200 };

function renderPolicy(state: State) {
  const { wrapper } = createWorkspaceTestHarness(governancePageMessages);

  render(<GovernancePolicy state={create(GovernanceStateSchema, state)} />, { wrapper });
}

describe("GovernancePolicy", () => {
  it("shows the plan, status, licensee and expiry behind the granted capabilities", () => {
    renderPolicy({
      ...openSource,
      grantedCapabilities: ["enterprise.console"],
      license: {
        planId: "enterprise",
        status: LicenseStatus.TRIAL,
        expiresAt: timestampFromMs(Date.UTC(2026, 11, 31, 12)),
        licensee: "Example Co.",
      },
    });

    expect(screen.getByText("enterprise.console")).toBeInTheDocument();
    expect(screen.getByText("enterprise")).toBeInTheDocument();
    expect(screen.getByText("Trial")).toBeInTheDocument();
    expect(screen.getByText("Issued to Example Co.")).toBeInTheDocument();
    expect(screen.getByText("Expires Dec 31, 2026")).toBeInTheDocument();
  });

  it("says the open-source core runs without a license", () => {
    renderPolicy(openSource);

    expect(screen.getByText("No license installed")).toBeInTheDocument();
    expect(screen.getByText("No extra features granted")).toBeInTheDocument();
  });
});
