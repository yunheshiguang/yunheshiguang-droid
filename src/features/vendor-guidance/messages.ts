import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const vendorGuidanceMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    vendorGuidance: {
      title: "厂商设置",
      loading: "正在识别设备厂商…",
      failed: "无法识别设备厂商",
      empty: "已连接的设备无需厂商设置",
      devices: "设备：{{names}}",
    },
  },
  "en-US": {
    vendorGuidance: {
      title: "Vendor settings",
      loading: "Identifying device vendors…",
      failed: "Could not identify device vendors",
      empty: "No connected device needs vendor settings",
      devices: "Devices: {{names}}",
    },
  },
};
