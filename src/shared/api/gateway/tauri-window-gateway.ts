import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

import type { WindowGateway } from "./window-gateway";

export function createTauriWindowGateway(): WindowGateway {
  return {
    minimize: () => getCurrentWindow().minimize(),
    close: () => getCurrentWindow().close(),
    toggleMaximize: () => getCurrentWindow().toggleMaximize(),
    isMaximized: () => getCurrentWindow().isMaximized(),
    async openWorkspaceWindow({ label, ...options }) {
      const child = new WebviewWindow(label, options);

      await new Promise<void>((resolve, reject) => {
        void child.once("tauri://created", () => {
          void child.setFocus();
          resolve();
        });
        void child.once<unknown>("tauri://error", (event) => {
          reject(new Error(String(event.payload)));
        });
      });
    },
    openExternal: (url) => openUrl(url),
  };
}
