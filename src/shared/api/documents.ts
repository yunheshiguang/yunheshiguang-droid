import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { type DocumentList, DocumentService } from "@/shared/proto";

import { workspaceMutationKeys } from "./query";

export type StoredDocuments = {
  documents: readonly { id: string; body: string }[];
};

export function documentsKey(collection: string) {
  return createConnectQueryKey({
    schema: DocumentService.method.listDocuments,
    input: { collection },
    cardinality: "finite",
  });
}

export function useDocumentList<T>(
  collection: string,
  select: (list: DocumentList) => T,
): T | undefined {
  return useQuery(DocumentService.method.listDocuments, { collection }, { select }).data;
}

export function useDocumentListError(collection: string): Error | null {
  return useQuery(DocumentService.method.listDocuments, { collection }).error;
}

export type DocumentActions<T> = {
  save: (item: T) => Promise<void>;
  remove: (id: string) => Promise<void>;
};

export function useDocumentActions<T extends { id: string }>(
  collection: string,
  encode: (item: T) => string,
): DocumentActions<T> {
  const queryClient = useQueryClient();
  const options = {
    mutationKey: workspaceMutationKeys.documents(collection),
    onSettled: () => queryClient.invalidateQueries({ queryKey: documentsKey(collection) }),
  };
  const save = useMutation(DocumentService.method.saveDocument, options);
  const remove = useMutation(DocumentService.method.removeDocument, options);

  return {
    save: async (item) => {
      await save.mutateAsync({ collection, document: { id: item.id, body: encode(item) } });
    },
    remove: async (id) => {
      await remove.mutateAsync({ collection, id });
    },
  };
}
