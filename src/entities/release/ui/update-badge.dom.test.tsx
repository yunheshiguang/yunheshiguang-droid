import type { MessageInitShape } from "@bufbuild/protobuf";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { setWindowGateway } from "@/shared/api";
import { UpdateService, type UpdateStateSchema, UpdateStatus } from "@/shared/proto";
import {
  createWorkspaceTestHarness,
  idleWindowGateway,
  installFakeServices,
} from "@/shared/testing";

import { releaseMessages } from "../messages";
import { UpdateBadge } from "./update-badge";

type State = MessageInitShape<typeof UpdateStateSchema>;

const PAGE = "https://github.com/yunheshiguang/yunheshiguang-android/releases/tag/v0.2.0";

const upToDate: State = {
  build: { channel: "release", version: "0.1.0" },
  status: UpdateStatus.UP_TO_DATE,
};

function renderBadge(getUpdateState: () => State) {
  const openExternal = vi.fn(async (_url: string) => {});
  setWindowGateway({ ...idleWindowGateway, openExternal });
  installFakeServices((router) => router.service(UpdateService, { getUpdateState }));
  const { wrapper } = createWorkspaceTestHarness(releaseMessages);
  render(<UpdateBadge />, { wrapper });

  return { openExternal };
}

afterEach(() => {
  setWindowGateway(idleWindowGateway);
});

describe("UpdateBadge", () => {
  it("opens the release page of an available update", async () => {
    const { openExternal } = renderBadge(() => ({
      ...upToDate,
      status: UpdateStatus.AVAILABLE,
      update: { id: "v0.2.0", channel: "release", version: "0.2.0", pageUrl: PAGE },
    }));

    const badge = await screen.findByRole("button", { name: "Update available" });
    expect(badge).toHaveAttribute("title", "Open the release page for 0.2.0");
    await userEvent.click(badge);

    expect(openExternal).toHaveBeenCalledWith(PAGE);
  });

  it("stays hidden while the build is up to date", async () => {
    const getUpdateState = vi.fn(() => upToDate);
    renderBadge(getUpdateState);

    await vi.waitFor(() => expect(getUpdateState).toHaveBeenCalled());
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
