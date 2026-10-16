import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const releaseMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    release: {
      channel: {
        nightly: "每日构建",
        release: "正式版",
        local: "本地构建",
      },
      commit: "提交 {{commit}}",
      available: {
        nightly: "有新的每日构建：{{commit}}",
        release: "新版本 {{version}} 已发布",
      },
      badge: "有可用更新",
      badgeTitle: "打开 {{version}} 的发布页面",
    },
  },
  "en-US": {
    release: {
      channel: {
        nightly: "Nightly",
        release: "Release",
        local: "Local build",
      },
      commit: "Commit {{commit}}",
      available: {
        nightly: "A newer nightly build is out: {{commit}}",
        release: "Version {{version}} is out",
      },
      badge: "Update available",
      badgeTitle: "Open the release page for {{version}}",
    },
  },
};
