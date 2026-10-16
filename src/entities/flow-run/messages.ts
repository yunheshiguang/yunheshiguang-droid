import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const flowRunMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    flowRun: {
      target: {
        label: "运行目标",
        modes: {
          devices: "指定设备",
          group: "指定分组",
          tag: "指定标签",
        },
        group: "分组",
        tag: "标签",
        pickGroup: "选择分组",
        missingGroup: "已不存在的分组",
        pickTag: "选择标签",
        noTags: "暂无标签，可在设备墙添加",
        onlineOnly: "仅包含开始运行时在线的设备",
      },
      options: {
        label: "运行选项",
        repeat: "运行次数",
        repeatIntervalSeconds: "运行间隔（秒）",
        retryAttempts: "失败设备重试次数",
        retryDelaySeconds: "重试间隔（秒）",
      },
      stepLabel: "第 {{index}} 步",
      summary: "{{succeeded}}/{{count}} 台设备成功",
      attempts: "尝试 {{count}} 次",
    },
  },
  "en-US": {
    flowRun: {
      target: {
        label: "Run on",
        modes: {
          devices: "Specific devices",
          group: "A group",
          tag: "A tag",
        },
        group: "Group",
        tag: "Tag",
        pickGroup: "Select a group",
        missingGroup: "A group that no longer exists",
        pickTag: "Select a tag",
        noTags: "No tags yet. Add one on the device wall",
        onlineOnly: "Only devices online when the run starts are included",
      },
      options: {
        label: "Run options",
        repeat: "Times to run",
        repeatIntervalSeconds: "Seconds between runs",
        retryAttempts: "Retries for failed devices",
        retryDelaySeconds: "Seconds before a retry",
      },
      stepLabel: "Step {{index}}",
      summary_one: "{{succeeded}} of {{count}} device succeeded",
      summary_other: "{{succeeded}} of {{count}} devices succeeded",
      attempts_one: "{{count}} attempt",
      attempts_other: "{{count}} attempts",
    },
  },
};
