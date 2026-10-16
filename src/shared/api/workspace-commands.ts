export const WORKSPACE_COMMANDS = {
  revealLogDirectory: "reveal_log_directory",
} as const;

export const WORKSPACE_EVENTS = {
  restoreLoading: "workspace:restore-loading",
  devicesChanged: "workspace:devices-changed",
  documentsChanged: "workspace:documents-changed",
  deviceProfilesChanged: "workspace:device-profiles-changed",
  automationChanged: "workspace:automation-changed",
  cloudPhonesChanged: "workspace:cloud-phones-changed",
  updateAvailable: "workspace:update-available",
} as const;

export type WorkspaceCommandName = (typeof WORKSPACE_COMMANDS)[keyof typeof WORKSPACE_COMMANDS];

export const workspaceCommandNames: readonly WorkspaceCommandName[] =
  Object.values(WORKSPACE_COMMANDS);
