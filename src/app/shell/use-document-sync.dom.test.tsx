import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useAutomationFlows } from "@/entities/automation";
import { setCommandGateway, WORKSPACE_EVENTS } from "@/shared/api";
import { DocumentService } from "@/shared/proto";
import {
  createFakeCommandGateway,
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { useDocumentSync } from "./use-document-sync";

describe("useDocumentSync", () => {
  it("re-reads only the collection another window changed, until it unmounts", async () => {
    const documents = createFakeDocuments();
    const list = vi.fn(documents.service.listDocuments);
    installFakeServices((router) =>
      router.service(DocumentService, { ...documents.service, listDocuments: list }),
    );
    const gateway = createFakeCommandGateway({});
    setCommandGateway(gateway);
    const { wrapper } = createWorkspaceTestHarness();

    const { result, unmount } = renderHook(
      () => {
        useDocumentSync();
        return useAutomationFlows();
      },
      { wrapper },
    );
    await waitFor(() => expect(list).toHaveBeenCalledTimes(1));

    documents.save(
      "automation-flows",
      "flow-1",
      JSON.stringify({
        version: 1,
        name: "Saved elsewhere",
        steps: [{ id: "s", kind: "SCREENSHOT", value: "/tmp/shots" }],
      }),
    );
    gateway.emit(WORKSPACE_EVENTS.documentsChanged, { collection: "another-collection" });
    gateway.emit(WORKSPACE_EVENTS.documentsChanged, { collection: "automation-flows" });

    await waitFor(() =>
      expect(result.current.map((flow) => flow.name)).toEqual(["Saved elsewhere"]),
    );
    expect(list).toHaveBeenCalledTimes(2);

    unmount();
    gateway.emit(WORKSPACE_EVENTS.documentsChanged, { collection: "automation-flows" });
    expect(list).toHaveBeenCalledTimes(2);
  });
});
