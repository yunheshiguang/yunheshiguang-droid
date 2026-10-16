import { describe, expect, it } from "vitest";

import * as sdk from "./index";

describe("yunheshiguang/sdk", () => {
  it("keeps the runtime surface extensions are built against", () => {
    expect(Object.keys(sdk).sort()).toEqual([
      "Badge",
      "Button",
      "EmptyState",
      "ErrorState",
      "FlowTemplateList",
      "LicenseStatus",
      "SectionCard",
      "defineWorkspaceExtension",
      "enumKey",
      "invokeExtensionCommand",
      "lazyModulePage",
      "mergeModuleMessages",
      "timestampDate",
      "useExtensionQuery",
      "useGovernanceState",
      "useGrantedCapabilities",
      "useI18n",
      "workspaceNavigationGroupIds",
    ]);
  });
});
