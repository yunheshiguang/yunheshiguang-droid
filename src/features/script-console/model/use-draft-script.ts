import { create } from "@bufbuild/protobuf";
import { createClient } from "@connectrpc/connect";
import { useTransport } from "@connectrpc/connect-query";
import { useMutation } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import { workspaceMutationKeys } from "@/shared/api";
import {
  AssistantService,
  type DraftScriptResult,
  DraftScriptResultSchema,
  type ScriptLanguage,
} from "@/shared/proto";

export function useDraftScript() {
  const transport = useTransport();
  const [partial, setPartial] = useState("");
  const running = useRef<AbortController | undefined>(undefined);
  useEffect(() => () => running.current?.abort(), []);
  const mutation = useMutation({
    mutationKey: workspaceMutationKeys.assistantDraft(),
    mutationFn: async ({
      prompt,
      language,
      currentBody,
    }: {
      prompt: string;
      language: ScriptLanguage;
      currentBody: string;
    }): Promise<DraftScriptResult | undefined> => {
      running.current?.abort();
      const abort = new AbortController();
      running.current = abort;
      setPartial("");
      let result = create(DraftScriptResultSchema);

      try {
        for await (const { event } of createClient(AssistantService, transport).draftScript(
          { prompt, language, currentBody },
          { signal: abort.signal },
        )) {
          if (event.case === "text") {
            const piece = event.value;
            setPartial((written) => written + piece);
          } else if (event.case === "result") {
            result = event.value;
          }
        }
      } catch (failure) {
        if (abort.signal.aborted) {
          return undefined;
        }
        throw failure;
      }

      return result;
    },
  });

  return {
    draft: mutation.mutateAsync,
    drafting: mutation.isPending,
    error: mutation.error,
    partial,
    stop: () => running.current?.abort(),
  };
}
