import type { DescService } from "@bufbuild/protobuf";
import { createClient, createRouterTransport } from "@connectrpc/connect";

import type { CommandPayload } from "@/shared/api";

import { createMockBackend } from "./mock-backend";

export function mockBackendHarness() {
  const { handlers, services } = createMockBackend({ latencyMs: 0 });
  const transport = createRouterTransport(services);
  const call = <T>(command: keyof typeof handlers, request?: unknown) =>
    handlers[command](
      request === undefined ? undefined : ({ request } as CommandPayload),
    ) as Promise<T>;

  return Object.assign(call, {
    client: <S extends DescService>(service: S) => createClient(service, transport),
  });
}
