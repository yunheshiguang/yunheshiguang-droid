import type { WindowGateway } from "@/shared/api";

export const idleWindowGateway: WindowGateway = {
  minimize: async () => {},
  close: async () => {},
  toggleMaximize: async () => {},
  isMaximized: async () => false,
  openWorkspaceWindow: async () => {},
  openExternal: async () => {},
};
