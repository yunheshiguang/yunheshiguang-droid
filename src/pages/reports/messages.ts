import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const reportsPageMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    reports: {
      period: "时间范围",
      periods: {
        week: "最近 7 天",
        month: "最近 30 天",
        quarter: "最近 90 天",
      },
      loading: "正在加载报表…",
      failed: "无法加载报表",
      counted: "共 {{count}} 次操作",
      truncated: "审计记录过多，仅统计最近 {{count}} 条",
      empty: "该时间范围内暂无操作",
      columns: {
        name: "名称",
        total: "次数",
        failed: "失败",
        denied: "被拒",
      },
      byDay: {
        title: "按天",
        bar: "{{day}}：{{count}} 次，失败 {{failed}} 次",
      },
      byOperator: {
        title: "按操作者",
      },
      byGroup: {
        title: "按分组",
        description: "一次操作涉及同组多台设备时只计一次",
      },
      byAction: {
        title: "按操作",
      },
    },
  },
  "en-US": {
    reports: {
      period: "Period",
      periods: {
        week: "Last 7 days",
        month: "Last 30 days",
        quarter: "Last 90 days",
      },
      loading: "Loading the report…",
      failed: "Could not load the report",
      counted_one: "{{count}} operation in total",
      counted_other: "{{count}} operations in total",
      truncated: "Too many audit entries; counted only the latest {{count}}",
      empty: "No operations in this period",
      columns: {
        name: "Name",
        total: "Runs",
        failed: "Failed",
        denied: "Denied",
      },
      byDay: {
        title: "By day",
        bar_one: "{{day}}: {{count}} run, {{failed}} failed",
        bar_other: "{{day}}: {{count}} runs, {{failed}} failed",
      },
      byOperator: {
        title: "By operator",
      },
      byGroup: {
        title: "By group",
        description: "An operation counts once per group, however many of its devices are in it",
      },
      byAction: {
        title: "By action",
      },
    },
  },
};
