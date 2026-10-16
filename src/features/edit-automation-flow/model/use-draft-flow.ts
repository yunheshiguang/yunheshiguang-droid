import { createClient } from "@connectrpc/connect";
import { useTransport } from "@connectrpc/connect-query";
import { useMutation } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import type { AutomationFlow } from "@/entities/automation";
import { workspaceMutationKeys } from "@/shared/api";
import { AssistantService, type DraftFlowResult, type UiText } from "@/shared/proto";

export type ReceivedFlowDraft = {
  flow: AutomationFlow;
  issues: { step: number; message?: UiText }[];
  model: string;
};

export function useDraftFlow() {
  const transport = useTransport();
  const [partial, setPartial] = useState("");
  const running = useRef<AbortController | undefined>(undefined);
  useEffect(() => () => running.current?.abort(), []);
  const mutation = useMutation({
    mutationKey: workspaceMutationKeys.assistantDraft(),
    mutationFn: async (
      prompt: string,
    ): Promise<{ received?: ReceivedFlowDraft; refused?: UiText }> => {
      running.current?.abort();
      const abort = new AbortController();
      running.current = abort;
      setPartial("");
      let result: DraftFlowResult | undefined;

      try {
        for await (const { event } of createClient(AssistantService, transport).draftFlow(
          { prompt },
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
          return {};
        }
        throw failure;
      }

      const flow = result?.draft?.flow;

      if (!result?.accepted || !result.draft || !flow) {
        return { refused: result?.detail };
      }

      return {
        received: {
          flow: {
            id: crypto.randomUUID(),
            name: flow.name,
            steps: flow.steps.map(({ id, kind, value }) => ({ id, kind, value })),
          },
          issues: result.draft.issues.map((issue) => ({
            step: flow.steps.findIndex((step) => step.id === issue.stepId) + 1,
            message: issue.message,
          })),
          model: result.draft.model,
        },
      };
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
