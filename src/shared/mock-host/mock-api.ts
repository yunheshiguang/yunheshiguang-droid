import { create } from "@bufbuild/protobuf";
import { timestampNow } from "@bufbuild/protobuf/wkt";
import type { ServiceImpl } from "@connectrpc/connect";

import {
  type ApiGatewayService,
  ApiGatewayStatus,
  type ApiToken,
  ApiTokenSchema,
} from "@/shared/proto";

import type { MockContext } from "./mock-context";

const DEFAULT_PORT = 47831;

export function createMockApiGateway(context: MockContext): ServiceImpl<typeof ApiGatewayService> {
  let config = { enabled: false, port: 0 };
  const tokens: ApiToken[] = [];

  const state = () => ({
    config,
    status: config.enabled ? ApiGatewayStatus.RUNNING : ApiGatewayStatus.STOPPED,
    address: config.enabled ? `http://127.0.0.1:${config.port || DEFAULT_PORT}` : "",
    tokens: [...tokens],
    authenticators: [
      {
        id: "core.tokens",
        label: { key: "workspace.api.tokens.label" },
        description: { key: "workspace.api.tokens.description" },
      },
    ],
  });
  const result = (secret = "") => context.settle({ accepted: true, state: state(), secret });
  const refused = (key: string, params?: Record<string, string>) =>
    context.settle({ accepted: false, detail: { key, params }, state: state() });

  return {
    getApiGateway: () => context.settle(state()),
    updateApiGateway: (request) => {
      const port = request.config?.port ?? 0;

      if (port !== 0 && (port < 1024 || port > 65_535)) {
        return refused("workspace.api.invalid.port", { min: "1024" });
      }
      config = { enabled: Boolean(request.config?.enabled), port };
      context.audit(
        "GOVERNED_ACTION_MANAGE_API",
        true,
        [],
        config.enabled
          ? { key: "workspace.api.audit.enabled", params: { port: String(port || DEFAULT_PORT) } }
          : { key: "workspace.api.audit.disabled" },
      );
      return result();
    },
    createApiToken: (request) => {
      const name = request.name.trim();

      if (!name) {
        return refused("workspace.api.invalid.tokenNameRequired");
      }
      tokens.push(
        create(ApiTokenSchema, { id: crypto.randomUUID(), name, createdAt: timestampNow() }),
      );
      context.audit("GOVERNED_ACTION_MANAGE_API", true, [], {
        key: "workspace.api.audit.tokenCreated",
        params: { name },
      });
      return result(`yhsg_mock_${crypto.randomUUID().replaceAll("-", "")}`);
    },
    revokeApiToken: (request) => {
      const index = tokens.findIndex((token) => token.id === request.id);

      if (index < 0) {
        return refused("workspace.api.invalid.tokenMissing");
      }
      const [removed] = tokens.splice(index, 1);
      context.audit("GOVERNED_ACTION_MANAGE_API", true, [], {
        key: "workspace.api.audit.tokenRevoked",
        params: { name: removed?.name ?? "" },
      });
      return result();
    },
  };
}
