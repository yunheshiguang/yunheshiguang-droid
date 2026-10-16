import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const scriptMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    script: {
      untitled: "未命名脚本",
      picker: {
        label: "脚本",
        placeholder: "选择脚本",
        empty: "暂无脚本，可在脚本控制台添加",
      },
      output: {
        label: "{{device}} 的输出",
        via: "由 {{engine}} 运行",
        exited: "退出码 {{code}}",
        launched: "已启动，结果保留在设备上",
        notRun: "未运行",
        truncated: "输出过长，仅保留末尾",
      },
    },
  },
  "en-US": {
    script: {
      untitled: "Untitled script",
      picker: {
        label: "Script",
        placeholder: "Select a script",
        empty: "No scripts yet. Add one in the script console",
      },
      output: {
        label: "Output from {{device}}",
        via: "Run by {{engine}}",
        exited: "Exit code {{code}}",
        launched: "Started. Results stay on the device",
        notRun: "Did not run",
        truncated: "Output too long. Only the end is kept",
      },
    },
  },
};
