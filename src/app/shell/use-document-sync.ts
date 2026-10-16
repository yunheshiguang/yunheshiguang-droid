import { useQueryClient } from "@tanstack/react-query";

import { documentsKey, WORKSPACE_EVENTS } from "@/shared/api";

import { useWorkspaceEvent } from "./use-workspace-event";

function collectionOf(payload: unknown): string | undefined {
  const collection = (payload as { collection?: unknown } | null)?.collection;

  return typeof collection === "string" && collection ? collection : undefined;
}

export function useDocumentSync(): void {
  const queryClient = useQueryClient();

  useWorkspaceEvent(WORKSPACE_EVENTS.documentsChanged, (payload) => {
    const collection = collectionOf(payload);

    if (collection) {
      void queryClient.invalidateQueries({ queryKey: documentsKey(collection) });
    }
  });
}
