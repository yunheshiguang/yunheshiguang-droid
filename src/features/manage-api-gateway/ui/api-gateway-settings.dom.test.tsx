import type { MessageInitShape } from "@bufbuild/protobuf";
import { timestampFromDate } from "@bufbuild/protobuf/wkt";
import type { ServiceImpl } from "@connectrpc/connect";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import {
  ApiGatewayService,
  type ApiGatewayStateSchema,
  ApiGatewayStatus,
  type RevokeApiTokenRequest,
  type UpdateApiGatewayRequest,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { manageApiGatewayMessages } from "../messages";
import { ApiGatewaySettings } from "./api-gateway-settings";

type GatewayState = MessageInitShape<typeof ApiGatewayStateSchema>;

const stopped: GatewayState = {
  config: { enabled: false, port: 0 },
  status: ApiGatewayStatus.STOPPED,
  tokens: [],
  authenticators: [{ id: "core.tokens", label: { key: "workspace.api.tokens.label" } }],
};

const running: GatewayState = {
  ...stopped,
  config: { enabled: true, port: 0 },
  status: ApiGatewayStatus.RUNNING,
  address: "http://127.0.0.1:47831",
};

function renderSettings(handlers: Partial<ServiceImpl<typeof ApiGatewayService>>) {
  installFakeServices((router) =>
    router.service(ApiGatewayService, { getApiGateway: () => stopped, ...handlers }),
  );
  const { wrapper } = createWorkspaceTestHarness(manageApiGatewayMessages);

  render(<ApiGatewaySettings />, { wrapper });
}

describe("ApiGatewaySettings", () => {
  it("turns the local API on and shows where it listens", async () => {
    const update = vi.fn((_request: UpdateApiGatewayRequest) => ({
      accepted: true,
      state: running,
    }));
    renderSettings({ updateApiGateway: update });

    expect(await screen.findByText("Off")).toBeInTheDocument();
    expect(screen.getByText("Local tokens (built-in)")).toBeInTheDocument();
    await userEvent.click(screen.getByLabelText("Turn on the local API"));
    await userEvent.click(screen.getByRole("button", { name: "Apply" }));

    expect(update.mock.calls[0]?.[0]).toMatchObject({
      config: { enabled: true, port: 0 },
    });
    expect(await screen.findByText("Listening on http://127.0.0.1:47831")).toBeInTheDocument();
  });

  it("shows a new token once and revokes tokens", async () => {
    const withToken: GatewayState = {
      ...stopped,
      tokens: [
        { id: "t-1", name: "CI", createdAt: timestampFromDate(new Date("2026-09-26T08:00:00Z")) },
      ],
    };
    const revoke = vi.fn((_request: RevokeApiTokenRequest) => ({
      accepted: true,
      state: stopped,
    }));
    renderSettings({
      createApiToken: () => ({ accepted: true, state: withToken, secret: "yhsg_secret-value" }),
      revokeApiToken: revoke,
    });

    await userEvent.type(await screen.findByLabelText("Purpose"), "CI");
    await userEvent.click(screen.getByRole("button", { name: "Issue token" }));

    expect(await screen.findByLabelText("New token")).toHaveValue("yhsg_secret-value");
    expect(screen.getByText("This token is shown only once. Copy it now")).toBeVisible();
    expect(screen.getByText("Never used", { exact: false })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Revoke token “CI”" }));
    expect(revoke).not.toHaveBeenCalled();
    await userEvent.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "Revoke" }),
    );
    expect(revoke.mock.calls[0]?.[0]).toMatchObject({ id: "t-1" });
  });
});
