# yunheshiguang

**English** · [العربية](./README.ar.md) · [中文](./README.zh-CN.md) · [Français](./README.fr.md) · [Русский](./README.ru.md) · [Español](./README.es.md)

A desktop workbench for running fleets of Android devices from one computer: connect them, watch them, and operate them in batches. Built on Tauri 2 (Rust backend, React frontend). Devices connect over ADB, with nothing to install on them.

## Features

- **Device wall**: USB, Wi-Fi (pairing and mDNS discovery), and cloud phones connect in one place. The ADB server pushes device changes, so nothing polls. Devices can be numbered, named, tagged, and grouped. Serial numbers and other sensitive data are masked by default, and masked values are not searchable.
- **Batch operations**: screenshot, screen record, install APK, upload file, input text, write clipboard, launch or stop an app, uninstall, clear data, recover connection, and run script. Actions that cannot be undone, such as uninstalling or clearing data, ask for confirmation first.
- **Mirroring**: opens a window per device with the official [scrcpy](https://github.com/Genymobile/scrcpy), downloaded in the app with a pinned version and SHA-256 digest.
- **Automation**: a flow editor, starter templates, cron schedules, and run history. Flows run in the backend; the interface only starts and stops them and shows their progress.
- **Scripts**: a shell script console that returns each device's output and exit code.
- **AI drafts**: describe a flow or a script in one sentence and the AI drafts it, shown as it is written. Drafts only open in the editor; saving and running are always up to you. Works with any OpenAI-compatible API (OpenAI, DeepSeek, Qwen, a local Ollama, and others); keys stay in the backend on this computer.
- **Governance and reports**: every action records its operator. The audit trail is kept in a local SQLite database and can be filtered and exported as CSV or JSON Lines. Reports total operations by group, operator, action, and day.
- **Template packages**: share flows and scripts as packages with a content digest, optionally signed with [minisign](https://jedisct1.github.io/minisign/). The install preview names the signer.
- **Local API**: HTTP and WebSocket for scripts and other systems. Off by default.
- **Interface**: Chinese and English, light and dark themes.

## Supported systems

| System | System or custom ADB | Platform Tools download | scrcpy download |
| --- | --- | --- | --- |
| Windows x86-64 | Yes | Yes | Yes |
| Windows ARM64 | Yes | No | Yes |
| macOS (Intel, Apple silicon) | Yes | Yes | Yes |
| Linux x86-64 (glibc) | Yes | Yes | Yes |
| Other Linux | With a compatible ADB | No | No |

The app downloads the official Android Platform Tools 37.0.0 after you accept the Android SDK license. URLs, sizes, and SHA-256 digests are pinned in the source; a package that does not match is rejected before extraction.

## Getting started

Requires Node.js 24, pnpm 10, Rust 1.98.1 (selected by `rust-toolchain.toml`), and the [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install
pnpm tauri dev      # run the desktop app
pnpm tauri build    # build installers
```

To work on the interface alone, run `pnpm dev` and open <http://localhost:1420> in a browser. Without a Tauri host, in-memory implementations of the same Connect services answer the interface; production builds leave them out.

## Downloads and updates

Installers are on the [releases page](https://github.com/yunheshiguang/yunheshiguang-android/releases):

- **Stable**: the latest release, published by hand from a version tag.
- **Nightly**: the `nightly` prerelease, rebuilt after every change on `main` passes CI.

Every release has the same file names (`yunheshiguang-windows-x86_64-setup.exe`, `yunheshiguang-macos-arm64.dmg`, `yunheshiguang-macos-x86_64.dmg`, `yunheshiguang-linux-x86_64.AppImage`, `yunheshiguang-linux-x86_64.deb`, `yunheshiguang-linux-x86_64.rpm`) along with `SHA256SUMS` and `release.json`. The app checks its own channel once a day (**Settings → Updates**) and links to the matching file when a newer build is out; it never downloads or installs anything by itself. The builds are not signed with a developer certificate yet. [docs/releases.md](./docs/releases.md) describes the channels, the manifest, and how to publish a release.

## Local API

Turn it on under **Settings → Local API**. It listens on `127.0.0.1` only, on port 47831 by default. Tokens are issued on the same page; each token is shown once, and the app stores only its digest. The API serves the same Connect services as the app's windows, to Connect, gRPC, and gRPC-Web clients alike. Every request except `/v1/health` needs a token:

```sh
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:47831/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot
```

`/v1/openapi.json` describes every procedure the API serves (OpenAPI 3.1, generated from the services in `proto/`); procedures that only read data also answer `GET`. A procedure is served over the API when its definition in `proto/` carries the `local_api` option. `/v1/events` is a WebSocket that pushes workspace events; browsers, which cannot set headers on it, can pass `?access_token=` instead. Actions through the API go through the same authorization and audit as the app's windows, and each caller is rate-limited separately.

## Extensions

The default build is the open-source core and contains no extension code. An extension has two halves: `native`, a Tauri plugin crate that depends only on `crates/sdk` and `crates/contract`, and `web`, a frontend package that uses the host only through `yunheshiguang/sdk`. Extensions can provide authorization, identity, licensing, audit storage and extra audit destinations, document storage, template sources, connection methods, telemetry, vendor adapters, input channels, script engines, API authentication, AI providers, and cloud phone providers; anything no extension provides falls back to the core's default.

`extensions/reference` is the reference extension and implements most of these (a Cedar authorization policy, team identity, a license, a team-shared audit trail, synced-folder storage, a template market, Redroid cloud phones, an AI provider for native APIs such as Claude and Gemini, Prometheus metrics, a board-farm vendor adapter, ADBKeyBoard input, the AutoJs6 script engine, enterprise API keys, and an enterprise page):

```sh
pnpm tauri dev --config src-tauri/tauri.reference.conf.json   # run with the reference extension
pnpm dev --mode reference                                      # interface only
```

## Development

| Command | What it does |
| --- | --- |
| `pnpm test` | Biome, the file-length limit, FSD layering (Steiger), import boundaries, Knip, the frontend–backend contract, and Vitest |
| `pnpm build` | Type-checks and builds the frontend |
| `pnpm check:distributions` | Builds the open-source and every composed distribution, and confirms the open-source build carries no extension code |
| `pnpm proto:lint` / `pnpm proto:generate` / `pnpm proto:openapi` / `pnpm proto:verify` | Lints the contracts, regenerates the TypeScript bindings, regenerates the OpenAPI document, and checks that the committed document is current |
| `cargo test --workspace --all-features` | Rust tests, including the build composed with extensions |
| `pnpm lint:rust` / `pnpm lint:deps` | clippy (`-D warnings`) and cargo-deny |

The contracts in `proto/` are the single source of truth: `pnpm install` generates the TypeScript bindings into `src/shared/proto/gen/` (not committed), and Rust bindings are generated at build time.

## Third-party components

- [YADB](https://github.com/ysbing/YADB) v1.1.2 (LGPL-3.0): types text and writes the device clipboard; embedded unmodified in `src-tauri/assets/yadb`.
- [Android SDK Platform Tools](https://developer.android.com/tools/releases/platform-tools): downloaded on demand after you accept its license; not distributed with the app.
- [scrcpy](https://github.com/Genymobile/scrcpy) (Apache-2.0): official releases downloaded on demand; not distributed with the app.

## License

[GPL-3.0-only](./LICENSE)
