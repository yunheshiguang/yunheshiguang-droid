import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const manageApiGatewayMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    manageApiGateway: {
      title: "本机 API",
      description: "只监听本机，每个请求都需要令牌；变更操作受授权策略约束并记入审计",
      enable: "开启本机 API",
      port: "端口",
      apply: "应用",
      authenticators: "认证方式",
      status: {
        running: "正在监听 {{address}}",
        stopped: "已关闭",
        failed: "监听失败",
      },
      tokens: {
        title: "令牌",
        description: "请求头带上 Authorization: Bearer <令牌>；只保存令牌的摘要",
        name: "用途",
        create: "签发令牌",
        secret: "新令牌",
        secretOnce: "令牌只显示一次，请立即复制",
        copy: "复制令牌",
        empty: "暂无令牌",
        created: "签发于 {{time}}",
        lastUsed: "最近使用 {{time}}",
        neverUsed: "未使用",
        revoke: "吊销令牌“{{name}}”",
        confirmRevoke: "吊销令牌“{{name}}”？",
        confirmRevokeBody: "使用这个令牌的客户端会立即失去访问权限",
        revokeAction: "吊销",
      },
      usage: {
        title: "调用示例",
      },
    },
  },
  "en-US": {
    manageApiGateway: {
      title: "Local API",
      description:
        "Listens on this computer only, and every request needs a token. Changes follow the authorization policy and are recorded in the audit trail",
      enable: "Turn on the local API",
      port: "Port",
      apply: "Apply",
      authenticators: "Authentication methods",
      status: {
        running: "Listening on {{address}}",
        stopped: "Off",
        failed: "Could not start listening",
      },
      tokens: {
        title: "Tokens",
        description:
          "Send Authorization: Bearer <token> in the request header. Only a digest of each token is stored",
        name: "Purpose",
        create: "Issue token",
        secret: "New token",
        secretOnce: "This token is shown only once. Copy it now",
        copy: "Copy token",
        empty: "No tokens yet",
        created: "Issued {{time}}",
        lastUsed: "Last used {{time}}",
        neverUsed: "Never used",
        revoke: "Revoke token “{{name}}”",
        confirmRevoke: "Revoke the token “{{name}}”?",
        confirmRevokeBody: "Clients using this token lose access immediately",
        revokeAction: "Revoke",
      },
      usage: {
        title: "Example request",
      },
    },
  },
};
