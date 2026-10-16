import { isTauri } from "@tauri-apps/api/core";

export function isTauriHost(): boolean {
  return isTauri();
}
