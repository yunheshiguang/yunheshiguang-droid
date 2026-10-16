import type { ErrorInfo } from "react";
import type { RootOptions } from "react-dom/client";

import { logGateway } from "@/shared/api";
import { errorMessage } from "@/shared/lib/error-message";

const COMPONENT_FRAMES = 6;

function describeError(error: unknown): string {
  return error instanceof Error ? `${error.name}: ${error.message}` : errorMessage(error);
}

export function describeRenderError(error: unknown, info: ErrorInfo): string {
  const frames = (info.componentStack ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .slice(0, COMPONENT_FRAMES);

  return [describeError(error), ...frames].join("\n  ");
}

export const renderErrorReporting: Pick<RootOptions, "onCaughtError" | "onUncaughtError"> = {
  onCaughtError(error, info) {
    console.error(error);
    logGateway().error(`render error in a module: ${describeRenderError(error, info)}`);
  },
  onUncaughtError(error, info) {
    console.error(error);
    logGateway().error(`render error in the shell: ${describeRenderError(error, info)}`);
  },
};

const reportedCopy = new Set<string>();

export function reportMissingCopy(key: string, locale: string) {
  const id = `${locale}\u0000${key}`;

  if (reportedCopy.has(id)) {
    return;
  }

  reportedCopy.add(id);
  logGateway().warn(`missing copy "${key}" in ${locale}`);
}

export function reportUnhandledErrors(target: Window = window): () => void {
  const handleError = (event: ErrorEvent) => {
    logGateway().error(`uncaught error: ${describeError(event.error ?? event.message)}`);
  };
  const handleRejection = (event: PromiseRejectionEvent) => {
    logGateway().error(`unhandled rejection: ${describeError(event.reason)}`);
  };
  const handleViolation = (event: SecurityPolicyViolationEvent) => {
    logGateway().warn(
      `content security policy blocked ${event.blockedURI || "inline code"} (${event.violatedDirective})`,
    );
  };

  target.addEventListener("error", handleError);
  target.addEventListener("unhandledrejection", handleRejection);
  target.document.addEventListener("securitypolicyviolation", handleViolation);

  return () => {
    target.removeEventListener("error", handleError);
    target.removeEventListener("unhandledrejection", handleRejection);
    target.document.removeEventListener("securitypolicyviolation", handleViolation);
  };
}
