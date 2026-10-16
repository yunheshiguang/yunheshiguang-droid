import { Building2 } from "lucide-react";
import {
  lazyModulePage,
  type WorkspaceModuleDefinition,
  workspaceNavigationGroupIds,
} from "yunheshiguang/sdk";

export const enterpriseModule: WorkspaceModuleDefinition = {
  id: "enterprise",
  component: lazyModulePage(
    () => import("./enterprise-page"),
    (page) => page.EnterprisePage,
  ),
  navigation: {
    groupId: workspaceNavigationGroupIds.riskSurface,
    order: 90,
    labelKey: "modules.enterprise.label",
    descriptionKey: "modules.enterprise.description",
    icon: Building2,
  },
  requires: ["reference.enterprise"],
  messages: {
    "zh-CN": {
      modules: { enterprise: { label: "企业", description: "许可、团队、策略和同步" } },
      enterprise: {
        loading: "正在加载扩展状态…",
        loadFailed: "无法加载扩展状态",
        license: {
          title: "许可",
          none: "未安装许可",
          licensee: "授权给 {{name}}",
          expires: "到期 {{date}}",
          perpetual: "永久有效",
        },
        team: {
          operator: "操作者：{{name}}",
          members: "团队名单中有 {{count}} 位成员",
          none: "未设置团队名单，操作者为本机登录用户",
          broken: "无法读取团队名单，所有人都按无角色的访客处理",
        },
        policy: {
          title: "团队和策略",
          description: "被拒绝的操作也会记入审计记录",
          rules: "策略中有 {{count}} 条规则，没有规则允许的操作会被拒绝",
          none: "未设置策略文件，允许所有操作",
          broken: "无法读取策略文件，拒绝所有操作",
        },
        files: {
          title: "文件",
          description: "将文档目录设在同步盘中，即可在多台电脑间同步",
          team: "团队名单",
          policy: "策略文件",
          license: "许可文件",
          audit: "团队审计目录",
          sync: "同步设置",
          documents: "文档目录",
          assistant: "AI 提供方设置",
          redroid: "Redroid 云手机设置",
          vendors: "主板机规则",
          metrics: "指标文件",
          apiKeys: "企业 API 密钥清单",
        },
        state: {
          missing: "未设置",
          loaded: "已生效",
          broken: "无法读取：{{detail}}",
        },
      },
    },
    "en-US": {
      modules: {
        enterprise: { label: "Enterprise", description: "License, team, policy, and sync" },
      },
      enterprise: {
        loading: "Loading the extension status…",
        loadFailed: "Could not load the extension status",
        license: {
          title: "License",
          none: "No license installed",
          licensee: "Issued to {{name}}",
          expires: "Expires {{date}}",
          perpetual: "Does not expire",
        },
        team: {
          operator: "Operator: {{name}}",
          members_one: "The team roster has {{count}} member",
          members_other: "The team roster has {{count}} members",
          none: "No team roster; the operator is the user signed in to this computer",
          broken: "Could not read the team roster; everyone is treated as a guest without roles",
        },
        policy: {
          title: "Team and policy",
          description: "Refused actions are also recorded in the audit trail",
          rules_one: "The policy has {{count}} rule. Actions that no rule allows are refused",
          rules_other: "The policy has {{count}} rules. Actions that no rule allows are refused",
          none: "No policy file; all actions are allowed",
          broken: "Could not read the policy file; all actions are refused",
        },
        files: {
          title: "Files",
          description: "Put the documents folder on a synced drive to share it between computers",
          team: "Team roster",
          policy: "Policy file",
          license: "License file",
          audit: "Team audit folder",
          sync: "Sync settings",
          documents: "Documents folder",
          assistant: "AI provider settings",
          redroid: "Redroid cloud phone settings",
          vendors: "Board farm rules",
          metrics: "Metrics file",
          apiKeys: "Enterprise API key list",
        },
        state: {
          missing: "Not set up",
          loaded: "In use",
          broken: "Unreadable: {{detail}}",
        },
      },
    },
  },
};
