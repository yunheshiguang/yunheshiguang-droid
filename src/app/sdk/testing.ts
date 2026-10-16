import { createWorkspaceTestHarness, type WorkspaceTestHarness } from "@/shared/testing";
import type { WorkspaceExtension } from "@/shared/workspace-module";

import { collectAppMessages, createAppWorkspaceRegistry } from "../registry";

export { timestampFromDate } from "@bufbuild/protobuf/wkt";

export { BatchActionKind, GovernanceService, TemplateService } from "@/shared/proto";
export type { FakeCommandGateway, WorkspaceTestHarness } from "@/shared/testing";
export {
  createFakeDocuments,
  installFakeCommandGateway,
  installFakeServices,
} from "@/shared/testing";

export function createExtensionTestHarness(extension: WorkspaceExtension): WorkspaceTestHarness {
  const registry = createAppWorkspaceRegistry({
    extensions: [extension],
    capabilities: (extension.modules ?? []).flatMap((module) => module.requires ?? []),
  });

  return createWorkspaceTestHarness(collectAppMessages(registry, [extension]));
}
