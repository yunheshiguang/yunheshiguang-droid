import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  globSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const here = path.dirname(fileURLToPath(import.meta.url));
export const root = path.resolve(here, "..", "..");
export const policyPath = path.join(here, "assets.json");
const policyFile = "scripts/release/assets.json";
export const CHECKSUMS = "SHA256SUMS";
export const MANIFEST = "release.json";
const NAME =
  /^yunheshiguang-(?<os>windows|macos|linux)-(?<arch>x86_64|arm64|universal)(?:-[a-z0-9]+)?(?:\.(?:exe|msi|dmg|AppImage|deb|rpm))?$/;
const SYSTEMS = { windows: "Windows", macos: "macOS", linux: "Linux" };
const KINDS = {
  installer: "Installer",
  "disk-image": "Disk image",
  appimage: "AppImage",
  deb: "Debian package",
  rpm: "RPM package",
};

export class ReleaseError extends Error {}

export function loadPolicy(file = policyPath) {
  const data = JSON.parse(readFileSync(file, "utf8"));
  if (data.schema_version !== 1) {
    throw new ReleaseError(`${file}: unsupported schema_version`);
  }
  const assets = data.assets ?? [];
  const retired = data.retired ?? [];
  const names = assets.map((asset) => asset.name);
  if (new Set(names).size !== names.length) {
    throw new ReleaseError(`${file}: duplicate asset names`);
  }
  if (new Set(retired).size !== retired.length || retired.some((name) => names.includes(name))) {
    throw new ReleaseError(`${file}: a retired name is repeated or still published`);
  }
  for (const asset of assets) {
    const match = NAME.exec(asset.name);
    if (!match) {
      throw new ReleaseError(`${asset.name}: breaks the naming rule`);
    }
    if (match.groups.os !== asset.os || match.groups.arch !== asset.arch) {
      throw new ReleaseError(`${asset.name}: os or arch disagrees with the name`);
    }
    if (!Object.hasOwn(KINDS, asset.kind)) {
      throw new ReleaseError(`${asset.name}: unknown kind ${asset.kind}`);
    }
    if (typeof asset.bundle !== "string" || asset.bundle === "") {
      throw new ReleaseError(`${asset.name}: no bundle pattern`);
    }
  }
  return { assets, retired };
}

export function collect(bundleDirectory, outDirectory, os, arch, assets) {
  const chosen = assets.filter((asset) => asset.os === os && asset.arch === arch);
  if (chosen.length === 0) {
    throw new ReleaseError(`no release asset is built on ${os} ${arch}`);
  }
  mkdirSync(outDirectory, { recursive: true });
  for (const asset of chosen) {
    const found = globSync(asset.bundle, { cwd: bundleDirectory });
    if (found.length !== 1) {
      throw new ReleaseError(
        `${asset.name}: expected one bundle matching ${asset.bundle}, found ${found.length}`,
      );
    }
    copyFileSync(path.join(bundleDirectory, found[0]), path.join(outDirectory, asset.name));
  }
  return chosen.map((asset) => asset.name);
}

