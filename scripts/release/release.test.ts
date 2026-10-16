import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  checksums,
  collect,
  compatibility,
  loadPolicy,
  manifest,
  notes,
  readVersion,
  root,
  uploaded,
  verify,
} from "./release.mjs";

type Asset = { name: string; os: string; arch: string; kind: string; bundle: string };

const fixture = JSON.parse(
  readFileSync(path.join(import.meta.dirname, "fixtures", "manifest.json"), "utf8"),
);
const { assets } = loadPolicy() as { assets: Asset[]; retired: string[] };

function temporary() {
  return mkdtempSync(path.join(tmpdir(), "release-"));
}

function built(directory: string) {
  for (const [index, asset] of assets.entries()) {
    writeFileSync(path.join(directory, asset.name), "x".repeat(index + 1));
  }
  return directory;
}

const info = {
  repo: "yunheshiguang/yunheshiguang-android",
  tag: "nightly",
  channel: "nightly",
  commit: "0123456789abcdef0123456789abcdef01234567",
  date: "2026-10-14",
  version: "0.1.0",
};

describe("release assets", () => {
  it("publishes exactly the assets the app's manifest fixture describes", () => {
    expect(assets.map(({ name, os, arch, kind }) => ({ name, os, arch, kind }))).toEqual(
      fixture.assets.map(({ name, os, arch, kind }: Asset) => ({ name, os, arch, kind })),
    );
  });

  it("gives each bundle its fixed name and refuses an ambiguous bundle folder", () => {
    const bundles = temporary();
    const out = temporary();
    mkdirSync(path.join(bundles, "nsis"));
    writeFileSync(path.join(bundles, "nsis", "yunheshiguang_0.1.0_x64-setup.exe"), "setup");

    expect(collect(bundles, out, "windows", "x86_64", assets)).toEqual([
      "yunheshiguang-windows-x86_64-setup.exe",
    ]);
    expect(readFileSync(path.join(out, "yunheshiguang-windows-x86_64-setup.exe"), "utf8")).toBe(
      "setup",
    );

    writeFileSync(path.join(bundles, "nsis", "old_0.0.9_x64-setup.exe"), "old");
    expect(() => collect(bundles, out, "windows", "x86_64", assets)).toThrow(/found 2/);
  });

  it("writes checksums and a manifest in the shape the app reads", () => {
    const directory = built(temporary());
    const document = JSON.parse(manifest(directory, assets, info));
    const lines: string[] = checksums(directory, assets).trimEnd().split("\n");

    expect(Object.keys(document)).toEqual(Object.keys(fixture));
    expect(Object.keys(document.assets[0])).toEqual(Object.keys(fixture.assets[0]));
    expect(document.assets[0].url).toBe(
      "https://github.com/yunheshiguang/yunheshiguang-android/releases/download/nightly/yunheshiguang-windows-x86_64-setup.exe",
    );
    expect(lines).toHaveLength(assets.length);
    expect(lines.every((line) => /^[0-9a-f]{64} {2}yunheshiguang-\S+$/.test(line))).toBe(true);
    expect(lines.map((line) => line.slice(66))).toEqual(assets.map((asset) => asset.name).sort());
  });

  it("finds missing and unexpected files and unfinished uploads", () => {
    const directory = built(temporary());
    writeFileSync(path.join(directory, "notes.txt"), "stray");

    expect(verify(directory, assets.slice(1))).toEqual({
      missing: [],
      unexpected: [assets[0]?.name, "notes.txt"].sort(),
    });
    expect(
      uploaded(directory, {
        assets: [
          { name: assets[0]?.name, size: 1, state: "uploaded" },
          { name: assets[1]?.name, size: 99, state: "uploaded" },
          { name: "stale.dmg", size: 1, state: "uploaded" },
          { name: assets[2]?.name, size: 3, state: "starter" },
        ],
      }),
    ).toEqual([
      `${assets[2]?.name}: the upload did not finish`,
      ...[assets[2]?.name, assets[3]?.name, assets[4]?.name, assets[5]?.name, "notes.txt"]
        .sort()
        .map((name) => `${name}: missing from the release`),
      "stale.dmg: not part of this build",
      `${assets[1]?.name}: 99 bytes uploaded, 2 built`,
    ]);
  });

  it("keeps published names unless they are retired", () => {
    const [first, second] = assets as [Asset, Asset];

    expect(compatibility(assets, assets)).toEqual([]);
    expect(compatibility([first, second], [second])).toEqual([
      `${first.name}: a published asset cannot be removed or renamed unless it is listed as retired`,
    ]);
    expect(compatibility([first], [second], [first.name])).toEqual([]);
    expect(compatibility([first], [{ ...first, kind: "deb" }])).toEqual([
      `${first.name}: the os, arch, and kind of an asset cannot change`,
    ]);
  });

  it("lists the changes, the downloads, and how to verify them", () => {
    const text = notes([{ sha: info.commit, subject: "Fix pairing" }], assets, {
      ...info,
      previous: "fedcba9876543210fedcba9876543210fedcba98",
      since: "the previous nightly",
    });

    expect(text).toContain("**yunheshiguang Nightly**, version 0.1.0");
    expect(text).toContain("## Changes since the previous nightly");
    expect(text).toContain("- Fix pairing ([`0123456`]");
    expect(text).toContain("| Linux | x86_64 | AppImage | [yunheshiguang-linux-x86_64.AppImage]");
    expect(text).toContain("sha256sum --ignore-missing -c SHA256SUMS");
  });

  it("reads one app version from every manifest that declares it", () => {
    const directory = temporary();
    mkdirSync(path.join(directory, "src-tauri"));
    writeFileSync(path.join(directory, "Cargo.toml"), '[workspace.package]\nversion = "1.0.0"\n');
    writeFileSync(path.join(directory, "package.json"), '{ "version": "1.0.0" }');
    writeFileSync(path.join(directory, "src-tauri", "tauri.conf.json"), '{ "version": "1.0.1" }');

    expect(readVersion(root)).toMatch(/^\d+\.\d+\.\d+/);
    expect(() => readVersion(directory)).toThrow(/disagree/);
  });
});
