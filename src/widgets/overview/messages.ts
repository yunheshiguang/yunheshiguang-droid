import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const overviewMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    overview: {
      sectionLabel: "总览",
      title: "设备、任务和告警",
      metrics: {
        devices: "在线设备",
        devicesNote: "已连接到本机",
        tasks: "最近任务",
        tasksNote: "本窗口运行过的批量操作",
        blocked: "需处理",
        blockedNote: "需要人工处理的告警",
        groups: "设备分组",
        groupsNote: "在设备墙中建立的分组",
      },
      empty: {
        title: "未检测到设备",
        description: "请连接设备并开启 USB 调试，或确认已安装 adb",
      },
      devices: {
        title: "设备列表",
      },
      diagnostics: {
        title: "告警",
        empty: "暂无告警",
      },
    },
  },
  "en-US": {
    overview: {
      sectionLabel: "Overview",
      title: "Devices, tasks, and alerts",
      metrics: {
        devices: "Online devices",
        devicesNote: "Connected to this computer",
        tasks: "Recent tasks",
        tasksNote: "Batch operations run in this window",
        blocked: "Needs attention",
        blockedNote: "Alerts that need manual action",
        groups: "Device groups",
        groupsNote: "Groups set up on the device wall",
      },
      empty: {
        title: "No devices detected",
        description: "Connect a device and turn on USB debugging, or make sure adb is installed",
      },
      devices: {
        title: "Device list",
      },
      diagnostics: {
        title: "Alerts",
        empty: "No alerts",
      },
    },
  },
};
