import { clone, create } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import {
  ScrcpyBinaryKind,
  ScrcpyBinarySource,
  ScrcpyConfigSchema,
  ScrcpyConnectionHealth,
  ScrcpyDiagnosticCode,
  ScrcpyInstallSchema,
  ScrcpyInstallStatus,
  type ScrcpyService,
  type ScrcpySession,
  ScrcpySessionSchema,
  ScrcpySessionStatus,
} from "@/shared/proto";

import type { MockContext } from "./mock-context";

const RELEASE = "v5.0";

const DIAGNOSTIC_KEYS: Partial<Record<ScrcpyDiagnosticCode, string>> = {
  [ScrcpyDiagnosticCode.BINARY_MISSING]: "binaryMissing",
  [ScrcpyDiagnosticCode.ALREADY_RUNNING]: "alreadyRunning",
  [ScrcpyDiagnosticCode.INVALID_SERIAL]: "invalidSerial",
};

export function createMockScrcpy(context: MockContext): ServiceImpl<typeof ScrcpyService> {
  let config = create(ScrcpyConfigSchema, {
    source: ScrcpyBinarySource.SYSTEM_PATH,
    videoEnabled: true,
    audioEnabled: true,
    controlEnabled: true,
  });
  let install = create(ScrcpyInstallSchema, { status: ScrcpyInstallStatus.NOT_INSTALLED });
  const sessions: ScrcpySession[] = [];

  const available = () =>
    config.source === ScrcpyBinarySource.APP_PRIVATE &&
    install.status === ScrcpyInstallStatus.INSTALLED;

  const state = (problem?: ScrcpyDiagnosticCode) => {
    const codes = [
      ...(problem ? [problem] : []),
      ...(available() ? [] : [ScrcpyDiagnosticCode.BINARY_MISSING]),
    ].filter((code, index, all) => all.indexOf(code) === index);
    const root = install.installRoot;

    return context.settle({
      config,
      binaries: [
        {
          kind: ScrcpyBinaryKind.CLIENT,
          resolvedPath: available() ? `${root}/scrcpy` : "",
          available: available(),
        },
        {
          kind: ScrcpyBinaryKind.SERVER,
          resolvedPath: available() ? `${root}/scrcpy-server` : "",
          available: available(),
        },
        { kind: ScrcpyBinaryKind.ADB, resolvedPath: "/mock/platform-tools/adb", available: true },
      ],
      adbPath: "/mock/platform-tools/adb",
      sessions: sessions.map((session) => clone(ScrcpySessionSchema, session)),
      diagnostics: codes.map((code) => {
        const key = DIAGNOSTIC_KEYS[code] ?? "unavailable";

        return {
          id: `scrcpy-${key}`,
          code,
          health:
            code === ScrcpyDiagnosticCode.BINARY_MISSING
              ? ScrcpyConnectionHealth.BLOCKED
              : ScrcpyConnectionHealth.DEGRADED,
          title: { key: `workspace.scrcpy.diagnostic.title.${key}` },
          impact: { key: "workspace.diagnostic.impact.default" },
          recovery: { key: `workspace.scrcpy.diagnostic.recovery.${key}` },
        };
      }),
      health: available() ? ScrcpyConnectionHealth.READY : ScrcpyConnectionHealth.BLOCKED,
      install,
    });
  };

  return {
    getScrcpyState: () => state(),
    updateScrcpyConfig: (request) => {
      config = request.config ? clone(ScrcpyConfigSchema, request.config) : config;
      context.audit("GOVERNED_ACTION_UPDATE_SCRCPY_CONFIG", true, []);
      return state();
    },
    installScrcpy: () => {
      install = create(ScrcpyInstallSchema, {
        status: ScrcpyInstallStatus.INSTALLED,
        releaseTag: RELEASE,
        installRoot: `/mock/app-data/scrcpy/${RELEASE}`,
      });
      config = { ...config, source: ScrcpyBinarySource.APP_PRIVATE };
      context.audit("GOVERNED_ACTION_INSTALL_SCRCPY", true, []);
      return state();
    },
    launchScrcpySession: (request) => {
      const serial = request.serial.trim();

      if (!available()) {
        return state(ScrcpyDiagnosticCode.BINARY_MISSING);
      }
      if (!context.devices().some((device) => device.id === serial)) {
        return state(ScrcpyDiagnosticCode.INVALID_SERIAL);
      }
      if (sessions.some((session) => session.serial === serial)) {
        return state(ScrcpyDiagnosticCode.ALREADY_RUNNING);
      }
      sessions.push(
        create(ScrcpySessionSchema, {
          id: crypto.randomUUID(),
          serial,
          status: ScrcpySessionStatus.RUNNING,
        }),
      );
      context.audit("GOVERNED_ACTION_MIRROR_DEVICE", true, [serial]);
      return state();
    },
    stopScrcpySession: (request) => {
      const index = sessions.findIndex((session) =>
        request.sessionId ? session.id === request.sessionId : session.serial === request.serial,
      );

      if (index >= 0) {
        sessions.splice(index, 1);
      }
      return state();
    },
  };
}
