import { glob, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const maxLines = 500;
const includedExtensions = ["ts", "tsx", "js", "mjs", "cjs", "rs", "css", "html"];
const ignoredDirectories = new Set([
  ".git",
  ".vscode",
  "dist",
  "node_modules",
  "coverage",
  "target",
  "icons",
  "gen",
]);
const ignoredBasenames = new Set([
  "Cargo.lock",
  "package-lock.json",
  "pnpm-lock.yaml",
  "LICENSE",
  "LICENSE.md",
  "LICENSE.txt",
  "LICENSE_APACHE-2.0",
  "LICENSE_MIT",
  "COPYING",
]);

function accept(file) {
  return !ignoredBasenames.has(path.basename(file));
}

const candidates = glob(`**/*.{${includedExtensions.join(",")}}`, {
  cwd: root,
  exclude: (entry) => ignoredDirectories.has(path.basename(entry)),
});
const files = (await Array.fromAsync(candidates, (relative) => path.join(root, relative))).filter(
  accept,
);
const violations = [];

for (const file of files) {
  const content = await readFile(file, "utf8");
  const lineCount = content === "" ? 0 : content.replace(/\r\n/g, "\n").split("\n").length;

  if (lineCount > maxLines) {
    violations.push({ file, lineCount });
  }
}

if (violations.length > 0) {
  console.error(`Found ${violations.length} file(s) over ${maxLines} lines:`);

  for (const violation of violations) {
    console.error(`${path.relative(root, violation.file)}: ${violation.lineCount}`);
  }

  process.exit(1);
}

console.log(`Checked ${files.length} source file(s); all within ${maxLines} lines.`);
