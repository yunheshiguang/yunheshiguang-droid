import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const runAutomationMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    runAutomation: {
      title: "运行流程",
      description: "逐步运行，任一步失败即停止；关闭页面不会中断",
      noFlows: "暂无流程，请先新建",
      flow: "流程",
      noDevices: "请先在设备墙选择设备",
      selected: "设备墙上已选 {{count}} 台设备",
      run: "运行",
      stop: "当前步骤结束后停止",
      confirm: {
        title: "运行“{{name}}”？",
        body: "以下步骤会改动设备上的数据，且无法撤销：{{steps}}",
      },
      progress: "运行进度",
      iteration: "“{{name}}”第 {{iteration}}/{{iterations}} 次",
      history: {
        title: "最近运行",
        empty: "暂无运行记录",
        devices: "{{count}} 台设备",
        operator: "操作者 {{name}}",
        replay: "再次运行",
        replayNamed: "按原设置再次运行“{{name}}”",
      },
    },
  },
  "en-US": {
    runAutomation: {
      title: "Run a flow",
      description:
        "Runs step by step and stops at the first failed step. Closing this page does not stop it",
      noFlows: "No flows yet. Create one first",
      flow: "Flow",
      noDevices: "Select devices on the device wall first",
      selected_one: "{{count}} device selected on the device wall",
      selected_other: "{{count}} devices selected on the device wall",
      run: "Run",
      stop: "Stop after this step",
      confirm: {
        title: "Run “{{name}}”?",
        body: "These steps change data on the devices and cannot be undone: {{steps}}",
      },
      progress: "Run progress",
      iteration: "“{{name}}”, run {{iteration}} of {{iterations}}",
      history: {
        title: "Recent runs",
        empty: "No runs yet",
        devices_one: "{{count}} device",
        devices_other: "{{count}} devices",
        operator: "by {{name}}",
        replay: "Run again",
        replayNamed: "Run “{{name}}” again with the same settings",
      },
    },
  },
};
