import type {
  AdbDeviceActionResultJson,
  DeviceScriptOutputJson,
  ScriptEnginesJson,
} from "@/shared/proto";

const SHELL_LABEL = { key: "workspace.scripts.shell.label" };

export function mockScriptEngines(): ScriptEnginesJson {
  return {
    engines: [
      {
        id: "core.shell",
        label: SHELL_LABEL,
        description: { key: "workspace.scripts.shell.description" },
        languages: ["SCRIPT_LANGUAGE_SHELL"],
        capturesOutput: true,
        setup: [],
      },
    ],
  };
}

export function readMockScript(
  body: string | undefined,
): { language?: string; body: string } | undefined {
  try {
    const script = JSON.parse(body ?? "") as { language?: string; body?: string };

    return typeof script.body === "string" && script.body.trim()
      ? { language: script.language, body: script.body }
      : undefined;
  } catch {
    return undefined;
  }
}

export function mockScriptOutputs(
  script: { language?: string; body: string },
  results: readonly AdbDeviceActionResultJson[],
): DeviceScriptOutputJson[] {
  const shell = script.language === "SCRIPT_LANGUAGE_SHELL";

  return results.map((result) => {
    if (!shell) {
      return {
        deviceId: result.deviceId,
        engineId: "",
        failure: { key: "workspace.scripts.noEngine" },
      };
    }

    return result.success
      ? {
          deviceId: result.deviceId,
          engineId: "core.shell",
          engine: SHELL_LABEL,
          exitCode: 0,
          output: script.body
            .split("\n")
            .filter((line) => line.trim())
            .map((line) => `$ ${line}\nok`)
            .join("\n"),
          truncated: false,
        }
      : {
          deviceId: result.deviceId,
          engineId: "core.shell",
          engine: SHELL_LABEL,
          failure: {
            key: "workspace.deviceShell.failed",
            params: { message: result.message ?? "" },
          },
        };
  });
}
