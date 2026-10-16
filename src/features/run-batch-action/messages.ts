import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const runBatchActionMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    runBatchAction: {
      running: "正在运行…",
      title: "批量操作结果",
      summary: "{{succeeded}}/{{count}} 台设备成功",
      requestFailed: "请求失败",
      replay: "再次运行",
      confirmReplay: "在 {{count}} 台设备上再次运行“{{action}}”？",
      unknownDevice: "已断开的设备",
      succeeded: "成功",
      failed: "失败",
      via: "经由 {{channel}}",
    },
  },
  "en-US": {
    runBatchAction: {
      running: "Running…",
      title: "Batch result",
      summary_one: "{{succeeded}} of {{count}} device succeeded",
      summary_other: "{{succeeded}} of {{count}} devices succeeded",
      requestFailed: "Request failed",
      replay: "Run again",
      confirmReplay_one: "Run “{{action}}” again on {{count}} device?",
      confirmReplay_other: "Run “{{action}}” again on {{count}} devices?",
      unknownDevice: "Disconnected device",
      succeeded: "Succeeded",
      failed: "Failed",
      via: "Via {{channel}}",
    },
  },
};
