export const auditWorkspaceZh = {
  unavailable: "审计历史不可用，只能查看应用启动以来的操作",
  storageFailed: "审计历史读写失败：{{message}}",
  invalid: {
    format: "请选择导出格式",
    path: "请选择可写入的位置",
  },
  exported: "已将 {{count}} 条审计记录导出到 {{path}}",
  audit: {
    exported: "导出 {{count}} 条审计记录（{{format}}）",
  },
} as const;

export const packagesWorkspaceZh = {
  invalid: {
    id: "模板包标识只能包含字母、数字、点、连字符和下划线，且须以字母或数字开头",
    nameRequired: "请输入模板包名称",
    version: "版本号须为语义化版本，例如 1.2.0",
    textTooLong: "内容最多 {{max}} 个字符",
    noFlows: "请至少选择一条流程",
    path: "请选择有效的位置",
    origin: "请选择要安装的模板包",
    itemId: "模板包中的 {{id}} 不是有效标识",
    duplicateId: "模板包中的标识 {{id}} 重复",
  },
  flowMissing: "流程库中已没有流程 {{id}}",
  scriptMissing: "流程“{{flow}}”运行的脚本 {{script}} 已不在脚本库中",
  marketMissing: "模板市场中已没有此模板包",
  readFailed: "无法读取模板包文件：{{message}}",
  tooLarge: "模板包文件不能超过 {{max}} MB",
  unreadable: "无法解析模板包文件：{{message}}",
  digestMismatch: "模板包摘要不匹配，文件可能已损坏或被篡改，无法安装",
  storageFailed: "模板包写入失败：{{message}}",
  problem: {
    unverified: "未由你信任的签发者签名，无法确认来源",
    scriptNotIncluded: "流程“{{flow}}”运行的脚本 {{script}} 既不在模板包中，也不在脚本库中",
    scriptEmpty: "脚本 {{id}} 为空",
  },
  exported: "已将 {{flows}} 条流程和 {{scripts}} 个脚本导出到 {{path}}",
  installed: "已安装 {{flows}} 条流程和 {{scripts}} 个脚本",
  signatureInvalid: "模板包签名无效，无法安装：{{message}}",
  signingFailed: "本机签名密钥不可用：{{message}}",
  signerInvalid: "不是有效的 minisign 公钥",
  signerTrusted: "{{name}} 签发的模板包将显示为可信",
  audit: {
    exported: "导出模板包“{{name}}” {{version}}",
    installed: "安装模板包“{{name}}” {{version}}",
    signerTrusted: "信任签发者“{{name}}”",
  },
} as const;

export const cloudWorkspaceZh = {
  registered: {
    label: "按地址登记",
    description: "按 adb 地址登记的云手机，团队共用一份清单；不支持启动和停止",
  },
  unsupported: "该提供方不支持此操作",
  providerUnknown: "未知的云手机提供方",
  phoneMissing: "未找到该云手机",
  invalid: {
    name: "请输入云手机名称，最多 {{max}} 个字符",
    endpoint: "请输入有效的主机和 1 到 65535 之间的端口",
  },
  alreadyRegistered: "该地址已登记",
  timedOut: "云手机服务响应超时",
  storageFailed: "云手机清单读写失败：{{message}}",
  audit: {
    started: "启动云手机“{{name}}”",
    stopped: "停止云手机“{{name}}”",
    registered: "登记云手机“{{name}}”",
    removed: "移除云手机“{{name}}”",
  },
} as const;
