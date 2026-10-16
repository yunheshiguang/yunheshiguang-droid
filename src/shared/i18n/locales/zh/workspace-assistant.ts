export const assistantWorkspaceZh = {
  openai: {
    label: "OpenAI 兼容接口",
    description: "OpenAI、DeepSeek、通义千问、Ollama 等兼容 OpenAI 接口的服务",
  },
  invalid: {
    baseUrl: "请输入以 http:// 或 https:// 开头的地址",
    modelMissing: "请输入模型名称",
    keyMissing: "请输入密钥（本机服务无需密钥）",
    key: "密钥只能包含字母、数字和标点",
    providerUnknown: "未知的 AI 提供方",
    settingTooLong: "设置项最多 {{max}} 个字符",
    promptRequired: "请描述要做什么",
    promptTooLong: "描述最多 {{max}} 个字符",
  },
  noProvider: "没有可用的 AI 提供方",
  requestFailed: "AI 服务请求失败：{{message}}",
  refused: "模型拒绝了请求：{{message}}",
  emptyAnswer: "模型未返回回答",
  unreadable: "无法解析模型的回答：{{message}}",
  storageFailed: "AI 助手设置保存失败：{{message}}",
  draft: {
    empty: "草稿中没有可用的步骤",
    tooLong: "草稿过长，仅保留前 {{max}} 步",
    unknownAction: "第 {{step}} 步使用了未知操作，已移除",
    scriptMissing: "此步骤引用的脚本不在脚本库中",
  },
  script: {
    empty: "模型生成的脚本为空",
    tooLong: "脚本最多 {{max}} 个字符",
  },
  audit: {
    updated: "修改 AI 助手设置",
    flow: "使用 {{model}} 生成流程草稿",
    script: "使用 {{model}} 生成脚本草稿",
  },
} as const;
