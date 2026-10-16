import { execFileSync } from "node:child_process";
import { existsSync, globSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import process from "node:process";
import { build, loadEnv, normalizePath } from "vite";

const root = process.cwd();
const failures = [];
const requireFromRoot = createRequire(path.join(root, "package.json"));

const extensions = globSync("extensions/*/web/package.json", { cwd: root }).map((manifest) => {
  const web = path.join(root, path.dirname(manifest));
  const nativeManifest = path.join(web, "..", "native", "Cargo.toml");
  const crate = existsSync(nativeManifest)
    ? readFileSync(nativeManifest, "utf8").match(/^name\s*=\s*"([^"]+)"/m)?.[1]
    : undefined;

  return { name: path.relative(root, web), web: `${normalizePath(web)}/`, crate };
});

async function bundledModules(mode) {
  const modules = new Set();

  await build({
    root,
    mode,
    logLevel: "error",
    plugins: [
      {
        name: "record-bundled-modules",
        generateBundle(_options, bundle) {
          for (const output of Object.values(bundle)) {
            for (const id of output.type === "chunk" ? output.moduleIds : []) {
              modules.add(normalizePath(id));
            }
          }
        },
      },
    ],
    build: { write: false, reportCompressedSize: false },
  });

  return modules;
}

function carries(modules, extension) {
  return [...modules].some((id) => id.startsWith(extension.web));
}

function nativeCrates(features) {
  const args = ["tree", "--locked", "-p", "yunheshiguang", "-e", "normal", "--prefix", "none"];
  const output = execFileSync(
    "cargo",
    features.length > 0 ? [...args, "--features", features.join(",")] : args,
    { cwd: root, encoding: "utf8" },
  );

  return new Set(output.split("\n").map((line) => line.split(" ")[0]));
}

function webOf(item) {
  if (item.startsWith(".")) {
    return `${normalizePath(path.resolve(root, item))}/`;
  }
  try {
    return `${normalizePath(path.dirname(requireFromRoot.resolve(`${item}/package.json`)))}/`;
  } catch {
    return undefined;
  }
}

function distributions() {
  return globSync(".env.*", { cwd: root })
    .map((file) => file.slice(".env.".length))
    .filter((mode) => !mode.endsWith("local"))
    .map((mode) => {
      const listed = (loadEnv(mode, root, "").WORKSPACE_EXTENSIONS ?? "")
        .split(",")
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => ({ item, web: webOf(item) }));
      for (const { item, web } of listed) {
        if (!extensions.some((extension) => extension.web === web)) {
          failures.push(`.env.${mode} lists ${item}, which is not an extension's web package`);
        }
      }
      const tauriConfig = path.join(root, "src-tauri", `tauri.${mode}.conf.json`);
      const features = existsSync(tauriConfig)
        ? (JSON.parse(readFileSync(tauriConfig, "utf8")).build?.features ?? [])
        : [];

      return {
        mode,
        features,
        extensions: extensions.filter((extension) =>
          listed.some(({ web }) => web === extension.web),
        ),
      };
    })
    .filter((distribution) => distribution.extensions.length > 0);
}

const openSource = await bundledModules("production");
const openSourceCrates = nativeCrates([]);

for (const extension of extensions) {
  if (carries(openSource, extension)) {
    failures.push(`the open-source bundle contains modules from ${extension.name}`);
  }

  if (extension.crate && openSourceCrates.has(extension.crate)) {
    failures.push(`the open-source app depends on ${extension.crate}`);
  }
}

console.log(`open source: ${openSource.size} modules, no extensions`);

for (const { mode, features, extensions: installed } of distributions()) {
  const modules = await bundledModules(mode);
  const crates = nativeCrates(features);

  for (const extension of installed) {
    if (!carries(modules, extension)) {
      failures.push(`the ${mode} bundle does not contain ${extension.name}`);
    }

    if (extension.crate && !crates.has(extension.crate)) {
      failures.push(
        `the ${mode} app does not depend on ${extension.crate}; check tauri.${mode}.conf.json`,
      );
    }
  }

  console.log(
    `${mode}: ${modules.size} modules, with ${installed.map((extension) => extension.name).join(", ")}`,
  );
}

if (failures.length > 0) {
  console.error(`Found ${failures.length} distribution problem(s):`);

  for (const failure of failures) {
    console.error(failure);
  }

  process.exit(1);
}

console.log(
  `The open-source build carries none of ${extensions.length} extension(s); every composed build carries its own.`,
);
