import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const assistantSettingsMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    assistantSettings: {
      title: "AI 助手",
      description: "用一句话生成流程和脚本草稿；生成时会把输入内容发送给所选服务",
      loading: "正在加载 AI 助手设置…",
      ready: "可用",
      notReady: "不可用",
      provider: "提供方",
      firstReady: "首个可用的提供方",
      endpoint: "OpenAI 兼容接口",
      presets: {
        openai: "OpenAI",
        deepseek: "DeepSeek",
        qwen: "通义千问",
        ollama: "Ollama（本机）",
      },
      baseUrl: "地址",
      model: "模型",
      apiKey: "密钥",
      keyKept: "已保存，留空则保持不变",
      keyMissing: "未保存密钥",
      keyNote:
        "密钥只保存在本机，仅当前用户可读，不会显示；本机服务无需密钥；换用其他服务地址后需重新输入",
      structuredOutput: "结构化输出",
      save: "保存",
      clearKey: "清除密钥",
      confirmClearKey: "清除已保存的密钥？",
      confirmClearKeyBody: "保存新的密钥之前，AI 草稿无法使用",
      saved: "已保存",
      failed: "保存失败",
    },
  },
  "en-US": {
    assistantSettings: {
      title: "AI assistant",
      description:
        "Drafts flows and scripts from one sentence. What you type is sent to the selected service",
      loading: "Loading AI assistant settings…",
      ready: "Ready",
      notReady: "Not ready",
      provider: "Provider",
      firstReady: "First ready provider",
      endpoint: "OpenAI-compatible API",
      presets: {
        openai: "OpenAI",
        deepseek: "DeepSeek",
        qwen: "Qwen",
        ollama: "Ollama (this computer)",
      },
      baseUrl: "Base URL",
      model: "Model",
      apiKey: "API key",
      keyKept: "Saved. Leave empty to keep it",
      keyMissing: "No key saved",
      keyNote:
        "The key is stored only on this computer, readable only by the current user, and never shown. Local services need no key. Switching to another service needs the key again",
      structuredOutput: "Structured output",
      save: "Save",
      clearKey: "Clear key",
      confirmClearKey: "Clear the saved key?",
      confirmClearKeyBody: "AI drafts stop working until a new key is saved",
      saved: "Saved",
      failed: "Could not save",
    },
  },
};
