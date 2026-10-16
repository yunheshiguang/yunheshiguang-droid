import { useCallback, useEffect, useState } from "react";

import { windowGateway } from "@/shared/api";

export function useWindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);

  const syncMaximized = useCallback(async () => {
    try {
      setIsMaximized(await windowGateway().isMaximized());
    } catch {
      setIsMaximized(false);
    }
  }, []);

  useEffect(() => {
    void syncMaximized();

    const handleResize = () => {
      void syncMaximized();
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, [syncMaximized]);

  const minimize = useCallback(() => {
    void windowGateway().minimize();
  }, []);

  const close = useCallback(() => {
    void windowGateway().close();
  }, []);

  const toggleMaximize = useCallback(async () => {
    await windowGateway().toggleMaximize();
    await syncMaximized();
  }, [syncMaximized]);

  return { isMaximized, minimize, close, toggleMaximize };
}
