import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const auditMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    audit: {
      empty: "本次启动后暂无操作记录",
      unknownTime: "时间未知",
      operator: "操作者 {{operator}}",
      actor: "来自 {{actor}}",
      targets: "对象 {{targets}}",
      groups: "分组 {{groups}}",
      capacity: "仅保留最近 {{count}} 条，更早的记录见日志",
    },
  },
  "en-US": {
    audit: {
      empty: "No actions recorded since the app started",
      unknownTime: "Time unknown",
      operator: "By {{operator}}",
      actor: "From {{actor}}",
      targets: "On {{targets}}",
      groups: "In {{groups}}",
      capacity_one: "Keeps the latest {{count}} action. Older ones are in the log",
      capacity_other: "Keeps the latest {{count}} actions. Older ones are in the log",
    },
  },
};
