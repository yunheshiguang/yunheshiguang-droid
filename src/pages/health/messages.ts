import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const healthPageMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    health: {
      loading: "正在加载健康数据…",
      failed: "无法加载健康数据",
      availability: {
        title: "设备在线情况",
        online: "{{online}} / {{devices}} 台在线",
        unauthorized: "{{count}} 台待授权",
        offline: "{{count}} 台离线",
        probeTook: "上次探测耗时 {{ms}} 毫秒",
        noProbes: "暂无探测记录",
        trend: "最近 {{count}} 次探测的在线率",
      },
      operations: {
        title: "操作",
        description: "应用启动以来",
        empty: "暂无操作",
        operation: "操作",
        runs: "次数",
        failures: "失败",
        average: "平均",
        slowest: "最慢",
        milliseconds: "{{ms}} 毫秒",
      },
      sinks: {
        title: "指标去向",
      },
    },
  },
  "en-US": {
    health: {
      loading: "Loading health data…",
      failed: "Could not load health data",
      availability: {
        title: "Device availability",
        online: "{{online}} of {{devices}} online",
        unauthorized: "{{count}} waiting for authorization",
        offline: "{{count}} offline",
        probeTook: "Last probe took {{ms}} ms",
        noProbes: "No probes yet",
        trend_one: "Availability at the last probe",
        trend_other: "Availability over the last {{count}} probes",
      },
      operations: {
        title: "Operations",
        description: "Since the app started",
        empty: "No operations yet",
        operation: "Operation",
        runs: "Runs",
        failures: "Failed",
        average: "Average",
        slowest: "Slowest",
        milliseconds: "{{ms}} ms",
      },
      sinks: {
        title: "Metrics destinations",
      },
    },
  },
};
