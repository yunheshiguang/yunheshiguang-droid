# 云禾拾光（yunheshiguang）

[English](./README.md) · [العربية](./README.ar.md) · **中文** · [Français](./README.fr.md) · [Русский](./README.ru.md) · [Español](./README.es.md)

多设备运营工作台：在一台电脑上连接、查看和批量操作成批的 Android 设备。桌面应用基于 Tauri 2（Rust 后端 + React 前端），设备通过 ADB 连接，不需要在设备上安装客户端。

## 功能

- **设备墙**：USB、Wi-Fi（配对、mDNS 发现）和云手机在同一处连接，设备的插拔和状态变化由 ADB server 推送，不靠轮询。设备可以编号、命名、打标签、分组；序列号等敏感信息默认遮盖，遮盖后也无法被搜索。
- **批量操作**：截图、录屏、安装 APK、上传文件、输入文字、写入剪贴板、启动或停止应用、卸载、清除数据、恢复连接、运行脚本。卸载、清除数据等不可撤销的操作会先确认。
- **投屏**：用官方 [scrcpy](https://github.com/Genymobile/scrcpy) 为每台设备打开窗口，在应用内一键下载，版本和 SHA-256 摘要固定。
- **自动化**：流程编辑器、入门模板、按 cron 定时运行和运行记录。流程在后端执行，界面只负责启动、停止和查看进度。
- **脚本**：shell 脚本控制台，返回每台设备的输出和退出码。
- **AI 草稿**：用一句话让 AI 生成流程或脚本草稿，边生成边显示。草稿只会打开在编辑器里，保存和运行都由你决定。支持任何 OpenAI 兼容接口（OpenAI、DeepSeek、通义千问、本机 Ollama 等），密钥只保存在本机后端。
- **治理与报表**：每项操作都记录操作者。审计记录保存在本机 SQLite 中，可以筛选，也可以导出为 CSV 或 JSON Lines；报表按分组、操作者、操作和日期汇总。
- **模板包**：流程和脚本可以打包分享，包带内容摘要，可用 [minisign](https://jedisct1.github.io/minisign/) 签名，安装前的预览会显示签发者。
- **本机 API**：HTTP 和 WebSocket，供脚本和其他系统调用，默认关闭。
- **界面**：中文和英文，浅色和深色主题。

## 支持的系统

| 系统 | 系统自带或手动指定的 ADB | 应用内下载 Platform Tools | 应用内下载 scrcpy |
| --- | --- | --- | --- |
| Windows x86-64 | 支持 | 支持 | 支持 |
| Windows ARM64 | 支持 | 不提供 | 支持 |
| macOS（Intel、Apple silicon） | 支持 | 支持 | 支持 |
| Linux x86-64（glibc） | 支持 | 支持 | 支持 |
| 其他 Linux | 配置兼容的 ADB 即可 | 不提供 | 不提供 |

应用在你同意 Android SDK 许可后下载官方 Android Platform Tools 37.0.0。下载地址、大小和 SHA-256 摘要固定在源码中，不匹配的安装包在解压前就会被拒绝。

## 开始

需要 Node.js 24、pnpm 10、Rust 1.98.1（由 `rust-toolchain.toml` 自动选择），以及 [Tauri 的系统依赖](https://v2.tauri.app/start/prerequisites/)。

```sh
pnpm install
pnpm tauri dev      # 运行桌面应用
pnpm tauri build    # 构建安装包
```

只开发界面时，运行 `pnpm dev`，在浏览器中打开 <http://localhost:1420>。没有 Tauri 宿主时，由同一组 Connect 服务的内存实现响应界面；生产构建不包含它们。

## 下载与更新

安装包在[发布页面](https://github.com/yunheshiguang/yunheshiguang-android/releases)：

- **正式版**：最新的正式发布，由维护者根据版本标签手动发布。
- **每日构建**：标签为 `nightly` 的预发布版本，`main` 上的每次改动通过 CI 后重新构建。

每个版本的文件名都相同（`yunheshiguang-windows-x86_64-setup.exe`、`yunheshiguang-macos-arm64.dmg`、`yunheshiguang-macos-x86_64.dmg`、`yunheshiguang-linux-x86_64.AppImage`、`yunheshiguang-linux-x86_64.deb`、`yunheshiguang-linux-x86_64.rpm`），并附有 `SHA256SUMS` 和 `release.json`。应用每天检查一次所在渠道（**设置 → 更新**），有新版本时给出对应安装包的链接；它不会自行下载或安装任何东西。目前的安装包还没有使用开发者证书签名。渠道、清单格式和发布步骤见 [docs/releases.md](./docs/releases.md)。

## 本机 API

在“设置 → 本机 API”中开启，只监听 `127.0.0.1`，默认端口 47831。令牌也在同一页签发，每个令牌只显示一次，应用只保存其摘要。API 与应用窗口使用同一组 Connect 服务，Connect、gRPC 和 gRPC-Web 客户端都能直接调用。除 `/v1/health` 外，每个请求都需要令牌：

```sh
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:47831/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot
```

`/v1/openapi.json` 列出 API 提供的全部过程（OpenAPI 3.1，由 `proto/` 中的服务生成）；只读取数据的过程也可以用 `GET` 调用。`proto/` 中带 `local_api` 选项的过程才经 API 提供。`/v1/events` 是推送工作区事件的 WebSocket，浏览器无法为它设置请求头，可以改用 `?access_token=`。经 API 执行的操作与窗口中的操作一样受授权策略约束、记入审计，每个调用方单独限流。

## 扩展

默认构建是开源核心，不含任何扩展代码。扩展分两部分：`native` 是 Tauri 插件 crate，只依赖 `crates/sdk` 和 `crates/contract`；`web` 是前端包，只通过 `yunheshiguang/sdk` 使用宿主。授权、身份、许可、审计（存储和额外的去向）、文档存储、模板来源、连接方式、遥测、厂商适配、输入通道、脚本引擎、API 认证、AI 提供方和云手机提供方都可以由扩展提供，没有扩展提供的部分使用核心的默认实现。

`extensions/reference` 是参考扩展，实现了其中的大部分（Cedar 授权策略、团队身份、许可、团队共享的审计、同步盘存储、模板市场、Redroid 云手机、对接 Claude 和 Gemini 等原生接口的 AI 提供方、Prometheus 指标、主板机适配、ADBKeyBoard 输入、AutoJs6 脚本引擎、企业 API 密钥和企业页）：

```sh
pnpm tauri dev --config src-tauri/tauri.reference.conf.json   # 带参考扩展运行
pnpm dev --mode reference                                      # 只开发界面
```

## 开发

| 命令 | 作用 |
| --- | --- |
| `pnpm test` | Biome、文件行数上限、FSD 分层（Steiger）、import 边界、Knip、前后端契约和 Vitest |
| `pnpm build` | 类型检查并构建前端 |
| `pnpm check:distributions` | 构建开源版和每个组合发行版，确认开源版不含扩展代码 |
| `pnpm proto:lint` / `pnpm proto:generate` / `pnpm proto:openapi` / `pnpm proto:verify` | 检查契约、重新生成 TypeScript 绑定、重新生成 OpenAPI 文档、核对提交的文档是否最新 |
| `cargo test --workspace --all-features` | Rust 测试，包括带扩展的组合构建 |
| `pnpm lint:rust` / `pnpm lint:deps` | clippy（`-D warnings`）和 cargo-deny |

`proto/` 中的契约是唯一来源：`pnpm install` 把 TypeScript 绑定生成到 `src/shared/proto/gen/`（不提交），Rust 绑定在构建时生成。

## 第三方组件

- [YADB](https://github.com/ysbing/YADB) v1.1.2（LGPL-3.0）：输入文字和写入设备剪贴板，未经修改内嵌在 `src-tauri/assets/yadb`。
- [Android SDK Platform Tools](https://developer.android.com/tools/releases/platform-tools)：用户同意许可后按需下载，不随应用分发。
- [scrcpy](https://github.com/Genymobile/scrcpy)（Apache-2.0）：按需下载官方发布包，不随应用分发。

## 许可证

[GPL-3.0-only](./LICENSE)
