import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { CommandGateway } from "./types";

export function createTauriCommandGateway(): CommandGateway {
  return {
    invoke: (command, payload) => invoke(command, payload),
    async subscribe(event, handler) {
      const unlisten = await getCurrentWindow().listen(event, (received) => {
        handler(received.payload);
      });

      return () => {
        void unlisten();
      };
    },
  };
}
