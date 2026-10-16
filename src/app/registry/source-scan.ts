import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

const SKIPPED_DIRECTORIES = new Set(["gen"]);
const SCANNER_FILES = new Set(["message-params.ts", "message-usage.ts", "source-scan.ts"]);

export type SourceFile = {
  file: string;
  contents: string;
};

function listFiles(dir: string, keep: (entry: string) => boolean): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const absolute = path.join(dir, entry);

    if (statSync(absolute).isDirectory()) {
      return SKIPPED_DIRECTORIES.has(entry) ? [] : listFiles(absolute, keep);
    }

    return keep(entry) && !SCANNER_FILES.has(entry) ? [absolute] : [];
  });
}

function read(root: string, keep: (entry: string) => boolean): SourceFile[] {
  return listFiles(path.join(process.cwd(), root), keep).map((absolute) => ({
    file: path.relative(process.cwd(), absolute),
    contents: readFileSync(absolute, "utf8"),
  }));
}

function extensionRoots(subdirectory: string): string[] {
  const extensions = path.join(process.cwd(), "extensions");

  if (!existsSync(extensions)) {
    return [];
  }

  return readdirSync(extensions)
    .map((name) => path.join("extensions", name, subdirectory))
    .filter((root) => existsSync(path.join(process.cwd(), root)));
}

export function frontendSources(): SourceFile[] {
  const keep = (entry: string) => /\.tsx?$/.test(entry) && !/\.test\.tsx?$/.test(entry);

  return [
    ...read("src", keep),
    ...extensionRoots(path.join("web", "src")).flatMap((sourceRoot) => read(sourceRoot, keep)),
  ];
}

export function rustSources(): SourceFile[] {
  const keep = (entry: string) => entry.endsWith(".rs");

  return [
    ...read("src-tauri/src", keep),
    ...extensionRoots(path.join("native", "src")).flatMap((sourceRoot) => read(sourceRoot, keep)),
  ];
}

export const BACKEND_MESSAGES_FILE = "src-tauri/src/workspace/messages.rs";

function extensionMessagesFiles(): string[] {
  return extensionRoots(path.join("native", "src"))
    .map((sourceRoot) => path.join(sourceRoot, "messages.rs"))
    .filter((file) => existsSync(path.join(process.cwd(), file)));
}

function keyConstantsIn(file: string): [string, string][] {
  const contents = readFileSync(path.join(process.cwd(), file), "utf8");

  return Array.from(
    contents.matchAll(/const\s+(\w+_KEY):\s*&str\s*=\s*\n?\s*"([^"]+)"/g),
    (match) => [match[1], match[2]],
  );
}

export function backendKeyConstants(): Map<string, string> {
  const constants = new Map<string, string>();

  for (const file of [BACKEND_MESSAGES_FILE, ...extensionMessagesFiles()]) {
    for (const [name, key] of keyConstantsIn(file)) {
      const existing = constants.get(name);

      if (existing !== undefined && existing !== key) {
        throw new Error(`${name} is ${existing} in one crate and ${key} in ${file}`);
      }

      constants.set(name, key);
    }
  }

  return constants;
}

export function extensionBackendKeys(): string[] {
  return extensionMessagesFiles().flatMap((file) => keyConstantsIn(file).map(([, key]) => key));
}
