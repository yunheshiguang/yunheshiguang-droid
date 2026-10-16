import { useEffect } from "react";

const BLOCKED_MODIFIER_KEYS = new Set(["r", "p", "s"]);

export function useShellInputGuards() {
  useEffect(() => {
    const handleContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      const key = event.key.toLowerCase();
      const modifierKey = event.ctrlKey || event.metaKey;
      const shouldBlock = key === "f5" || (modifierKey && BLOCKED_MODIFIER_KEYS.has(key));

      if (!shouldBlock) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
    };

    document.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("keydown", handleKeyDown, true);

    return () => {
      document.removeEventListener("contextmenu", handleContextMenu);
      window.removeEventListener("keydown", handleKeyDown, true);
    };
  }, []);
}

export function useBootSplashTeardown(elementId = "boot-splash") {
  useEffect(() => {
    document.getElementById(elementId)?.remove();
  }, [elementId]);
}
