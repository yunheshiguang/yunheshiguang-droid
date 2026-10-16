import type { WorkspaceModuleMessages } from "@/shared/workspace-module";

const commandsZh = {
  get_workspace_snapshot: "读取设备、任务、连接方式、诊断和 Android platform-tools 状态",
  run_batch_action: "在已连接的设备上运行批量操作",
  install_android_platform_tools: "安装 Android platform-tools 到应用目录",
  update_android_platform_tools_config: "设置 adb 和 fastboot 的来源",
  connect_device: "按 IP 地址和端口连接开启了网络调试的设备",
  pair_device: "用配对码与开启了无线调试的设备配对",
  disconnect_device: "断开通过网络连接的设备",
  get_governance_state: "读取操作者、授权策略、许可、已开通的功能和本次启动以来的审计记录",
  list_documents: "列出一个集合中的全部文档，例如流程",
  save_document: "新建或替换一份文档",
  remove_document: "删除一份文档",
  list_flow_templates: "列出可用的流程模板",
  discover_devices: "列出附近可连接的设备，例如开启了无线调试的设备",
  identify_devices: "识别每台设备的厂商和所需的厂商设置",
  get_workspace_health: "读取设备在线情况，以及各类操作的成败和耗时",
  list_input_channels: "列出用于输入文字和写入剪贴板的输入通道，以及所需的设备设置",
  update_device_profiles: "修改一批设备的编号、名称、备注、标签或分组",
  save_device_group: "新建分组，或修改分组的名称和颜色",
  remove_device_group: "删除分组，组内设备变为未分组",
  list_script_engines: "列出脚本引擎、支持的语言和所需的设备设置",
  list_flows: "列出流程库中的流程，以及无法运行的流程的原因",
  start_flow_run: "在指定设备、分组或标签上运行流程，可设置重复和重试",
  stop_flow_run: "在当前步骤结束后停止运行",
  list_flow_runs: "列出进行中和最近结束的运行",
  get_flow_run: "读取一次运行及其逐步进度",
  list_schedules: "列出定时任务及其下次和上次运行时间",
  save_schedule: "新建或修改定时任务",
  remove_schedule: "删除定时任务",
  get_api_gateway: "读取本机 API 的开关、端口、令牌和认证方式",
  update_api_gateway: "开启或关闭本机 API，或修改端口",
  create_api_token: "签发本机 API 令牌",
  revoke_api_token: "吊销本机 API 令牌",
  query_audit: "按操作、结果、操作者和时间分页查询审计记录",
  report_audit: "按操作者、分组、操作和日期汇总审计记录",
  export_audit: "导出审计记录为 CSV 或 JSON Lines 文件",
  get_assistant: "读取已安装的 AI 提供方和设置，密钥只返回是否已保存",
  update_assistant: "保存 AI 助手的提供方、地址、模型和密钥",
  draft_flow: "用 AI 按一句话生成流程草稿，并按批量操作校验步骤",
  draft_script: "用 AI 生成脚本草稿，或修改正在编辑的脚本",
  export_package: "导出选中的流程及其脚本为模板包，带版本号和摘要",
  inspect_package: "安装前预览模板包的内容、与库中重名的项和校验结果",
  install_package: "按冲突策略安装模板包到流程库和脚本库",
  list_market_packages: "列出已开通的模板市场中的模板包",
  list_cloud_phones: "列出各提供方的云手机及其状态",
  start_cloud_phone: "启动云手机",
  stop_cloud_phone: "停止云手机",
  register_cloud_phone: "按 adb 地址登记云手机",
  remove_cloud_phone: "移除按地址登记的云手机",
  get_scrcpy_state: "读取 scrcpy 的来源、安装状态和已打开的投屏窗口",
  update_scrcpy_config: "修改 scrcpy 的来源和投屏选项",
  install_scrcpy: "下载并安装官方发布的 scrcpy",
  launch_scrcpy_session: "为一台设备打开投屏窗口",
  stop_scrcpy_session: "关闭投屏窗口",
  trust_package_signer: "添加模板包签发者的公钥到本机信任名单",
  get_update_state: "读取本机安装的版本、所在的发布渠道和上次检查更新的结果",
  check_for_updates: "立即向发布页面检查是否有新版本",
  set_automatic_checks: "开启或关闭每天自动检查更新",
};

