import type { ServiceImpl } from "@connectrpc/connect";

import {
  type UpdateService,
  UpdateSettingsResultSchema,
  type UpdateStateJson,
  UpdateStateSchema,
} from "@/shared/proto";

import { fromMockJson, type MockContext } from "./mock-context";

const REPOSITORY = "yunheshiguang/yunheshiguang-android";
const RELEASES = `https://github.com/${REPOSITORY}/releases`;
const RUNNING = "0123456789abcdef0123456789abcdef01234567";
const PUBLISHED = "89abcdef0123456789abcdef0123456789abcdef";
const DOWNLOAD = "yunheshiguang-macos-arm64.dmg";

export function createMockUpdates(context: MockContext): ServiceImpl<typeof UpdateService> {
  let state: UpdateStateJson = {
    build: { channel: "nightly", version: "0.1.0", commit: RUNNING, repository: REPOSITORY },
    checkAutomatically: true,
    status: "UPDATE_STATUS_IDLE",
    releasesUrl: RELEASES,
  };
  const current = () => fromMockJson(UpdateStateSchema, state);

  return {
    getUpdateState: () => context.settle(current()),
    checkForUpdates: () => {
      state = {
        ...state,
        status: "UPDATE_STATUS_AVAILABLE",
        checkedAt: new Date().toISOString(),
        update: {
          id: PUBLISHED,
          channel: "nightly",
          version: "0.1.0",
          commit: PUBLISHED,
          pageUrl: `${RELEASES}/tag/nightly`,
          downloadUrl: `${RELEASES}/download/nightly/${DOWNLOAD}`,
          downloadName: DOWNLOAD,
        },
      };
      return context.settle(current());
    },
    setAutomaticChecks: (request) => {
      state = { ...state, checkAutomatically: request.checkAutomatically };
      return context.settle(fromMockJson(UpdateSettingsResultSchema, { accepted: true, state }));
    },
  };
}
