import { error, warn } from "@tauri-apps/plugin-log";

import type { LogGateway } from "./log-gateway";

function swallow(write: Promise<void>) {
  write.catch(() => {});
}

export function createTauriLogGateway(): LogGateway {
  return {
    error: (message) => swallow(error(message)),
    warn: (message) => swallow(warn(message)),
  };
}
