import type { ServiceImpl } from "@connectrpc/connect";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { setDialogGateway } from "@/shared/api";
import {
  ConflictPolicy,
  type InspectPackageRequest,
  type InstallPackageRequest,
  PackageService,
  type TrustPackageSignerRequest,
} from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { sharePackagesMessages } from "../messages";
import { TemplatePackages } from "./template-packages";

const preview = {
  accepted: true,
  preview: {
    manifest: { id: "com.example.nightly", name: "Nightly", version: "1.2.0", author: "Ops" },
    flows: [{ id: "nightly", name: "Nightly check", exists: true }],
    scripts: [{ id: "battery", name: "Battery", exists: false }],
    problems: [{ key: "workspace.packages.problem.unverified" }],
    verified: false,
  },
};

const sourceCopy = {
  "zh-CN": { test: { market: "模板市场" } },
  "en-US": { test: { market: "Template market" } },
};

function renderPackages(handlers: Partial<ServiceImpl<typeof PackageService>>) {
  installFakeServices(createFakeDocuments().routes, (router) =>
    router.service(PackageService, { listMarketPackages: () => ({ packages: [] }), ...handlers }),
  );
  const { wrapper } = createWorkspaceTestHarness(sharePackagesMessages, sourceCopy);

  render(<TemplatePackages />, { wrapper });
}

describe("TemplatePackages", () => {
  it("previews a package file and installs it by the chosen conflict policy", async () => {
    setDialogGateway({ pickPath: async () => "/tmp/nightly.json" });
    const inspect = vi.fn((_request: InspectPackageRequest) => preview);
    const install = vi.fn((_request: InstallPackageRequest) => ({
      accepted: true,
      detail: { key: "workspace.packages.installed", params: { flows: "1", scripts: "1" } },
      flows: 1,
      scripts: 1,
    }));
    renderPackages({ inspectPackage: inspect, installPackage: install });

    await userEvent.click(screen.getByRole("button", { name: "Import from file" }));

    expect(inspect.mock.calls[0]?.[0].origin?.origin).toEqual({
      case: "path",
      value: "/tmp/nightly.json",
    });
    const card = await screen.findByRole("region", { name: "Package contents" });
    expect(card).toHaveTextContent("Nightly v1.2.0");
    expect(card).toHaveTextContent("Not verified");
    expect(card).toHaveTextContent("Already in the library");
    expect(card).toHaveTextContent("Not signed by a signer you trust");
    expect(card).toHaveTextContent("Unsigned, so the publisher cannot be verified");

    await userEvent.selectOptions(screen.getByLabelText("If already in the library"), "REPLACE");
    await userEvent.click(screen.getByRole("button", { name: "Install" }));

    expect(install.mock.calls[0]?.[0]).toMatchObject({
      origin: { origin: { case: "path", value: "/tmp/nightly.json" } },
      onConflict: ConflictPolicy.REPLACE,
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Installed the package (flows: 1, scripts: 1)",
    );
    expect(screen.queryByRole("region", { name: "Package contents" })).not.toBeInTheDocument();
    expect(install).toHaveBeenCalledTimes(1);
  });

  it("names who signed a package and trusts that signer on request", async () => {
    setDialogGateway({ pickPath: async () => "/tmp/nightly.json" });
    const trust = vi.fn((_request: TrustPackageSignerRequest) => ({ accepted: true }));
    renderPackages({
      inspectPackage: () => ({
        accepted: true,
        preview: {
          ...preview.preview,
          verified: true,
          problems: [],
          signer: { keyId: "5D3A1F0C9B8E7D6A", name: "Ops", trusted: false, publicKey: "RWQ..." },
        },
      }),
      trustPackageSigner: trust,
    });

    await userEvent.click(screen.getByRole("button", { name: "Import from file" }));
    const card = await screen.findByRole("region", { name: "Package contents" });
    expect(card).toHaveTextContent("Signed by Ops (key 5D3A1F0C9B8E7D6A)");
    expect(card).toHaveTextContent("Not in your trusted signers");

    await userEvent.click(screen.getByRole("button", { name: "Trust this signer" }));

    expect(trust.mock.calls[0]?.[0]).toMatchObject({ publicKey: "RWQ...", name: "Ops" });
    expect(await screen.findByText("Trusted")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Trust this signer" })).not.toBeInTheDocument();
  });

  it("lists market packages and says when a file cannot be read", async () => {
    setDialogGateway({ pickPath: async () => "/tmp/broken.json" });
    renderPackages({
      listMarketPackages: () => ({
        packages: [
          {
            sourceId: "reference.market",
            source: { key: "test.market" },
            manifest: { id: "reference.nightly-check", name: "Nightly check", version: "1.0.0" },
            flows: 1,
            scripts: 1,
          },
        ],
      }),
      inspectPackage: () => ({
        accepted: false,
        detail: { key: "workspace.packages.digestMismatch" },
      }),
    });

    expect(await screen.findByText("Nightly check v1.0.0")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Import from file" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("digest does not match");
  });
});
