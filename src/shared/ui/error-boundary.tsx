import type { ErrorInfo, ReactNode } from "react";
import { ErrorBoundary as ReactErrorBoundary } from "react-error-boundary";

import { errorMessage } from "@/shared/lib/error-message";

type ErrorBoundaryProps = {
  resetKey?: unknown;
  fallback: (error: Error, reset: () => void) => ReactNode;
  onError?: (error: Error, info: ErrorInfo) => void;
  children: ReactNode;
};

function asError(value: unknown): Error {
  return value instanceof Error ? value : new Error(errorMessage(value));
}

export function ErrorBoundary({ resetKey, fallback, onError, children }: ErrorBoundaryProps) {
  return (
    <ReactErrorBoundary
      resetKeys={[resetKey]}
      onError={(error, info) => onError?.(asError(error), info)}
      fallbackRender={({ error, resetErrorBoundary }) =>
        fallback(asError(error), resetErrorBoundary)
      }
    >
      {children}
    </ReactErrorBoundary>
  );
}
