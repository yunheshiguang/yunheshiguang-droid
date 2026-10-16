import type { ConnectRouter } from "@connectrpc/connect";

import { createMockDocuments } from "@/shared/mock-host";
import { DocumentService } from "@/shared/proto";

export function createFakeDocuments() {
  const documents = createMockDocuments();

  return {
    service: documents.service,
    routes: (router: ConnectRouter) => router.service(DocumentService, documents.service),
    save: (collection: string, id: string, body: string) =>
      documents.save({ collection, document: { id, body } }),
    bodies: (collection: string): unknown[] =>
      (documents.list({ collection }).documents ?? []).map(({ body }) => JSON.parse(body ?? "")),
  };
}
