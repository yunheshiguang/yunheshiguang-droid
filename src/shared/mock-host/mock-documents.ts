import type { ServiceImpl } from "@connectrpc/connect";

import { compareIds } from "@/shared/lib/text";
import {
  type DocumentListJson,
  DocumentListSchema,
  type DocumentService,
  type ListDocumentsRequestJson,
  type RemoveDocumentRequestJson,
  type SaveDocumentRequestJson,
} from "@/shared/proto";

import { fromMockJson } from "./mock-context";

export function createMockDocuments() {
  const collections = new Map<string, Map<string, string>>();

  const collectionOf = (name: string) => {
    const existing = collections.get(name) ?? new Map<string, string>();

    collections.set(name, existing);
    return existing;
  };

  const store = {
    list({ collection = "" }: ListDocumentsRequestJson): DocumentListJson {
      const documents = [...collectionOf(collection).entries()]
        .sort(([left], [right]) => compareIds(left, right))
        .map(([id, body]) => ({ id, body }));

      return { collection, documents };
    },

    save({ collection = "", document }: SaveDocumentRequestJson): void {
      const id = document?.id ?? "";
      const body = document?.body ?? "";

      if (!/^[a-z0-9][a-z0-9-]*$/.test(collection) || !/^[A-Za-z0-9][\w-]*$/.test(id)) {
        throw new Error("invalid collection or document name");
      }

      JSON.parse(body);
      collectionOf(collection).set(id, body);
    },

    remove({ collection = "", id = "" }: RemoveDocumentRequestJson): void {
      collectionOf(collection).delete(id);
    },
  };

  const service: ServiceImpl<typeof DocumentService> = {
    listDocuments: ({ collection }) => fromMockJson(DocumentListSchema, store.list({ collection })),
    saveDocument: ({ collection, document }) => {
      store.save({ collection, document: { id: document?.id, body: document?.body } });
      return {};
    },
    removeDocument: ({ collection, id }) => {
      store.remove({ collection, id });
      return {};
    },
  };

  return { ...store, service };
}
