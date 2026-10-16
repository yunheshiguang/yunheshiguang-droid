import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

export const useFlowTemplateMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    useFlowTemplate: {
      loading: "正在加载模板…",
      loadFailed: "无法加载模板",
      empty: "暂无可用模板",
      use: "使用",
      useNamed: "用模板“{{name}}”新建流程",
      added: "已新建流程“{{name}}”，请在流程编辑器中填写各步骤的值",
      addFailed: "无法从模板新建流程",
    },
  },
  "en-US": {
    useFlowTemplate: {
      loading: "Loading templates…",
      loadFailed: "Could not load templates",
      empty: "No templates available",
      use: "Use",
      useNamed: "Create a flow from the “{{name}}” template",
      added: "Created flow “{{name}}”. Fill in each step's value in the flow editor",
      addFailed: "Could not create a flow from the template",
    },
  },
};
