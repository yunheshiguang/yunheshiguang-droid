import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const taskMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    task: {
      actions: {
        screenshot: "截图",
        screenRecord: "录屏",
        installApk: "安装 APK",
        uploadFile: "上传文件",
        inputText: "输入文字",
        clipboardWrite: "写入剪贴板",
        launchApp: "启动应用",
        stopApp: "停止应用",
        clearAppData: "清除应用数据",
        uninstallApp: "卸载应用",
        recoverConnection: "恢复连接",
        runScript: "运行脚本",
      },
      inputs: {
        text: "输入要发送的文字",
        packageName: "输入应用包名",
        sourcePath: "输入文件路径",
        savePath: "本地输出目录",
        script: "从脚本库选择脚本",
      },
    },
  },
  "en-US": {
    task: {
      actions: {
        screenshot: "Screenshot",
        screenRecord: "Screen record",
        installApk: "Install APK",
        uploadFile: "Upload file",
        inputText: "Input text",
        clipboardWrite: "Write clipboard",
        launchApp: "Launch app",
        stopApp: "Stop app",
        clearAppData: "Clear app data",
        uninstallApp: "Uninstall app",
        recoverConnection: "Recover connection",
        runScript: "Run script",
      },
      inputs: {
        text: "Enter text to send",
        packageName: "Enter package name",
        sourcePath: "Enter file path",
        savePath: "Local output directory",
        script: "Select a script from the library",
      },
    },
  },
};