function filesIn(directory) {
  return new Map(
    readdirSync(directory, { withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => [entry.name, statSync(path.join(directory, entry.name)).size]),
  );
}

export function verify(directory, assets) {
  const present = new Set(filesIn(directory).keys());
  const expected = new Set(assets.map((asset) => asset.name));

  return {
    missing: [...expected].filter((name) => !present.has(name)).sort(),
    unexpected: [...present].filter((name) => !expected.has(name)).sort(),
  };
}

export function uploaded(directory, release) {
  const built = filesIn(directory);
  const sizes = new Map();
  const problems = [];
  for (const asset of release.assets ?? []) {
    if (asset.state === "uploaded") {
      sizes.set(asset.name, asset.size);
    } else {
      problems.push(`${asset.name}: the upload did not finish`);
    }
  }
  for (const name of [...built.keys()].filter((name) => !sizes.has(name)).sort()) {
    problems.push(`${name}: missing from the release`);
  }
  for (const name of [...sizes.keys()].filter((name) => !built.has(name)).sort()) {
    problems.push(`${name}: not part of this build`);
  }
  for (const [name, size] of [...built].filter(([name]) => sizes.has(name))) {
    if (sizes.get(name) !== size) {
      problems.push(`${name}: ${sizes.get(name)} bytes uploaded, ${size} built`);
    }
  }
  return problems;
}

export function compatibility(previous, current, retired = []) {
  const byName = new Map(current.map((asset) => [asset.name, asset]));
  const problems = [];
  for (const asset of previous) {
    const kept = byName.get(asset.name);
    if (retired.includes(asset.name)) {
      continue;
    }
    if (!kept) {
      problems.push(
        `${asset.name}: a published asset cannot be removed or renamed unless it is listed as retired`,
      );
    } else if (kept.os !== asset.os || kept.arch !== asset.arch || kept.kind !== asset.kind) {
      problems.push(`${asset.name}: the os, arch, and kind of an asset cannot change`);
    }
  }
  return problems;
}

function baselineAssets(directory, ref) {
  try {
    const text = execFileSync("git", ["show", `${ref}:${policyFile}`], {
      cwd: directory,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    return JSON.parse(text).assets;
  } catch {
    return undefined;
  }
}

export function sha256(file) {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

export function checksums(directory, assets) {
  return assets
    .map((asset) => asset.name)
    .sort()
    .map((name) => `${sha256(path.join(directory, name))}  ${name}\n`)
    .join("");
}

export function downloadUrl(repo, tag, name) {
  return `https://github.com/${repo}/releases/download/${tag}/${name}`;
}

export function manifest(directory, assets, info) {
  const entries = assets.map((asset) => {
    const file = path.join(directory, asset.name);

    return {
      name: asset.name,
      os: asset.os,
      arch: asset.arch,
      kind: asset.kind,
      size: statSync(file).size,
      sha256: sha256(file),
      url: downloadUrl(info.repo, info.tag, asset.name),
    };
  });
  const document = {
    schema_version: 1,
    channel: info.channel,
    tag: info.tag,
    version: info.version,
    commit: info.commit,
    date: info.date,
    assets: entries,
  };

  return `${JSON.stringify(document, null, 2)}\n`;
}

export function readVersion(directory = root) {
  const read = (file) => readFileSync(path.join(directory, file), "utf8");
  const versions = {
    "Cargo.toml": /^version\s*=\s*"([^"]+)"/m.exec(read("Cargo.toml"))?.[1],
    "package.json": JSON.parse(read("package.json")).version,
    "src-tauri/tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
  };
  const distinct = new Set(Object.values(versions));
  if (distinct.size !== 1 || distinct.has(undefined)) {
    const listed = Object.entries(versions)
      .map(([file, version]) => `${file} ${version}`)
      .join(", ");
    throw new ReleaseError(`the app versions disagree: ${listed}`);
  }
  return versions["package.json"];
}

function commitsBetween(directory, previous, commit, limit) {
  const args = ["log", "--first-parent", "--format=%H%x1f%s%x1e"];
  if (!previous) {
    args.push(`--max-count=${limit}`);
  }
  args.push(previous ? `${previous}..${commit}` : commit);

  return execFileSync("git", args, { cwd: directory, encoding: "utf8" })
    .split("\x1e")
    .map((record) => record.trim())
    .filter(Boolean)
    .map((record) => {
      const [sha, subject] = record.split("\x1f");
      return { sha, subject };
    });
}

function commitLink(repo, sha) {
  return `[\`${sha.slice(0, 7)}\`](https://github.com/${repo}/commit/${sha})`;
}

export function notes(commits, assets, info) {
  const label = info.channel === "nightly" ? "Nightly" : info.tag;
  const lines = [
    `**yunheshiguang ${label}**, version ${info.version}, built on ${info.date} from ${commitLink(info.repo, info.commit)}.`,
    "",
    `## Changes since ${info.since || "the previous build"}`,
    "",
    info.previous
      ? `[\`${info.previous.slice(0, 7)}...${info.commit.slice(0, 7)}\`](https://github.com/${info.repo}/compare/${info.previous}...${info.commit}), ${commits.length} commits.`
      : `First build of this channel, latest ${commits.length} commits.`,
    "",
    ...(commits.length === 0
      ? ["No changes."]
      : commits.map(({ sha, subject }) => `- ${subject} (${commitLink(info.repo, sha)})`)),
    "",
    "## Downloads",
    "",
    "| System | Architecture | Package | File |",
    "| --- | --- | --- | --- |",
    ...assets.map(
      (asset) =>
        `| ${SYSTEMS[asset.os]} | ${asset.arch} | ${KINDS[asset.kind]} | [${asset.name}](${downloadUrl(info.repo, info.tag, asset.name)}) |`,
    ),
    "",
    "## Verify",
    "",
    `\`${CHECKSUMS}\` lists the SHA-256 of every file above, and \`${MANIFEST}\` carries the same data for scripts and for the update check in the app.`,
    "",
    "```sh",
    `sha256sum --ignore-missing -c ${CHECKSUMS}       # Linux`,
    `shasum -a 256 --ignore-missing -c ${CHECKSUMS}   # macOS`,
    "```",
    "",
    `On Windows, compare \`(Get-FileHash <file>).Hash\` with the line in \`${CHECKSUMS}\`.`,
    "",
    "These builds are not signed with a developer certificate. On macOS, open the app once, then choose Open Anyway in System Settings > Privacy & Security. On Windows, confirm the SmartScreen prompt with More info and Run anyway. On Linux, make the AppImage executable with `chmod +x` first.",
  ];

  return `${lines.join("\n")}\n`;
}

function write(text, output) {
  if (output) {
    writeFileSync(output, text, "utf8");
  } else {
    process.stdout.write(text);
  }
}

function info(values) {
  for (const key of ["repo", "tag", "channel", "commit", "date"]) {
    if (!values[key]) {
      throw new ReleaseError(`--${key} is required`);
    }
  }
  if (!["nightly", "release"].includes(values.channel)) {
    throw new ReleaseError("--channel is nightly or release");
  }
  return { ...values, version: values.version || readVersion() };
}

const options = {
  output: { type: "string", short: "o" },
  repo: { type: "string" },
  tag: { type: "string" },
  channel: { type: "string" },
  commit: { type: "string" },
  date: { type: "string" },
  version: { type: "string" },
  previous: { type: "string", default: "" },
  since: { type: "string" },
  limit: { type: "string", default: "50" },
  baseline: { type: "string", default: "" },
  os: { type: "string" },
  arch: { type: "string" },
  expect: { type: "string" },
};

export function main(argv) {
  const [command, ...rest] = argv;
  const { values, positionals } = parseArgs({ args: rest, options, allowPositionals: true });
  const { assets, retired } = loadPolicy();

  if (command === "collect") {
    const names = collect(positionals[0], positionals[1], values.os, values.arch, assets);
    console.log(`Collected ${names.join(", ")}.`);
  } else if (command === "verify") {
    const { missing, unexpected } = verify(positionals[0], assets);
    for (const name of missing) {
      console.error(`Missing release asset: ${name}`);
    }
    for (const name of unexpected) {
      console.error(`Unexpected release asset: ${name}`);
    }
    return missing.length + unexpected.length > 0 ? 1 : 0;
  } else if (command === "checksums") {
    write(checksums(positionals[0], assets), values.output);
  } else if (command === "manifest") {
    write(manifest(positionals[0], assets, info(values)), values.output);
  } else if (command === "notes") {
    const details = info(values);
    const commits = commitsBetween(root, details.previous, details.commit, Number(details.limit));
    write(notes(commits, assets, details), values.output);
  } else if (command === "uploaded") {
    const problems = uploaded(positionals[0], JSON.parse(readFileSync(positionals[1], "utf8")));
    for (const problem of problems) {
      console.error(problem);
    }
    return problems.length > 0 ? 1 : 0;
  } else if (command === "compat") {
    const previous = /^0*$/.test(values.baseline)
      ? undefined
      : baselineAssets(root, values.baseline);
    if (!previous) {
      console.log(
        `No release asset list at ${values.baseline || "the baseline"}; nothing to compare.`,
      );
      return 0;
    }
    const problems = compatibility(previous, assets, retired);
    for (const problem of problems) {
      console.error(problem);
    }
    return problems.length > 0 ? 1 : 0;
  } else if (command === "version") {
    const version = readVersion();
    if (values.expect && values.expect !== version) {
      throw new ReleaseError(
        `the tag asks for ${values.expect}, but the app is version ${version}`,
      );
    }
    console.log(version);
  } else {
    throw new ReleaseError(
      "usage: release.mjs collect|verify|checksums|manifest|notes|uploaded|compat|version",
    );
  }
  return 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(`release.mjs: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
