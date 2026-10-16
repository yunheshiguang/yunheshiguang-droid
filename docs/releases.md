# Releases and updates

yunheshiguang ships on two channels. Both are GitHub releases of this repository with the same file names, so a link to a file keeps working from one build to the next.

| | Nightly | Stable |
| --- | --- | --- |
| Published by | `.github/workflows/nightly.yml`, after CI passes on a push to `main` | `.github/workflows/release.yml`, started by hand with a version tag |
| Release | One prerelease with the tag `nightly`, replaced in place | One release per tag, created as a draft for review |
| The app reads | `releases/download/nightly/release.json` | `releases/latest/download/release.json` |
| A build is newer when | It was made from a different commit | Its version is higher by SemVer |

## Files

| File | System | Package |
| --- | --- | --- |
| `yunheshiguang-windows-x86_64-setup.exe` | Windows x86-64 | Installer |
| `yunheshiguang-macos-arm64.dmg` | macOS on Apple silicon | Disk image |
| `yunheshiguang-macos-x86_64.dmg` | macOS on Intel | Disk image |
| `yunheshiguang-linux-x86_64.AppImage` | Linux x86-64 | AppImage |
| `yunheshiguang-linux-x86_64.deb` | Linux x86-64 | Debian package |
| `yunheshiguang-linux-x86_64.rpm` | Linux x86-64 | RPM package |
| `SHA256SUMS` | | SHA-256 of every file above |
| `release.json` | | The manifest described below |

The list lives in `scripts/release/assets.json`. Each entry names the file, its system, architecture, and package kind, and the Tauri bundle it is copied from. CI compares the list with the one on the base commit and fails when a published file disappears or changes its name, unless the old name moves to `retired`; links that people and older builds already use stay valid that way.

## `release.json`

```json
{
  "schema_version": 1,
  "channel": "nightly",
  "tag": "nightly",
  "version": "0.1.0",
  "commit": "0123456789abcdef0123456789abcdef01234567",
  "date": "2026-10-14",
  "assets": [
    {
      "name": "yunheshiguang-macos-arm64.dmg",
      "os": "macos",
      "arch": "arm64",
      "kind": "disk-image",
      "size": 18874368,
      "sha256": "…",
      "url": "https://github.com/yunheshiguang/yunheshiguang-android/releases/download/nightly/yunheshiguang-macos-arm64.dmg"
    }
  ]
}
```

- `schema_version` is `1`. A reader that meets another value treats the manifest as unreadable rather than guessing.
- `channel` is `nightly` or `release`; `tag` is the release tag; `version` is the app version; `commit` is the full commit the build was made from.
- Every asset carries its size, SHA-256, and download URL, which always points at this release of this repository.

`scripts/release/fixtures/manifest.json` is a complete example. The release script tests and the update check in the app both read it, so the two cannot drift apart.

## The update check in the app

- The release workflows embed the channel, the commit, and the repository into the build (`YUNHESHIGUANG_BUILD_CHANNEL`, `YUNHESHIGUANG_BUILD_COMMIT`, `YUNHESHIGUANG_BUILD_REPOSITORY`). A build made any other way is a local build and does not check.
- The first check runs 30 seconds after the app starts and then once a day. **Settings → Updates** turns the daily check off and has a **Check now** button.
- The request goes over HTTPS through the system proxy, gives up after 30 seconds, and reads at most 256 KB. A channel with nothing published yet (HTTP 404) counts as up to date.
- The manifest is checked before use: an unknown schema version, channel, or a malformed commit makes it unreadable, and an asset is dropped unless its name, hash, size, and URL all hold up.
- The app offers the file that matches how it was installed: the installer on Windows, the disk image for the Mac's architecture, the AppImage when it runs as one, and the deb or rpm package when it was installed under `/usr`. When nothing matches, it links to the release page.
- A newer build shows in Settings and as a badge in the title bar. The app only opens links in the browser; it never downloads or replaces itself.

## Publishing a stable release

1. Set the same version in `Cargo.toml` (`[workspace.package]`), `package.json`, and `src-tauri/tauri.conf.json`. `node scripts/release/release.mjs version` prints it and fails when the three disagree.
2. Merge the change to `main`. Tagging is optional: when the tag exists, it has to point at the commit the workflow runs on.
3. Run **Actions → Release → Run workflow** with the tag, such as `v0.2.0`. The version has to match the tag without its `v`. A tag with a suffix, such as `v0.2.0-rc.1`, becomes a prerelease, which the stable channel does not offer.
4. The workflow runs CI, builds the installers on Windows, macOS, and Linux, and drafts the release with the files above and notes that list the changes since the previous stable tag. Running it again for the same tag replaces the draft's files.
5. Review the draft and publish it. Publishing creates the tag when it does not exist yet, and the new release becomes the one the stable channel reads.

## How the nightly release is replaced

Runs on `main` queue rather than cancel each other, so a publish is never cut off halfway. After CI and the installer builds pass, the workflow:

1. checks the downloaded installers against the file list;
2. writes `SHA256SUMS`, `release.json`, and notes listing the changes since the previous nightly build;
3. creates the `nightly` prerelease if it does not exist;
4. uploads the installers over the old ones, then `SHA256SUMS` and `release.json` last, so the manifest never names a file that is not there yet;
5. deletes files this build no longer ships;
6. moves the `nightly` tag to the new commit and updates the title and notes;
7. confirms that every file finished uploading with the size that was built.

Network calls to GitHub are retried with growing waits (`scripts/retry.sh`).

## Verifying a download

```sh
sha256sum --ignore-missing -c SHA256SUMS       # Linux
shasum -a 256 --ignore-missing -c SHA256SUMS   # macOS
```

On Windows, compare `(Get-FileHash <file>).Hash` with the line in `SHA256SUMS`.

The builds are not signed with a developer certificate yet. On macOS the app carries an ad-hoc signature; open it once, then choose **Open Anyway** in **System Settings → Privacy & Security**. On Windows, SmartScreen asks for confirmation under **More info → Run anyway**.

## Release script

`scripts/release/release.mjs` uses only Node.js built-ins and backs both workflows:

| Command | What it does |
| --- | --- |
| `collect <bundle dir> <out dir> --os <os> --arch <arch>` | Copies this machine's Tauri bundles to their release names; fails unless each matches exactly one file |
| `verify <dir>` | Fails when a listed file is missing or an unlisted one is present |
| `checksums <dir>` | Writes `SHA256SUMS` |
| `manifest <dir> --repo --tag --channel --commit --date [--version]` | Writes `release.json` |
| `notes --repo --tag --channel --commit --date [--previous <commit>] [--since <label>]` | Writes the release notes |
| `uploaded <dir> <gh release view --json assets>` | Fails unless the release holds exactly these files, fully uploaded, at these sizes |
| `compat --baseline <commit or tag>` | Fails when a published file name is removed without being retired |
| `version [--expect <version>]` | Prints the app version; fails when the version files disagree or differ from the expected one |
