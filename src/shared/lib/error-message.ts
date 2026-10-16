import { ConnectError } from "@connectrpc/connect";

export function errorMessage(error: unknown): string {
  if (error instanceof ConnectError) {
    return error.rawMessage;
  }

  return error instanceof Error ? error.message : String(error);
}
