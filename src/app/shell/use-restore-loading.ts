import { useEffect, useRef, useState } from "react";

import { WORKSPACE_EVENTS } from "@/shared/api";

import { useWorkspaceEvent } from "./use-workspace-event";

export function useRestoreLoading(holdMs = 900) {
  const [restoreLoading, setRestoreLoading] = useState(false);
  const hideTimeout = useRef<number | undefined>(undefined);

  useWorkspaceEvent(WORKSPACE_EVENTS.restoreLoading, () => {
    setRestoreLoading(true);
    window.clearTimeout(hideTimeout.current);
    hideTimeout.current = window.setTimeout(() => setRestoreLoading(false), holdMs);
  });

  useEffect(() => () => window.clearTimeout(hideTimeout.current), []);

  return restoreLoading;
}
