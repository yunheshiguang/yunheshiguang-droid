import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const inputGuidanceMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    inputGuidance: {
      title: "输入通道",
      description: "每台设备使用排在最前的可用通道；中文、表情和引号原样发送",
      loading: "正在加载输入通道…",
      failed: "无法加载输入通道",
      none: "未安装输入通道，输入文字和写入剪贴板都会失败",
    },
  },
  "en-US": {
    inputGuidance: {
      title: "Input channels",
      description:
        "Each device uses the first channel that can serve it. Chinese, emoji, and quotes are sent as typed",
      loading: "Loading input channels…",
      failed: "Could not load input channels",
      none: "No input channel is installed. Typing and clipboard writes will fail",
    },
  },
};
