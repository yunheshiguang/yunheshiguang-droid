import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const settingsMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    settings: {
      title: "设置",
      theme: {
        title: "外观",
      },
      locale: {
        title: "语言",
      },
      updates: {
        title: "更新",
        installed: "当前版本",
        automatic: "每天自动检查更新",
        checkNow: "立即检查",
        allReleases: "所有版本",
        download: "下载 {{name}}",
        releaseNotes: "更新说明",
        noDownload: "没有与当前安装方式对应的安装包，请在更新说明页面选择下载",
        status: {
          idle: "尚未检查",
          checking: "正在检查…",
          latest: "已是最新版本",
          upToDate: "已是最新版本 · {{time}} 检查",
          failed: "检查更新失败",
          localBuild: "这份应用不是由发布流程构建的，不会检查更新",
        },
      },
    },
  },
  "en-US": {
    settings: {
      title: "Settings",
      theme: {
        title: "Appearance",
      },
      locale: {
        title: "Language",
      },
      updates: {
        title: "Updates",
        installed: "Installed version",
        automatic: "Check for updates every day",
        checkNow: "Check now",
        allReleases: "All releases",
        download: "Download {{name}}",
        releaseNotes: "Release notes",
        noDownload: "No file matches how this copy was installed. Pick one on the release page",
        status: {
          idle: "Not checked yet",
          checking: "Checking…",
          latest: "Up to date",
          upToDate: "Up to date · checked {{time}}",
          failed: "Could not check for updates",
          localBuild:
            "This copy was not built by the release workflows, so it does not check for updates",
        },
      },
    },
  },
};