const commandsEn = {
  get_workspace_snapshot:
    "Read devices, tasks, connection modes, diagnostics, and Android platform-tools state",
  run_batch_action: "Run a batch operation on connected devices",
  install_android_platform_tools: "Install Android platform-tools into the app folder",
  update_android_platform_tools_config: "Set where adb and fastboot come from",
  connect_device: "Connect a device that has network debugging on, by IP address and port",
  pair_device: "Pair with a device that has wireless debugging on, using its pairing code",
  disconnect_device: "Disconnect a device connected over the network",
  get_governance_state:
    "Read the operator, policy, license, granted features, and this session's audit trail",
  list_documents: "List every document in a collection, such as flows",
  save_document: "Create or replace a document",
  remove_document: "Delete a document",
  list_flow_templates: "List the available flow templates",
  discover_devices:
    "List nearby devices ready to connect, such as those with wireless debugging on",
  identify_devices: "Identify each device's vendor and the vendor settings it needs",
  get_workspace_health: "Read device availability and each operation's results and durations",
  list_input_channels:
    "List the input channels that type text and write the clipboard, with the device setup they need",
  update_device_profiles: "Change the number, name, note, tags, or group of a batch of devices",
  save_device_group: "Create a group, or change a group's name or color",
  remove_device_group: "Delete a group; its devices become ungrouped",
  list_script_engines:
    "List the script engines, the languages they run, and the device setup they need",
  list_flows: "List the flows in the flow library, and why a flow cannot run if it cannot",
  start_flow_run: "Run a flow on chosen devices, a group, or a tag, with repeats and retries",
  stop_flow_run: "Stop a run after its current step",
  list_flow_runs: "List runs in progress and recent runs",
  get_flow_run: "Read one run and its step-by-step progress",
  list_schedules: "List schedules with their next and last run times",
  save_schedule: "Create or change a schedule",
  remove_schedule: "Delete a schedule",
  get_api_gateway: "Read the local API's on/off state, port, tokens, and ways to authenticate",
  update_api_gateway: "Turn the local API on or off, or change its port",
  create_api_token: "Issue a local API token",
  revoke_api_token: "Revoke a local API token",
  query_audit: "Search audit entries by action, outcome, operator, and time, one page at a time",
  report_audit: "Summarize audit entries by operator, group, action, and day",
  export_audit: "Export audit entries as a CSV or JSON Lines file",
  get_assistant:
    "Read the installed AI providers and settings; keys are reported only as saved or not",
  update_assistant: "Save the AI assistant's provider, endpoint, model, and key",
  draft_flow:
    "Draft a flow from one sentence with AI, with its steps checked against the batch operations",
  draft_script: "Draft a script with AI, or revise the one being edited",
  export_package:
    "Export chosen flows and the scripts they run as a template package with a version and digest",
  inspect_package:
    "Preview a package before installing: its contents, name clashes with the library, and verification",
  install_package:
    "Install a template package into the flow and script libraries by a conflict policy",
  list_market_packages: "List the template packages on the unlocked template markets",
  list_cloud_phones: "List each provider's cloud phones and their status",
  start_cloud_phone: "Start a cloud phone",
  stop_cloud_phone: "Stop a cloud phone",
  register_cloud_phone: "Register a cloud phone by its adb address",
  remove_cloud_phone: "Remove a cloud phone registered by address",
  get_scrcpy_state: "Read scrcpy's source, install status, and open mirroring windows",
  update_scrcpy_config: "Change scrcpy's source and mirroring options",
  install_scrcpy: "Download and install the official scrcpy release",
  launch_scrcpy_session: "Open a mirroring window for a device",
  stop_scrcpy_session: "Close a mirroring window",
  trust_package_signer:
    "Add a template package signer's public key to this computer's trusted signers",
  get_update_state:
    "Read the installed version, its release channel, and the result of the last update check",
  check_for_updates: "Ask the release page now whether a newer version is out",
  set_automatic_checks: "Turn the daily update check on or off",
};

const endpointsZh = {
  health: "检查本机 API 是否在运行，无需令牌",
  events: "通过 WebSocket 订阅工作区事件，浏览器可用 ?access_token= 传令牌",
  openapi: "读取本 API 的 OpenAPI 3.1 描述，用于生成客户端",
};

const endpointsEn = {
  health: "Check that the local API is running; no token needed",
  events:
    "Subscribe to workspace events over WebSocket; browsers can pass the token as ?access_token=",
  openapi: "Read the OpenAPI 3.1 description of this API, for generating clients",
};

export const externalApiMessages: WorkspaceModuleMessages = {
  "zh-CN": {
    externalApi: {
      title: "API",
      body: "本机 API 的操作同样受授权策略约束，并以 api:<调用方> 记入审计",
      commandsTitle: "应用窗口可调用的过程",
      http: {
        title: "本机 API",
        description:
          "在“设置”中开启，只监听 127.0.0.1；下列过程同时支持 Connect、gRPC 和 gRPC-Web，请求头需带 Authorization: Bearer <令牌>，按调用方限流",
      },
      stream: {
        title: "事件推送",
        description: "连接 /v1/events 后，每条消息是 { event, payloadJson }",
      },
      endpoints: endpointsZh,
      events: {
        devicesChanged: "设备连接、断开或状态变化",
        documentsChanged: "某个集合中的文档被保存或删除",
        deviceProfilesChanged: "设备资料或分组有变化",
        automationChanged: "流程运行有进展，或定时任务有变化",
        cloudPhonesChanged: "云手机启动、停止、登记或移除",
      },
      commands: commandsZh,
    },
  },
  "en-US": {
    externalApi: {
      title: "API",
      body: "Local API actions follow the same policy and are audited as api:<caller>",
      commandsTitle: "Procedures app windows can call",
      http: {
        title: "Local API",
        description:
          "Turn it on in Settings. It listens on 127.0.0.1 only. The procedures below answer Connect, gRPC, and gRPC-Web clients; send Authorization: Bearer <token>, and each caller is rate-limited separately",
      },
      stream: {
        title: "Event stream",
        description: "After connecting to /v1/events, each message is { event, payloadJson }",
      },
      endpoints: endpointsEn,
      events: {
        devicesChanged: "A device connected, disconnected, or changed state",
        documentsChanged: "A document in a collection was saved or deleted",
        deviceProfilesChanged: "Device profiles or groups changed",
        automationChanged: "A flow run progressed, or a schedule changed",
        cloudPhonesChanged: "A cloud phone was started, stopped, registered, or removed",
      },
      commands: commandsEn,
    },
  },
};
