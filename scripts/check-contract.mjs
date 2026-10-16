import { glob, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();

const commandsFile = path.join(root, "src/shared/api/workspace-commands.ts");
const invokeHandlerFile = path.join(root, "src-tauri/src/workspace/commands.rs");
const schemaCompatibilityFile = path.join(root, "src/shared/api/schema-compatibility.ts");
const snapshotFile = path.join(root, "src-tauri/src/workspace/snapshot/mod.rs");
const rustSourceRoot = path.join(root, "src-tauri/src");
const protoRoot = path.join(root, "proto/yunheshiguang");
const BRIDGE_COMMANDS = ["rpc", "rpc_cancel"];

function collectRecordValues(contents, constantName) {
  const block = contents.match(
    new RegExp(`export const ${constantName} = \\{([\\s\\S]*?)\\n\\} as const;`),
  );

  if (!block) {
    throw new Error(`${constantName} is missing or no longer a plain object literal`);
  }

  return new Set(Array.from(block[1].matchAll(/:\s*"([^"]+)"/g), (match) => match[1]));
}

function collectInvokeHandlerCommands(contents) {
  const block = contents.match(/tauri::generate_handler!\[([\s\S]*?)\]/);

  if (!block) {
    throw new Error("workspace_invoke_handler! no longer expands tauri::generate_handler!");
  }

  return new Set(
    block[1]
      .split(",")
      .map((entry) => entry.trim())
      .filter(Boolean),
  );
}

async function readRustSources(base = rustSourceRoot, pattern = "**/*.rs") {
  const files = await Array.fromAsync(glob(pattern, { cwd: base }), (relative) =>
    path.join(base, relative),
  );

  return Promise.all(
    files.map(async (file) => ({
      file: path.relative(root, file),
      contents: await readFile(file, "utf8"),
    })),
  );
}

async function collectRpcCommands() {
  const files = await Array.fromAsync(glob("**/*.proto", { cwd: protoRoot }), (relative) =>
    path.join(protoRoot, relative),
  );
  const sources = await Promise.all(files.map((file) => readFile(file, "utf8")));
  const commands = new Set();

  for (const source of sources) {
    for (const match of source.matchAll(/^\s*rpc (\w+)\(/gm)) {
      commands.add(match[1].replace(/(?<!^)[A-Z]/g, (letter) => `_${letter}`).toLowerCase());
    }
  }

  return commands;
}

function collectEmittedEvents(rustSources) {
  const emitted = new Set();

  for (const { contents } of rustSources) {
    for (const match of contents.matchAll(/"(workspace:[a-z0-9-]+)"/g)) {
      emitted.add(match[1]);
    }
  }

  return emitted;
}

const COMMAND_DEFINITION_PATTERN =
  /#\[tauri::command(?:\(([^)]*)\))?\]\s*(?:(?:\/\/[^\n]*|#\[[^\]]*\])\s*)*(?:pub(?:\([^)]*\))?\s+)?(async\s+)?fn\s+(\w+)/g;

function collectCommandDefinitions(rustSources) {
  const definitions = new Map();

  for (const { file, contents } of rustSources) {
    for (const match of contents.matchAll(COMMAND_DEFINITION_PATTERN)) {
      const [, attributeArgs = "", asyncKeyword, name] = match;

      definitions.set(name, {
        file,
        offMainThread: Boolean(asyncKeyword) || /\basync\b/.test(attributeArgs),
      });
    }
  }

  return definitions;
}

function checkCommandThreading(registered, definitions, problems) {
  for (const name of registered) {
    const definition = definitions.get(name);

    if (!definition) {
      problems.push(
        `command "${name}" is registered but its #[tauri::command] definition was not found`,
      );
    } else if (!definition.offMainThread) {
      problems.push(
        `command "${name}" in ${definition.file} is synchronous; Tauri runs it on the main thread, so make it async and move blocking work into run_blocking`,
      );
    }
  }
}

function reportMismatch(label, frontend, backend, problems) {
  for (const name of frontend) {
    if (!backend.has(name)) {
      problems.push(
        `${label} "${name}" is declared in the frontend but the backend never provides it`,
      );
    }
  }

  for (const name of backend) {
    if (!frontend.has(name)) {
      problems.push(
        `${label} "${name}" is provided by the backend but the frontend never declares it`,
      );
    }
  }
}

function extractLiteral(contents, pattern, label) {
  const match = contents.match(pattern);

  if (!match) {
    throw new Error(`${label} is missing or no longer a plain string literal`);
  }

  return match[1];
}

const [
  commandsSource,
  invokeHandlerSource,
  compatibilitySource,
  snapshotSource,
  rustSources,
  rpcCommands,
] = await Promise.all([
  readFile(commandsFile, "utf8"),
  readFile(invokeHandlerFile, "utf8"),
  readFile(schemaCompatibilityFile, "utf8"),
  readFile(snapshotFile, "utf8"),
  readRustSources(),
  collectRpcCommands(),
]);

const problems = [];
const registeredCommands = collectInvokeHandlerCommands(invokeHandlerSource);

const declaredSchemaVersion = extractLiteral(
  compatibilitySource,
  /EXPECTED_SCHEMA_VERSION = "([^"]+)"/,
  "EXPECTED_SCHEMA_VERSION",
);
const servedSchemaVersion = extractLiteral(
  snapshotSource,
  /SCHEMA_VERSION: &str = "([^"]+)"/,
  "SCHEMA_VERSION",
);

if (declaredSchemaVersion !== servedSchemaVersion) {
  problems.push(
    `schema version "${declaredSchemaVersion}" expected by the frontend does not match "${servedSchemaVersion}" served by the backend`,
  );
}

const hostCommands = new Set([
  ...BRIDGE_COMMANDS,
  ...[...collectRecordValues(commandsSource, "WORKSPACE_COMMANDS")].filter(
    (command) => !rpcCommands.has(command),
  ),
]);

reportMismatch("command", hostCommands, registeredCommands, problems);

reportMismatch(
  "event",
  collectRecordValues(commandsSource, "WORKSPACE_EVENTS"),
  collectEmittedEvents(rustSources),
  problems,
);

checkCommandThreading(registeredCommands, collectCommandDefinitions(rustSources), problems);

const extensionSources = await readRustSources(root, "extensions/*/native/src/**/*.rs");
const extensionCommands = new Set(
  extensionSources.flatMap(({ contents }) =>
    Array.from(contents.matchAll(/tauri::generate_handler!\[([\s\S]*?)\]/g), (match) =>
      match[1]
        .split(",")
        .map((entry) => entry.trim().split("::").at(-1))
        .filter(Boolean),
    ).flat(),
  ),
);
checkCommandThreading(extensionCommands, collectCommandDefinitions(extensionSources), problems);

if (problems.length > 0) {
  console.error(`Found ${problems.length} frontend/backend contract problem(s):`);

  for (const problem of problems) {
    console.error(problem);
  }

  process.exit(1);
}

console.log(
  "Host commands and event names match between the frontend and the backend, and every command runs off the main thread.",
);
