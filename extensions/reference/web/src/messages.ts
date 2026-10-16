import type { WorkspaceModuleMessages } from "yunheshiguang/sdk";

export const referenceMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    reference: {
      telemetry: {
        prometheus: {
          label: "Prometheus 指标文件",
          description: "以 OpenMetrics 格式写出指标，供 Prometheus 采集",
        },
      },
      vendor: {
        boardFarm: {
          label: "主板机",
          guidance: {
            power: "板卡离线时先检查供电，共用电源电压不足会导致反复断开",
            adbKey: "用同一台电脑的 adb 密钥授权整排板卡，更换电脑需重新授权",
          },
          explain: {
            storage: "主板机的数据分区较小，安装前先删除旧的安装包",
          },
        },
      },
      input: {
        adbKeyboard: {
          label: "ADBKeyBoard（参考扩展）",
          description: "设备上的输入法，只负责打字；不是当前输入法时改用内置的 YADB",
          setup: {
            install: "在设备上安装 ADBKeyBoard（Android 16 使用 v2.5-dev）",
            select:
              "将其设为当前输入法：adb shell ime enable {{ime}}，再 adb shell ime set {{ime}}",
          },
          textOnly: "ADBKeyBoard 只支持打字",
          failed: "ADBKeyBoard 输入失败：{{message}}",
        },
      },
      scripts: {
        autojs6: {
          label: "AutoJs6（参考扩展）",
          description: "设备上的 JavaScript 自动化应用，可点击控件、找图、操作无障碍节点",
          setup: {
            install: "在设备上安装 AutoJs6 并开启其无障碍服务",
            storage: "允许 AutoJs6 读取存储，脚本保存在下载目录",
          },
          failed: "AutoJs6 无法启动脚本：{{message}}",
        },
      },
      api: {
        keys: {
          label: "企业 API 密钥（参考扩展）",
          description: "由企业密钥系统签发，摘要和角色记录在 api-keys.json",
        },
      },
      policy: {
        label: "团队策略（Cedar）",
        description: "按 policy.cedar 的规则，依据操作者的角色和目标设备的分组决定是否允许",
        denied: "团队策略中没有允许 {{action}} 的规则",
        forbidden: "团队策略规则 {{rules}} 禁止 {{action}}",
        broken: "无法读取策略文件，拒绝所有操作",
      },
      team: {
        label: "团队名单",
        description: "按 team.json 识别本机登录的成员及其角色，未列出的用户视为无角色的访客",
      },
      license: {
        label: "许可文件",
        description: "开通已安装许可包含的功能",
        plan: "{{plan}} 版",
      },
      audit: {
        label: "团队审计",
        description: "每台电脑的审计记录写入同步盘中自己的 JSON Lines 文件，查询覆盖整个团队",
        readFailed: "无法读取团队审计：{{message}}",
      },
      assistant: {
        label: "厂商原生接口（参考扩展）",
        description:
          "直连 Anthropic、Gemini 等服务的原生接口，按 assistant.json 中的模型名称选择服务",
        broken: "无法读取 assistant.json：{{detail}}",
        modelMissing: "assistant.json 未填写模型名称",
        keyMissing: "环境变量 {{variable}} 中没有密钥",
        failed: "AI 服务请求失败：{{message}}",
        empty: "模型未给出回答",
      },
      redroid: {
        label: "Redroid 云手机（参考扩展）",
        description:
          "按 redroid.json 通过 Docker Engine API 管理带标签的 Redroid 容器，可启动和停止",
        broken: "无法读取 redroid.json：{{detail}}",
        failed: "Docker 操作失败：{{message}}",
        gone: "操作完成后未找到这台云手机",
      },
      market: {
        label: "模板市场",
        description: "参考扩展提供的流程模板",
        nightlyHealthCheck: {
          name: "夜间巡检",
          description: "重新连接夜间离线的设备，再为它们截图",
        },
        releaseSmokeTest: {
          name: "发版冒烟测试",
          description: "安装发布版并启动，再为每台设备截图",
        },
      },
    },
  },
  "en-US": {
    reference: {
      telemetry: {
        prometheus: {
          label: "Prometheus metrics file",
          description: "Writes metrics in OpenMetrics format for Prometheus to collect",
        },
      },
      vendor: {
        boardFarm: {
          label: "Board farm",
          guidance: {
            power:
              "If boards go offline, check the power first. A shared supply that sags can disconnect them repeatedly",
            adbKey:
              "Authorize a whole rack with one computer's adb key. Switching computers requires authorizing again",
          },
          explain: {
            storage:
              "Board farm devices have a small data partition. Remove old packages before installing",
          },
        },
      },
      input: {
        adbKeyboard: {
          label: "ADBKeyBoard (reference extension)",
          description:
            "An input method on the device that handles typing only. When it is not the current keyboard, the built-in YADB types instead",
          setup: {
            install: "Install ADBKeyBoard on the device (use v2.5-dev on Android 16)",
            select:
              "Set it as the current keyboard: adb shell ime enable {{ime}}, then adb shell ime set {{ime}}",
          },
          textOnly: "ADBKeyBoard only supports typing",
          failed: "ADBKeyBoard could not type the text: {{message}}",
        },
      },
      scripts: {
        autojs6: {
          label: "AutoJs6 (reference extension)",
          description:
            "A JavaScript automation app on the device that can tap controls, find images, and drive accessibility nodes",
          setup: {
            install: "Install AutoJs6 on the device and turn on its accessibility service",
            storage: "Allow AutoJs6 to read storage. Scripts are saved in the Download folder",
          },
          failed: "AutoJs6 could not start the script: {{message}}",
        },
      },
      api: {
        keys: {
          label: "Enterprise API keys (reference extension)",
          description:
            "Issued by the enterprise key system; digests and roles are listed in api-keys.json",
        },
      },
      policy: {
        label: "Team policy (Cedar)",
        description:
          "Allows or refuses actions by the rules in policy.cedar, based on the operator's roles and the target devices' groups",
        denied: "No rule in the team policy allows {{action}}",
        forbidden: "Team policy rule {{rules}} forbids {{action}}",
        broken: "Could not read the policy file; all actions are refused",
      },
      team: {
        label: "Team roster",
        description:
          "Identifies the member signed in to this computer and their roles from team.json; anyone not listed is a guest without roles",
      },
      license: {
        label: "License file",
        description: "Grants the features included in the installed license",
        plan: "{{plan}} plan",
      },
      audit: {
        label: "Team audit",
        description:
          "Each computer writes its audit entries to its own JSON Lines file in the synced folder; queries cover the whole team",
        readFailed: "Could not read the team audit: {{message}}",
      },
      assistant: {
        label: "Vendor-native APIs (reference extension)",
        description:
          "Calls the native APIs of Anthropic, Gemini, and other services directly, picking the service by the model name in assistant.json",
        broken: "Could not read assistant.json: {{detail}}",
        modelMissing: "assistant.json has no model name",
        keyMissing: "No key in the environment variable {{variable}}",
        failed: "The AI service request failed: {{message}}",
        empty: "The model gave no answer",
      },
      redroid: {
        label: "Redroid cloud phones (reference extension)",
        description:
          "Manages labeled Redroid containers through the Docker Engine API as set in redroid.json, and can start and stop them",
        broken: "Could not read redroid.json: {{detail}}",
        failed: "The Docker operation failed: {{message}}",
        gone: "The cloud phone was not found after the operation",
      },
      market: {
        label: "Template market",
        description: "Flow templates from the reference extension",
        nightlyHealthCheck: {
          name: "Nightly health check",
          description: "Reconnect devices that went offline overnight, then screenshot them",
        },
        releaseSmokeTest: {
          name: "Release smoke test",
          description: "Install the release build, launch it, then screenshot every device",
        },
      },
    },
  },
};
