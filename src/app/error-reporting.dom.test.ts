import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { type LogGateway, setLogGateway } from "@/shared/api";

import {
  describeRenderError,
  renderErrorReporting,
  reportMissingCopy,
  reportUnhandledErrors,
} from "./error-reporting";

function recordLogs() {
  const errors: string[] = [];
  const warnings: string[] = [];
  const gateway: LogGateway = {
    error: (message) => errors.push(message),
    warn: (message) => warnings.push(message),
  };

  setLogGateway(gateway);

  return { errors, warnings };
}

describe("error reporting", () => {
  let stopReporting: () => void = () => {};

  beforeEach(() => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
  });

  afterEach(() => {
    stopReporting();
    vi.restoreAllMocks();
  });

  it("keeps only the component frames nearest to the failure", () => {
    const stack = Array.from({ length: 20 }, (_, index) => `    at Frame${index}`).join("\n");
    const described = describeRenderError(new TypeError("boom"), { componentStack: stack });

    expect(described.startsWith("TypeError: boom")).toBe(true);
    expect(described).toContain("at Frame5");
    expect(described).not.toContain("at Frame6");
  });

  it("writes an error a boundary caught into the run log", () => {
    const { errors } = recordLogs();

    renderErrorReporting.onCaughtError?.(new Error("module crashed"), { componentStack: "" });

    expect(errors).toEqual(["render error in a module: Error: module crashed"]);
  });

  it("writes errors and rejections outside React into the run log", () => {
    const { errors } = recordLogs();
    stopReporting = reportUnhandledErrors(window);

    window.dispatchEvent(new ErrorEvent("error", { error: new RangeError("out of range") }));
    window.dispatchEvent(
      Object.assign(new Event("unhandledrejection"), { reason: "adb server is not running" }),
    );

    expect(errors).toEqual([
      "uncaught error: RangeError: out of range",
      "unhandled rejection: adb server is not running",
    ]);
  });

  it("logs missing copy once per key and locale, however often it renders", () => {
    const { warnings } = recordLogs();

    for (let render = 0; render < 3; render += 1) {
      reportMissingCopy("probe.onlyOnce", "en-US");
    }
    reportMissingCopy("probe.onlyOnce", "zh-CN");

    expect(warnings).toEqual([
      'missing copy "probe.onlyOnce" in en-US',
      'missing copy "probe.onlyOnce" in zh-CN',
    ]);
  });

  it("writes what the content security policy blocked into the run log", () => {
    const { warnings } = recordLogs();
    stopReporting = reportUnhandledErrors(window);

    document.dispatchEvent(
      Object.assign(new Event("securitypolicyviolation"), {
        blockedURI: "",
        violatedDirective: "script-src-elem",
      }),
    );

    expect(warnings).toEqual(["content security policy blocked inline code (script-src-elem)"]);
  });
});
