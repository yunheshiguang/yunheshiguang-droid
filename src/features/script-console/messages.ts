import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const scriptConsoleMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    scriptConsole: {
      newName: "新脚本",
      unknownDevice: "未知设备",
      pick: "从脚本库选择脚本，或新建脚本",
      ai: {
        write: "用 AI 生成脚本",
        change: "用 AI 修改此脚本",
        placeholder: "例如：输出每台设备的电量和 Android 版本",
        writing: "AI 正在生成的脚本",
      },
      noDevices: "请先在设备墙选择设备",
      target: "将在选中的 {{count}} 台设备上运行",
      run: "运行",
      save: "保存",
      remove: "删除脚本",
      confirmRemove: "删除脚本“{{name}}”？",
      unsaved: "有未保存的改动，运行时会先保存",
      results: "运行结果",
      library: {
        title: "脚本库",
        empty: "暂无脚本",
      },
      editor: {
        title: "脚本控制台",
        name: "脚本名称",
        language: "语言",
        body: "脚本内容",
      },
      engines: {
        title: "脚本引擎",
        capturesOutput: "返回输出和退出码",
        launchOnly: "仅启动脚本，结果留在设备上",
      },
    },
  },
  "en-US": {
    scriptConsole: {
      newName: "New script",
      unknownDevice: "Unknown device",
      pick: "Select a script from the library, or create one",
      ai: {
        write: "Draft a script with AI",
        change: "Edit this script with AI",
        placeholder: "For example: print each device's battery level and Android version",
        writing: "Script being drafted by AI",
      },
      noDevices: "Select devices on the device wall first",
      target_one: "Runs on {{count}} selected device",
      target_other: "Runs on {{count}} selected devices",
      run: "Run",
      save: "Save",
      remove: "Delete script",
      confirmRemove: "Delete the script “{{name}}”?",
      unsaved: "Unsaved changes. Running saves them first",
      results: "Run results",
      library: {
        title: "Script library",
        empty: "No scripts yet",
      },
      editor: {
        title: "Script console",
        name: "Script name",
        language: "Language",
        body: "Script",
      },
      engines: {
        title: "Script engines",
        capturesOutput: "Returns output and exit code",
        launchOnly: "Only starts the script. Results stay on the device",
      },
    },
  },
};
