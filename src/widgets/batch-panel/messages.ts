import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const batchPanelMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    batchPanel: {
      title: "批量操作",
      run: "运行",
      confirm: {
        body: "“{{action}}”将在 {{count}} 台设备上运行，此操作不可撤销",
        run: "确认运行",
      },
      selection: {
        title: "已选设备",
        description: "共 {{count}} 台",
        empty: "请先在设备墙选择设备",
      },
    },
  },
  "en-US": {
    batchPanel: {
      title: "Batch operations",
      run: "Run",
      confirm: {
        body_one: "“{{action}}” will run on {{count}} device. This cannot be undone",
        body_other: "“{{action}}” will run on {{count}} devices. This cannot be undone",
        run: "Run anyway",
      },
      selection: {
        title: "Selected devices",
        description_one: "{{count}} device",
        description_other: "{{count}} devices",
        empty: "Select devices on the device wall first",
      },
    },
  },
};
