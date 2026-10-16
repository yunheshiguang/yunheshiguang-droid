import { useEffect, useRef } from "react";

import { commandGateway, type Unsubscribe } from "@/shared/api";

export function useWorkspaceEvent(event: string, handler: (payload: unknown) => void): void {
  const latest = useRef(handler);

  useEffect(() => {
    latest.current = handler;
  });

  useEffect(() => {
    let disposed = false;
    let stop: Unsubscribe | undefined;

    void commandGateway()
      .subscribe(event, (payload) => latest.current(payload))
      .then((unsubscribe) => {
        if (disposed) {
          unsubscribe();
        } else {
          stop = unsubscribe;
        }
      });

    return () => {
      disposed = true;
      stop?.();
    };
  }, [event]);
}
