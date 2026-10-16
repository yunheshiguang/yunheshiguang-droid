import { assistantWorkspaceZh } from "./workspace-assistant";
import { scrcpyWorkspaceZh } from "./workspace-scrcpy";
import { auditWorkspaceZh, cloudWorkspaceZh, packagesWorkspaceZh } from "./workspace-sharing";

export const workspaceZh = {
  workspace: {
    assistant: assistantWorkspaceZh,
    audit: auditWorkspaceZh,
    packages: packagesWorkspaceZh,
    cloud: cloudWorkspaceZh,
    scrcpy: scrcpyWorkspaceZh,
    device: {
      group: {
        default: "未分组",
      },
      note: {
        connected: "已连接",
        usbUnauthorized: "尚未允许 USB 调试",
        offline: "已离线，请重新连接",
        adbStatus: "ADB 状态：{{status}}",
      },
    },
    diagnostic: {
      title: {
        adbConflict: "ADB 不可用",
        deviceOffline: "设备离线",
        routerCongestion: "网络不稳定",
        usbUnauthorized: "等待设备授权",
        vendorPermission: "厂商权限限制",
        webview2Missing: "缺少 WebView2",
        androidSecurityLimit: "操作被阻止",
        androidPlatformToolsUnavailable: "Android platform-tools 不可用",
        androidPlatformToolsInvalidDirectory: "platform-tools 目录无效",
        androidPlatformToolsAdbMissing: "未找到 adb",
        androidPlatformToolsFastbootMissing: "未找到 fastboot",
        androidPlatformToolsDownloadFailed: "下载失败",
        androidPlatformToolsArchiveInvalid: "安装包无效",
        androidPlatformToolsInstallFailed: "安装失败",
        androidPlatformToolsStateRecovered: "设置已重置",
        actionNotAuthorized: "未获授权",
        none: "状态正常",
        unknown: "状态未知",
      },
      recovery: {
        adbConflict: "确认已安装 adb，并关闭占用 ADB 的手机助手",
        deviceOffline: "连接 USB 并允许调试，或在同一网络下重新连接无线调试",
        routerCongestion: "减少同一路由器下的设备数量，或改用 USB、OTG 连接",
        usbUnauthorized: "在设备的弹窗中允许 USB 调试，然后刷新",
        vendorPermission: "检查开发者选项中的 USB 安装和安全设置",
        webview2Missing: "安装 WebView2 后重启应用",
        androidSecurityLimit: "检查所需权限和输入后重试",
        androidPlatformToolsUnavailable: "安装 platform-tools，或选择有效的 platform-tools 目录",
        androidPlatformToolsInvalidDirectory: "选择解压后的 platform-tools 目录，然后重新应用",
        androidPlatformToolsAdbMissing: "确认所选目录中有 adb",
        androidPlatformToolsFastbootMissing: "确认所选目录中有 fastboot",
        androidPlatformToolsDownloadFailed: "检查网络后重新安装",
        androidPlatformToolsArchiveInvalid: "重新下载，或选择有效的 platform-tools 目录",
        androidPlatformToolsInstallFailed: "检查目录权限和磁盘空间后重新安装",
        androidPlatformToolsStateRecovered: "检查 platform-tools 设置，必要时重新应用",
        actionNotAuthorized: "请管理员授权此操作，或在“治理”中查看授权策略",
        none: "无需处理",
        unknown: "重新检测设备状态",
      },
      impact: {
        default: "操作未完成",
        runtime: "{{message}}",
      },
    },
    connection: {
      sectionTitle: "连接方式",
      usbFirst: {
        title: "首次 USB 连接",
        step1: "开启开发者选项",
        step2: "打开“USB 调试”",
        step3: "在设备上允许 USB 调试",
        step4: "刷新设备列表",
      },
      wifi: {
        title: "无线调试",
        step1: "在设备的开发者选项中打开“无线调试”（Android 11 及以上）",
        step2: "确保电脑和设备在同一网络",
        step3: "点按“使用配对码配对设备”，然后在附近的设备中输入配对码",
        step4: "配对后即可连接，断开后可重新连接",
        discoveryFailed: "无法查找附近的设备：{{message}}",
      },
      cloud: {
        title: "云手机",
        step1: "在“云手机”中按地址登记，或安装云手机服务的扩展",
        step2: "启动云手机后，它会出现在附近的设备中",
        step3: "点击“连接”，连接后在设备墙上标为云手机",
      },
      otg: {
        title: "OTG 与机箱",
        step1: "检查线缆和供电",
        step2: "确认已备份授权文件",
        step3: "确保主控机的 ADB 密钥可用",
        step4: "然后进入批量操作",
      },
      result: {
        connected: "已连接 {{device}}",
        alreadyConnected: "{{device}} 已连接",
        paired: "已与 {{endpoint}} 配对，请用“无线调试”中显示的 IP 地址和端口连接",
        disconnected: "已断开 {{device}}",
        invalidHost: "请输入 IP 地址或主机名",
        invalidPort: "请输入 1 到 65535 之间的端口",
        invalidPairingCode: "请输入设备上显示的 6 位配对码",
        invalidDevice: "只能断开网络设备",
        unreachable: "无法连接设备：{{reason}}",
        unauthorized: "设备拒绝连接，请在设备上允许调试后重试",
        pairingRejected: "配对失败，请核对配对码后重试",
        notConnected: "{{device}} 未连接",
        failed: "连接失败：{{reason}}",
      },
    },
    governance: {
      provider: {
        localOperator: {
          label: "本机操作者",
          description: "使用本应用的人可以执行所有操作",
        },
        coreEntitlements: {
          label: "内置功能",
          description: "未开通付费功能，需要付费功能的模块不会显示",
        },
        sessionTrail: {
          label: "最近操作",
          description: "保留应用启动以来的最近操作",
        },
        runLog: {
          label: "运行日志",
          description: "每项操作都写入运行日志，重启后仍可查看",
        },
        localUser: {
          label: "本机登录用户",
          description: "以这台电脑的登录用户作为操作者记入审计",
        },
        sqliteAudit: {
          label: "本机审计库",
          description: "审计记录保存在这台电脑上，重启后仍可查询和导出",
        },
      },
      audit: {
        unfinished: "操作结束时未报告结果",
        interrupted: "应用退出时操作尚未完成",
      },
    },
    documents: {
      storageFailed: "无法修改文档：{{message}}",
      audit: {
        saved: "在 {{collection}} 中保存 {{id}}",
        removed: "从 {{collection}} 中删除 {{id}}",
      },
    },
    telemetry: {
      health: {
        label: "健康汇总",
        description: "应用启动以来的在线率、失败率和耗时",
      },
    },
    vendors: {
      xiaomi: {
        label: "小米、Redmi、POCO",
        guidance: {
          securitySettings: "在开发者选项中打开“USB 调试（安全设置）”，需插入 SIM 卡并登录小米账号",
          installViaUsb: "打开“USB 安装”，才能通过 USB 安装应用",
        },
        explain: {
          securitySettings: "未打开“USB 调试（安全设置）”时，小米会拦截模拟点击和文字输入",
          installViaUsb: "小米拒绝安装，请在开发者选项中打开“USB 安装”",
        },
      },
      huawei: {
        label: "华为、荣耀",
        guidance: {
          chargeOnly:
            "在开发者选项中打开“仅充电模式下允许 ADB 调试”，否则 USB 切到充电模式时设备会断开",
          alwaysAllow: "设备询问是否允许 USB 调试时，勾选“始终允许这台计算机”",
        },
      },
      oppo: {
        label: "OPPO、一加、realme",
        guidance: {
          permissionMonitoring: "在开发者选项中打开“禁止权限监控”，才能通过 USB 控制输入和权限",
          installPrompt: "首次通过 USB 安装应用时，在设备上确认安装提示",
        },
        explain: {
          permissionMonitoring: "权限监控开启时，ColorOS 会拦截模拟输入",
          installPrompt: "设备未确认安装，请在设备上接受安装提示后重试",
        },
      },
      samsung: {
        label: "三星",
        guidance: {
          autoBlocker:
            "One UI 6 及以上需在“设置 → 安全和隐私”中关闭“自动拦截”，否则它会拦截 USB 命令",
        },
      },
    },
    input: {
      yadb: {
        label: "YADB（内置）",
        description: "无需安装应用或切换输入法；输入文字时经剪贴板粘贴，会覆盖设备剪贴板",
        pushFailed: "无法将 YADB 推送到设备：{{message}}",
        failed: "YADB 出错：{{message}}",
        unconfirmed: "YADB 无法写入剪贴板，文字未输入",
      },
      noChannel: "已安装的输入通道都不支持这台设备",
    },
    deviceShell: {
      failed: "设备命令执行失败：{{message}}",
      invalid: "参数中含有设备命令行不支持的字符",
      tooLong: "命令过长，超出设备命令行的上限",
    },
    organize: {
      invalid: {
        noDevices: "请至少选择一台设备",
        aliasTooLong: "名称最多 {{max}} 个字符",
        noteTooLong: "备注最多 {{max}} 个字符",
        tagTooLong: "每个标签最多 {{max}} 个字符",
        tooManyTags: "一台设备最多 {{max}} 个标签",
        numberTooLarge: "编号不能大于 {{max}}",
        groupMissing: "该分组已不存在",
        groupId: "分组标识无效",
        groupNameRequired: "请输入分组名称",
        groupNameTooLong: "分组名称最多 {{max}} 个字符",
      },
      storageFailed: "设备资料保存失败：{{message}}",
      audit: {
        profiles: "更新 {{count}} 台设备的资料",
        groupSaved: "保存分组“{{name}}”",
        groupRemoved: "删除分组“{{name}}”",
      },
    },
    scripts: {
      shell: {
        label: "设备 Shell（内置）",
        description: "用设备的 sh 运行脚本，返回输出和退出码，最长 5 分钟",
      },
      uploadFailed: "无法将脚本推送到设备：{{message}}",
      noExitCode: "脚本未运行完就中断，没有退出码",
      noEngine: "已安装的脚本引擎都无法运行此脚本",
      loadFailed: "无法读取脚本库：{{message}}",
      missing: "脚本库中未找到该脚本",
      unreadable: "无法解析此脚本",
      empty: "脚本为空",
    },
    automation: {
      loadFailed: "无法读取流程库：{{message}}",
      flowMissing: "流程库中未找到该流程",
      flowUnreadable: "无法解析此流程",
      flowEmpty: "流程没有步骤",
      targetMissing: "请选择运行目标",
      noDevices: "目标中没有在线设备",
      optionsTooLarge: "运行次数、重试次数或间隔超出上限",
      runNotRunning: "运行已结束",
      stepFailed: "第 {{step}} 步未在所有设备上完成，运行已停止",
      stopped: "已按请求停止",
      interrupted: "应用退出时运行尚未完成",
      audit: {
        run: "运行流程“{{name}}”",
        stopped: "停止流程“{{name}}”",
      },
    },
    schedules: {
      invalid: {
        id: "定时任务标识无效",
        nameRequired: "请输入定时任务名称",
        nameTooLong: "定时任务名称最多 {{max}} 个字符",
        flowRequired: "请选择要运行的流程",
        missing: "该定时任务已不存在",
        cron: "cron 表达式无效：{{message}}",
        limit: "最多只能有 {{max}} 个定时任务",
      },
      storageFailed: "定时任务保存失败：{{message}}",
      audit: {
        saved: "保存定时任务“{{name}}”",
        removed: "删除定时任务“{{name}}”",
      },
    },
    api: {
      tokens: {
        label: "本机令牌（内置）",
        description: "在设置中签发，仅保存摘要",
      },
      invalid: {
        tokenNameRequired: "请输入令牌用途",
        tokenNameTooLong: "令牌用途最多 {{max}} 个字符",
        tokenLimit: "最多只能有 {{max}} 个令牌",
        tokenNameTaken: "已有令牌使用这个用途",
        tokenMissing: "该令牌已不存在",
        port: "请输入 {{min}} 到 65535 之间的端口",
      },
      storageFailed: "本机 API 设置保存失败：{{message}}",
      hostFiles: "本机 API 不能使用这台电脑上的文件，请在应用中运行此操作",
      bindFailed: "无法打开端口：{{message}}",
      audit: {
        enabled: "开启本机 API，端口 {{port}}",
        disabled: "关闭本机 API",
        tokenCreated: "签发令牌“{{name}}”",
        tokenRevoked: "吊销令牌“{{name}}”",
      },
      errors: {
        unauthorized: "令牌无效或缺失",
        rateLimited: "请求过于频繁，请 {{seconds}} 秒后重试",
        notFound: "未找到记录",
        badRequest: "请求无效：{{message}}",
        unavailable: "暂时无法处理：{{message}}",
      },
    },
    updates: {
      unreachable: "无法访问发布页面：{{message}}",
      unreadable: "发布页面返回的信息无法被当前版本识别",
      storageFailed: "无法保存更新设置：{{message}}",
    },
    templates: {
      starter: {
        label: "入门模板",
        description: "内置的常用流程",
      },
      installAndLaunch: {
        name: "安装并启动应用",
        description: "在所选设备上安装 APK 并启动",
      },
      captureScreens: {
        name: "为每台设备截图",
        description: "将所选设备的截图保存到同一文件夹",
      },
      recoverAndCapture: {
        name: "恢复连接后截图",
        description: "重新连接已断开的设备，然后截图",
      },
    },
    task: {
      title: {
        screenshot: "截图",
        screenRecord: "录屏",
        installApk: "安装 APK",
        uploadFile: "上传文件",
        inputText: "输入文字",
        launchApp: "启动应用",
        clipboardWrite: "写入剪贴板",
        recoverConnection: "恢复连接",
        runScript: "运行脚本",
        stopApp: "停止应用",
        clearAppData: "清除应用数据",
        uninstallApp: "卸载应用",
        default: "批量操作",
      },
      target: {
        count: "{{count}} 台设备",
      },
      detail: {
        completed: "已在 {{count}} 台设备上完成",
        saved: "已将 {{count}} 个文件保存到 {{path}}",
      },
    },
    validation: {
      needRealDevice: "请至少选择一台设备",
      needText: "请输入要发送的文字",
      needPackage: "请输入应用包名",
      needPath: "请选择有效的文件",
      needSavePath: "请选择保存位置",
      needScript: "请选择要运行的脚本",
      deviceNotConnected: "设备未连接",
    },
    adb: {
      unauthorized: "{{serial}} 尚未允许 USB 调试",
      offline: "ADB 报告设备离线",
    },
    androidPlatformTools: {
      unavailable: "adb 和 fastboot 均不可用",
      invalidDirectory: "无法读取 {{path}}",
      adbMissing: "未找到 {{binary}}",
      fastbootMissing: "未找到 {{binary}}",
    },
  },
} as const;
