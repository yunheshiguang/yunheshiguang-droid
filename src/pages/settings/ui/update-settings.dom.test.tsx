import type { MessageInitShape } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { releaseMessages } from "@/entities/release";
import { setWindowGateway } from "@/shared/api";
import {
  type SetAutomaticChecksRequest,
  UpdateService,
  type UpdateStateSchema,
  UpdateStatus,
} from "@/shared/proto";
import {
  createWorkspaceTestHarness,
  idleWindowGateway,
  installFakeServices,
} from "@/shared/testing";

import { settingsMessages } from "../messages";
import { UpdateSettings } from "./update-settings";

type State = MessageInitShape<typeof UpdateStateSchema>;

const RELEASES = "https://github.com/yunheshiguang/yunheshiguang-android/releases";
const DOWNLOAD = `${RELEASES}/download/nightly/yunheshiguang-macos-arm64.dmg`;

const idle: State = {
  build: {
    channel: "nightly",
    version: "0.1.0",
    commit: "0123456789abcdef0123456789abcdef01234567",
    repository: "yunheshiguang/yunheshiguang-android",
  },
  checkAutomatically: true,
  status: UpdateStatus.IDLE,
  releasesUrl: RELEASES,
};

const available: State = {
  ...idle,
  status: UpdateStatus.AVAILABLE,
  update: {
    id: "89abcdef0123456789abcdef0123456789abcdef",
    channel: "nightly",
    version: "0.1.0",
    commit: "89abcdef0123456789abcdef0123456789abcdef",
    pageUrl: `${RELEASES}/tag/nightly`,
    downloadUrl: DOWNLOAD,
    downloadName: "yunheshiguang-macos-arm64.dmg",
  },
};

function renderWith(ui: ReactElement, handlers: Partial<ServiceImpl<typeof UpdateService>>) {
  const openExternal = vi.fn(async (_url: string) => {});
  setWindowGateway({ ...idleWindowGateway, openExternal });
  installFakeServices((router) =>
    router.service(UpdateService, { getUpdateState: () => idle, ...handlers }),
  );
  const { wrapper } = createWorkspaceTestHarness(settingsMessages, releaseMessages);
  render(ui, { wrapper });

  return { openExternal };
}

afterEach(() => {
  setWindowGateway(idleWindowGateway);
});

describe("UpdateSettings", () => {
  it("checks on request and offers the file that matches this install", async () => {
    const { openExternal } = renderWith(<UpdateSettings />, {
      checkForUpdates: () => available,
    });

    expect(await screen.findByText("Not checked yet")).toBeInTheDocument();
    expect(screen.getByText("Nightly")).toBeInTheDocument();
    expect(screen.getByText("Commit 0123456")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByText("A newer nightly build is out: 89abcde")).toBeInTheDocument();

    await userEvent.click(
      screen.getByRole("button", { name: "Download yunheshiguang-macos-arm64.dmg" }),
    );
    expect(openExternal).toHaveBeenCalledWith(DOWNLOAD);
    await userEvent.click(screen.getByRole("button", { name: "Release notes" }));
    expect(openExternal).toHaveBeenLastCalledWith(`${RELEASES}/tag/nightly`);
  });

  it("does not offer checks to a local build", async () => {
    renderWith(<UpdateSettings />, {
      getUpdateState: () => ({
        ...idle,
        build: { version: "0.1.0", repository: "yunheshiguang/yunheshiguang-android" },
        status: UpdateStatus.LOCAL_BUILD,
      }),
    });

    expect(
      await screen.findByText(
        "This copy was not built by the release workflows, so it does not check for updates",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Local build")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Check now" })).toBeDisabled();
    expect(screen.getByLabelText("Check for updates every day")).toBeDisabled();
  });

  it("says why the daily check setting was not saved", async () => {
    const setAutomaticChecks = vi.fn((_request: SetAutomaticChecksRequest) => ({
      accepted: false,
      detail: { key: "workspace.updates.storageFailed", params: { message: "disk full" } },
      state: idle,
    }));
    renderWith(<UpdateSettings />, { setAutomaticChecks });

    await userEvent.click(await screen.findByLabelText("Check for updates every day"));

    expect(setAutomaticChecks.mock.calls[0]?.[0]).toMatchObject({ checkAutomatically: false });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not save the update setting: disk full",
    );
    expect(screen.getByLabelText("Check for updates every day")).toBeChecked();
  });

  it("explains a failed check", async () => {
    renderWith(<UpdateSettings />, {
      getUpdateState: () => ({
        ...idle,
        status: UpdateStatus.FAILED,
        problem: { key: "workspace.updates.unreachable", params: { message: "HTTP 503" } },
      }),
    });

    expect(
      await screen.findByText(
        "Could not check for updates · Could not reach the release page: HTTP 503",
      ),
    ).toBeInTheDocument();
  });
});
