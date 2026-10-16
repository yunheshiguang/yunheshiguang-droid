import {
  createConnectQueryKey,
  useMutation,
  useQuery,
  useTransport,
} from "@connectrpc/connect-query";
import { useQueryClient } from "@tanstack/react-query";

import { workspaceMutationKeys } from "@/shared/api";
import {
  type AssistantResult,
  AssistantService,
  type AssistantSettings,
  StructuredOutput,
} from "@/shared/proto";

export const assistantPresets = [
  {
    id: "openai",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1-mini",
    structuredOutput: StructuredOutput.JSON_SCHEMA,
  },
  {
    id: "deepseek",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    structuredOutput: StructuredOutput.JSON_OBJECT,
  },
  {
    id: "qwen",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-plus",
    structuredOutput: StructuredOutput.JSON_OBJECT,
  },
  {
    id: "ollama",
    baseUrl: "http://localhost:11434/v1",
    model: "qwen2.5",
    structuredOutput: StructuredOutput.JSON_SCHEMA,
  },
] as const;

const getAssistant = AssistantService.method.getAssistant;

export function useAssistantSettings() {
  const queryClient = useQueryClient();
  const transport = useTransport();
  const { data: state } = useQuery(getAssistant, {});
  const save = useMutation(AssistantService.method.updateAssistant, {
    mutationKey: workspaceMutationKeys.assistant(),
    onSuccess: (result: AssistantResult) => {
      if (result.state) {
        queryClient.setQueryData(
          createConnectQueryKey({
            schema: getAssistant,
            input: {},
            transport,
            cardinality: "finite",
          }),
          result.state,
        );
      }
    },
  });

  return {
    state,
    save: ({ settings, apiKey }: { settings: AssistantSettings; apiKey?: string }) =>
      save.mutateAsync({ settings, apiKey }),
    saving: save.isPending,
    error: save.error,
  };
}
