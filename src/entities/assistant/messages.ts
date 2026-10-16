import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const assistantMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    assistant: {
      draft: "生成草稿",
      drafting: "正在生成…",
      stop: "停止",
      review: "草稿不会自动保存或运行，请检查后保存",
      notReady: "AI 助手不可用，请在设置中配置",
      byModel: "由 {{model}} 生成",
    },
  },
  "en-US": {
    assistant: {
      draft: "Draft",
      drafting: "Drafting…",
      stop: "Stop",
      review: "Drafts are not saved or run automatically. Review before saving",
      notReady: "The AI assistant is not ready. Configure it in Settings",
      byModel: "Drafted by {{model}}",
    },
  },
};
