import { useQuery } from "@tanstack/react-query";

import { commandGateway, workspaceQueryKeys } from "@/shared/api";
import { errorMessage } from "@/shared/lib/error-message";

type CommandArgs = Readonly<Record<string, unknown>>;

const NO_ARGS: CommandArgs = {};

export function invokeExtensionCommand<TResult>(
  plugin: string,
  command: string,
  args: CommandArgs = NO_ARGS,
): Promise<TResult> {
  return commandGateway().invoke<TResult>(`plugin:${plugin}|${command}`, { ...args });
}

export type ExtensionQuery<TResult> = {
  data?: TResult;
  loading: boolean;
  error: string | null;
  refetch: () => void;
};

export function useExtensionQuery<TResult>(
  plugin: string,
  command: string,
  args: CommandArgs = NO_ARGS,
): ExtensionQuery<TResult> {
  const { data, isPending, error, refetch } = useQuery({
    queryKey: workspaceQueryKeys.extension(plugin, command, args),
    queryFn: () => invokeExtensionCommand<TResult>(plugin, command, args),
  });

  return {
    data,
    loading: isPending,
    error: error == null ? null : errorMessage(error),
    refetch: () => {
      void refetch();
    },
  };
}
