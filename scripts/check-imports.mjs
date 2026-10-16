import { existsSync } from "node:fs";
import { glob, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const sourceRoot = path.join(root, "src");
const layers = new Set(["shared", "entities", "features", "widgets", "pages", "app"]);

const layersWithoutSlices = new Set(["shared"]);

function importsPackage(specifier, name) {
  return name.endsWith("/")
    ? specifier.startsWith(name)
    : specifier === name || specifier.startsWith(`${name}/`);
}

const packageBoundaries = [
  {
    packages: ["@tauri-apps/"],
    allows: (location) =>
      location.relative.startsWith(path.join("shared", "api", "gateway") + path.sep),
    rule: "reach the Tauri host only through src/shared/api/gateway",
  },
  {
    packages: [
      "@connectrpc/connect-query",
      "@tanstack/react-query",
      "@tanstack/query-core",
      "@tanstack/react-query-devtools",
      "zustand",
    ],
    allows: (location) => location.layer === "app" || ["model", "api"].includes(location.segment),
    rule: "keep state libraries inside model or api segments and expose them through hooks",
  },
  {
    packages: ["i18next"],
    allows: (location) => location.layer === "shared" && location.segment === "i18n",
    rule: "translate through useI18n from src/shared/i18n",
  },
  {
    packages: [
      "radix-ui",
      "@radix-ui/",
      "react-error-boundary",
      "@uiw/react-codemirror",
      "@codemirror/",
    ],
    allows: (location) => location.layer === "shared" && location.segment === "ui",
    rule: "build on the components exported from src/shared/ui",
  },
];

const restrictedDirectories = [
  {
    directory: path.join("shared", "testing"),
    rule: "only tests may use",
  },
  {
    directory: path.join("shared", "mock-host"),
    except: ["main.tsx"],
    rule: "only tests and the dev-only branch in src/main.tsx may use",
  },
  {
    directory: path.join("app", "sdk", "testing.ts"),
    rule: "only extension tests may use",
  },
];

function isInside(relative, directory) {
  return relative === directory || relative.startsWith(directory + path.sep);
}

function isTestInfrastructure(location, absolutePath) {
  return (
    /\.test\.tsx?$/.test(absolutePath) ||
    isInside(location.relative, restrictedDirectories[0].directory) ||
    isInside(location.relative, restrictedDirectories[2].directory)
  );
}

const importPattern = /(?:\bfrom\s*|(?:^|[^.\w])\bimport\s*|\bimport\s*\(\s*)["']([^"']+)["']/g;

function readImports(contents) {
  return Array.from(contents.matchAll(importPattern), (match) => match[1]);
}

function locate(absolutePath) {
  const relative = path.relative(sourceRoot, absolutePath);
  const [layer, slice, segment] = relative.split(path.sep);

  if (!relative.startsWith("..") && !relative.includes(path.sep)) {
    return { layer: "app", segment: undefined, relative };
  }
  if (relative.startsWith("..") || !layers.has(layer)) {
    return null;
  }

  return layersWithoutSlices.has(layer)
    ? { layer, segment: slice, relative }
    : { layer, segment, relative };
}

function resolveInSource(specifier, fromFile) {
  if (specifier.startsWith("@/")) {
    return path.join(sourceRoot, specifier.slice(2));
  }

  return specifier.startsWith(".") ? path.resolve(path.dirname(fromFile), specifier) : null;
}

const extensionsRoot = path.join(root, "extensions");

function globExtensions(pattern) {
  return existsSync(extensionsRoot) ? glob(pattern, { cwd: extensionsRoot }) : [];
}

async function extensionPackageNames() {
  const names = new Set();

  for await (const manifest of globExtensions("*/web/package.json")) {
    names.add(JSON.parse(await readFile(path.join(extensionsRoot, manifest), "utf8")).name);
  }

  return names;
}

const extensionNames = await extensionPackageNames();

function reachesExtension(specifier, sourcePath) {
  const targetPath = resolveInSource(specifier, sourcePath);

  return (
    [...extensionNames].some((name) => importsPackage(specifier, name)) ||
    (targetPath !== null && isInside(targetPath, extensionsRoot))
  );
}

function violationFor(source, sourcePath, specifier) {
  if (reachesExtension(specifier, sourcePath)) {
    return `imports "${specifier}"; the core must not depend on an extension`;
  }

  if (isTestInfrastructure(source, sourcePath)) {
    return null;
  }

  const boundary = packageBoundaries.find((candidate) =>
    candidate.packages.some((name) => importsPackage(specifier, name)),
  );

  if (boundary && !boundary.allows(source)) {
    return `imports "${specifier}"; ${boundary.rule}`;
  }

  const targetPath = resolveInSource(specifier, sourcePath);
  const target = targetPath ? locate(targetPath) : null;
  const restricted = target
    ? restrictedDirectories.find(
        ({ directory, except = [] }) =>
          isInside(target.relative, directory) &&
          !isInside(source.relative, directory) &&
          !except.includes(source.relative),
      )
    : undefined;

  if (restricted) {
    return `imports "${specifier}", which ${restricted.rule}`;
  }

  return null;
}

const violations = [];

for await (const relative of glob("**/*.{ts,tsx}", { cwd: sourceRoot })) {
  const sourcePath = path.join(sourceRoot, relative);
  const source = locate(sourcePath);

  if (!source) {
    continue;
  }

  const contents = await readFile(sourcePath, "utf8");

  for (const specifier of readImports(contents)) {
    const violation = violationFor(source, sourcePath, specifier);

    if (violation) {
      violations.push(`${path.relative(root, sourcePath)}: ${violation}`);
    }
  }
}

const extensionImports = {
  exact: ["yunheshiguang/sdk"],
  packages: ["react", "lucide-react"],
  testExact: ["yunheshiguang/sdk/testing"],
  testPackages: ["vitest", "@testing-library/"],
};

function extensionViolation(packageRoot, sourcePath, specifier) {
  if (specifier.startsWith(".")) {
    return isInside(path.resolve(path.dirname(sourcePath), specifier), packageRoot)
      ? null
      : `imports "${specifier}", which leaves the extension package`;
  }

  const isTest = /\.test\.tsx?$/.test(sourcePath);
  const exact = isTest
    ? [...extensionImports.exact, ...extensionImports.testExact]
    : extensionImports.exact;
  const packages = isTest
    ? [...extensionImports.packages, ...extensionImports.testPackages]
    : extensionImports.packages;

  return exact.includes(specifier) || packages.some((name) => importsPackage(specifier, name))
    ? null
    : `imports "${specifier}"; extensions may use the host only through yunheshiguang/sdk`;
}

for await (const relative of globExtensions("*/web/src/**/*.{ts,tsx}")) {
  const sourcePath = path.join(extensionsRoot, relative);
  const packageRoot = path.join(extensionsRoot, relative.split(path.sep)[0], "web");
  const contents = await readFile(sourcePath, "utf8");

  for (const specifier of readImports(contents)) {
    const violation = extensionViolation(packageRoot, sourcePath, specifier);

    if (violation) {
      violations.push(`${path.relative(root, sourcePath)}: ${violation}`);
    }
  }
}

if (violations.length > 0) {
  console.error(`Found ${violations.length} import boundary violation(s):`);

  for (const violation of violations) {
    console.error(violation);
  }

  process.exit(1);
}

console.log(
  "Third-party runtimes, test infrastructure, the mock host, and extensions stay where they belong.",
);
